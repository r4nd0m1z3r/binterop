use ariadne::{Label, Report, ReportKind, Source};
use chumsky::{container::Container, prelude::*, text::Char};
use std::{
    cell::Cell,
    collections::VecDeque,
    env, fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

type ParserExtra<'a> =
    extra::Full<Rich<'a, char>, extra::SimpleState<ParserState<'a, VecDeque<Token<'a>>>>, ()>;

#[derive(Debug)]
pub enum Type<'a> {
    Named(&'a str),
    Array(Box<Type<'a>>, usize),
    Vector(Box<Type<'a>>),
    Pointer(Box<Type<'a>>),
}

#[derive(Debug)]
pub enum Token<'a> {
    Struct(
        Vec<(String, String)>,
        &'a str,
        Vec<(Vec<(String, String)>, &'a str, Type<'a>)>,
    ),
    Enum(Vec<(String, String)>, &'a str, Vec<&'a str>),
    Union(Vec<(String, String)>, &'a str, Vec<&'a str>),
    Include(PathBuf, VecDeque<Token<'a>>),
    Function(&'a str, Vec<(&'a str, Type<'a>)>, Option<Type<'a>>),
}

fn type_parser<'a>() -> impl Parser<'a, &'a str, Type<'a>, ParserExtra<'a>> {
    recursive(|type_parser| {
        let named_parser = text::ident().map(Type::Named);

        let array_parser = type_parser
            .clone()
            .padded()
            .then_ignore(just(':').padded())
            .then(text::int(10))
            .delimited_by(just('[').padded(), just(']').padded())
            .try_map(|(inner_type, size): (Type, &str), span| {
                let size = size.parse().map_err(|e| Rich::custom(span, e))?;
                Ok(Type::Array(Box::new(inner_type), size))
            });

        let vector_parser = type_parser
            .clone()
            .padded()
            .delimited_by(just('<').padded(), just('>').padded())
            .map(|inner_type| Type::Vector(Box::new(inner_type)));

        let base_type_parser = choice((array_parser, vector_parser, named_parser));

        base_type_parser
            .then(just('*').padded().repeated().count())
            .map(|(mut ty, count)| {
                for _ in 0..count {
                    ty = Type::Pointer(Box::new(ty));
                }
                ty
            })
            .padded()
    })
}

fn attributes_parser<'a>() -> impl Parser<'a, &'a str, Vec<(String, String)>, ParserExtra<'a>> {
    let string_parser = one_of("\"'")
        .ignore_then(none_of("\"'").repeated().collect::<String>())
        .then_ignore(one_of("\"'"));
    let attribute = text::ident()
        .map(ToString::to_string)
        .then_ignore(just('='))
        .then(string_parser)
        .padded();
    let attributes = attribute
        .separated_by(just(','))
        .collect()
        .delimited_by(just("@[").padded(), just(']').padded())
        .padded();

    attributes
}

fn fields_parser<'a>(
    delimiter_start: char,
    delimiter_end: char,
) -> impl Parser<'a, &'a str, Vec<(Vec<(String, String)>, &'a str, Type<'a>)>, ParserExtra<'a>> {
    let field = attributes_parser()
        .or_not()
        .map(Option::unwrap_or_default)
        .then(text::ident())
        .padded()
        .then_ignore(just(':'))
        .then(type_parser())
        .map(|((attributes, field_name), ty)| (attributes, field_name, ty));

    field
        .separated_by(just(','))
        .allow_trailing()
        .collect()
        .delimited_by(just(delimiter_start).padded(), just(delimiter_end).padded())
}

fn struct_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, ParserExtra<'a>> {
    let struct_decl = attributes_parser()
        .or_not()
        .map(Option::unwrap_or_default)
        .then_ignore(text::keyword("struct"))
        .padded()
        .then(text::ident().padded())
        .map(|(attributes, name)| Token::Struct(attributes, name, Vec::new()));

    let fields = fields_parser('{', '}');

    struct_decl
        .then(fields)
        .map(|(mut struct_decl, fields)| {
            if let Token::Struct(_, _, struct_fields) = &mut struct_decl {
                *struct_fields = fields;
            } else {
                unreachable!(
                    "struct_decl is supposed to only yield Token::Struct, but got {struct_decl:?}"
                );
            }

            struct_decl
        })
        .padded()
}

