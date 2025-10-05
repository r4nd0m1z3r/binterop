use ariadne::{Label, Report, ReportKind, Source};
use chumsky::{container::Container, prelude::*, text::Char};
use std::{
    cell::{Cell, RefCell},
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
    Struct(&'a str, Vec<(&'a str, Type<'a>)>),
    Enum(&'a str, Vec<&'a str>),
    Union(&'a str, Vec<&'a str>),
    Include(PathBuf, VecDeque<Token<'a>>),
}

fn struct_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, ParserExtra<'a>> {
    let struct_decl = text::keyword("struct")
        .padded()
        .ignore_then(text::ident().padded())
        .map(|name| Token::Struct(name, Vec::new()));

    let type_parser = recursive(|type_parser| {
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
    });

    let field = text::ident()
        .padded()
        .then_ignore(just(':'))
        .then(type_parser.padded())
        .map(|(field_name, ty)| (field_name, ty));

    let fields = field
        .separated_by(just(','))
        .allow_trailing()
        .collect()
        .delimited_by(just('{').padded(), just('}').padded());

    struct_decl
        .then(fields)
        .map(|(mut struct_decl, fields)| {
            if let Token::Struct(_, struct_fields) = &mut struct_decl {
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
    let enum_decl = text::keyword("enum")
        .padded()
        .ignore_then(text::ident().padded())
        .map(|name| Token::Enum(name, Vec::new()));

    enum_decl
        .then(variants_parser())
        .map(|(mut enum_decl, variants)| {
            if let Token::Enum(_, enum_variants) = &mut enum_decl {
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
    let union_decl = text::keyword("union")
        .padded()
        .ignore_then(text::ident().padded())
        .map(|name| Token::Union(name, Vec::new()));

    union_decl
        .then(variants_parser())
        .map(|(mut union_decl, variants)| {
            if let Token::Union(_, union_variants) = &mut union_decl {
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

fn parser<'a, C: Container<Token<'a>>>() -> impl Parser<'a, &'a str, C, ParserExtra<'a>> {
    let parser = choice((
        include_parser(),
        struct_parser(),
        enum_parser(),
        union_parser(),
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
        // This whole thing is needed since chumsky doesn't support arc'ed strings
        // SAFETY: We're reconstructing the string from leaked str which was previously shrinked to length
        // TODO: We probably should implement our custom arc'ed string type that should implement Input
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
        let text = text.leak();

        let include = Include::new(path.to_path_buf(), text);
        self.includes.push(Arc::new(include));
        let include = self.includes.last().cloned().unwrap();

        Ok(include)
    }
}

pub struct Tokenizer<'a> {
    file_path: Option<Arc<PathBuf>>,
    tokens: VecDeque<Token<'a>>,
}
impl<'a> Tokenizer<'a> {
    pub fn new(file_path: Option<&Path>, text: &'a str) -> Self {
        let file_path = file_path.map(Path::to_path_buf).map(Arc::new);

        let parser = parser().boxed();
        let mut state = ParserState::new(parser.clone(), file_path.clone());

        let (output, errors) = parser
            .parse_with_state(text, &mut state)
            .into_output_errors();

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

        dbg!(&output);

        Self {
            file_path,
            tokens: output.unwrap_or_default(),
        }
    }

    pub fn tokens(&mut self) -> impl Iterator<Item = &Token<'a>> {
        self.tokens.iter()
    }
}