fn variants_parser<'a>() -> impl Parser<'a, &'a str, Vec<&'a str>, ParserExtra<'a>> {
    text::ident()
        .padded()
        .separated_by(just(','))
        .allow_trailing()
        .collect()
        .delimited_by(just('{').padded(), just('}').padded())
}

fn enum_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, ParserExtra<'a>> {
    let enum_decl = attributes_parser()
        .or_not()
        .map(Option::unwrap_or_default)
        .then_ignore(text::keyword("enum"))
        .padded()
        .then(text::ident().padded())
        .map(|(attributes, name)| Token::Enum(attributes, name, Vec::new()));

    enum_decl
        .then(variants_parser())
        .map(|(mut enum_decl, variants)| {
            if let Token::Enum(_, _, enum_variants) = &mut enum_decl {
                *enum_variants = variants;
            } else {
                unreachable!(
                    "enum_decl is supposed to only yield Token::Enum, but got {enum_decl:?}"
                );
            }

            enum_decl
        })
        .padded()
}

fn union_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, ParserExtra<'a>> {
    let union_decl = attributes_parser()
        .or_not()
        .map(Option::unwrap_or_default)
        .then_ignore(text::keyword("union"))
        .padded()
        .then(text::ident().padded())
        .map(|(attributes, name)| Token::Union(attributes, name, Vec::new()));

    union_decl
        .then(variants_parser())
        .map(|(mut union_decl, variants)| {
            if let Token::Union(_, _, union_variants) = &mut union_decl {
                *union_variants = variants;
            } else {
                unreachable!(
                    "union_decl is supposed to only yield Token::Union, but got {union_decl:?}"
                );
            }

            union_decl
        })
        .padded()
}

fn include_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, ParserExtra<'a>> {
    let path_content = any()
        .filter(|c: &char| !c.is_newline())
        .repeated()
        .to_slice()
        .labelled("file path");
    let path_parser = path_content.try_map_with(move |path: &'a str, extra| {
        let span = extra.span();
        let state: &mut extra::SimpleState<ParserState<'a, _>> = extra.state();

        let path = state
            .file_path
            .parent()
            .unwrap_or(&env::current_dir().unwrap_or_default())
            .join(path);

        Path::canonicalize(&path).map_err(|e| Rich::custom(span, e))
    });

    let include_decl = text::keyword("include")
        .padded()
        .ignore_then(path_parser)
        .try_map_with(|path, extra| {
            let span = extra.span();
            let state: &mut extra::SimpleState<ParserState<'a, _>> = extra.state();

            let include = state.include(&path).map_err(|e| Rich::custom(span, e))?;
            let include_text = include.text;

            let parser = state.parser.clone();

            parser
                .parse_with_state(include_text, state)
                .into_result()
                .map(|include_tokens| Token::Include(path, include_tokens))
                .map_err(|errors| {
                    include.add_errors(errors);
                    Rich::custom(span, "error within included file")
                })
        });

    include_decl
}

fn function_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, ParserExtra<'a>> {
    let args_parser = fields_parser('(', ')')
        .map(|field| field.into_iter().map(|(_, name, ty)| (name, ty)).collect());
    text::keyword("fn")
        .padded()
        .ignore_then(text::ident())
        .then(args_parser)
        .then(just("->").padded().ignore_then(type_parser()).or_not())
        .map(|((name, args), return_type)| Token::Function(name, args, return_type))
}

fn parser<'a, C: Container<Token<'a>>>() -> impl Parser<'a, &'a str, C, ParserExtra<'a>> {
    let parser = choice((
        include_parser(),
        struct_parser(),
        enum_parser(),
        union_parser(),
        function_parser(),
    ));

    parser.repeated().collect()
}

struct Include<'a> {
    path: PathBuf,
    text: &'a str,
    errors: Cell<Vec<Rich<'a, char>>>,
}
impl<'a> Include<'a> {
    fn new(path: PathBuf, text: &'a str) -> Self {
        Self {
            path,
            text,
            errors: Cell::default(),
        }
    }

    fn add_errors(&self, errors: Vec<Rich<'a, char>>) {
        self.errors.set(errors);
    }
}
impl<'a> Drop for Include<'a> {
    fn drop(&mut self) {
        // SAFETY: self.text is a leaked string, trimmed to its length
        let _ = unsafe {
            String::from_raw_parts(
                self.text.as_ptr().cast_mut(),
                self.text.len(),
                self.text.len(),
            )
        };
    }
}

struct ParserState<'a, C: Container<Token<'a>>> {
    parser: Boxed<'a, 'a, &'a str, C, ParserExtra<'a>>,
    file_path: Arc<PathBuf>,
    includes: Vec<Arc<Include<'a>>>,
}
impl<'a, C: Container<Token<'a>>> ParserState<'a, C> {
    fn new(
        parser: Boxed<'a, 'a, &'a str, C, ParserExtra<'a>>,
        file_path: Option<Arc<PathBuf>>,
    ) -> extra::SimpleState<Self> {
        extra::SimpleState(Self {
            parser,
            file_path: file_path.unwrap_or_else(|| Arc::new(PathBuf::new())),
            includes: Vec::new(),
        })
    }

    fn include<'b>(&'b mut self, path: &Path) -> Result<Arc<Include<'a>>, io::Error> {
        let mut text = fs::read_to_string(&path)?;
        text.shrink_to_fit();

        let include = Arc::new(Include::new(path.to_path_buf(), text.leak()));
        self.includes.push(include.clone());

        Ok(include)
    }
}

pub struct Tokenizer<'a> {
    _file_path: Option<Arc<PathBuf>>,
    _state: extra::SimpleState<ParserState<'a, VecDeque<Token<'a>>>>,
    tokens: Arc<VecDeque<Token<'a>>>,
    failed: bool,
}
impl<'a> Tokenizer<'a> {
    pub fn new(file_path: Option<&Path>, text: &'a str) -> Self {
        let file_path = file_path.map(Path::to_path_buf).map(Arc::new);

        let parser = parser().boxed();
        let mut state = ParserState::new(parser.clone(), file_path.clone());

        let (output, errors) = parser
            .parse_with_state(text, &mut state)
            .into_output_errors();
        let failed = !errors.is_empty();

        let include_errors = state.includes.iter().flat_map(|include| {
            let source_path = include.path.to_str().unwrap();

            include
                .errors
                .take()
                .into_iter()
                .map(|err| (source_path, err, include.text))
                .collect::<Vec<_>>()
        });
        let report_source = file_path
            .as_ref()
            .map(|path| path.to_str().unwrap_or("default"))
            .unwrap_or("default");
        let reports = errors
            .into_iter()
            .map(|err| (report_source, err, text))
            .chain(include_errors)
            .map(|(report_path, err, source)| {
                let span = (report_path, err.span().into_range());

                (
                    report_path,
                    source,
                    Report::build(ReportKind::Error, span.clone())
                        .with_label(Label::new(span).with_message(err.reason()))
                        .finish(),
                )
            });

        reports.for_each(|(report_path, source, report)| {
            report.print((report_path, Source::from(source))).unwrap();
        });

        Self {
            _file_path: file_path,
            _state: state,
            tokens: Arc::new(output.unwrap_or_default()),
            failed,
        }
    }

    pub fn tokens(&mut self) -> Option<Arc<VecDeque<Token<'a>>>> {
        if self.failed {
            None
        } else {
            Some(self.tokens.clone())
        }
    }
}
