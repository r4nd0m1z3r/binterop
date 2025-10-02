use ariadne::{Label, Report, ReportKind, Source};
use chumsky::{container::Container, prelude::*, text::Char};
use std::{
    collections::VecDeque,
    env,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum Type<'a> {
    Named(&'a str),
    Array(&'a str, usize),
    Vector(&'a str),
    Pointer(&'a str),
}

#[derive(Debug)]
pub enum Token<'a> {
    Struct(&'a str, Vec<(&'a str, Type<'a>)>),
    Enum(&'a str, Vec<&'a str>),
    Union(&'a str, Vec<&'a str>),
    Include(PathBuf, Vec<Token<'a>>),
}

pub fn struct_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, extra::Err<Rich<'a, char>>> {
    let struct_decl = text::keyword("struct")
        .padded()
        .ignore_then(text::ident().padded())
        .map(|name| Token::Struct(name, Vec::new()));

    let named_parser = text::ident().map(Type::Named);
    let array_parser = just('[')
        .padded()
        .ignore_then(text::ident())
        .then_ignore(just(':').padded())
        .then(text::int(10))
        .then_ignore(just(']').padded())
        .try_map(|(inner_type_name, size): (&str, &str), span| {
            let size = size.parse().map_err(|e| Rich::custom(span, e))?;
            Ok(Type::Array(inner_type_name, size))
        });
    let vector_parser = text::ident()
        .delimited_by(just('<').padded(), just('>').padded())
        .map(Type::Vector);
    let pointer_parser = text::ident().then_ignore(just('*')).map(Type::Pointer);

    let type_parser = choice((array_parser, vector_parser, pointer_parser, named_parser));

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

pub fn variants_parser<'a>() -> impl Parser<'a, &'a str, Vec<&'a str>, extra::Err<Rich<'a, char>>> {
    text::ident()
        .padded()
        .separated_by(just(','))
        .allow_trailing()
        .collect()
        .delimited_by(just('{').padded(), just('}').padded())
}

pub fn enum_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, extra::Err<Rich<'a, char>>> {
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

pub fn union_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, extra::Err<Rich<'a, char>>> {
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

pub fn include_parser<'a>() -> impl Parser<'a, &'a str, Token<'a>, extra::Err<Rich<'a, char>>> {
    let path_content = any()
        .filter(|c: &char| !c.is_newline())
        .repeated()
        .to_slice()
        .labelled("file path");
    let path_parser = path_content.try_map(|path: &'a str, span| {
        let path = todo!("Figure out a way to give parser a path to current file")
            .parent()
            .unwrap_or(&env::current_dir().unwrap_or_default())
            .join(path);

        Path::canonicalize(&path).map_err(|e| Rich::custom(span, e))
    });

    let include_decl = text::keyword("include")
        .padded()
        .ignore_then(path_parser)
        .map(|path| Token::Include(path, Vec::new()));

    include_decl
}

pub fn parser<'a, C: Container<Token<'a>>>(
) -> impl Parser<'a, &'a str, C, extra::Err<Rich<'a, char>>> {
    let parser = choice((
        include_parser(),
        struct_parser(),
        enum_parser(),
        union_parser(),
    ));

    parser.repeated().collect()
}

pub struct Tokenizer<'a> {
    file_path: Option<PathBuf>,
    tokens: VecDeque<Token<'a>>,
}
impl<'a> Tokenizer<'a> {
    pub fn new(file_path: Option<PathBuf>, text: &'a str) -> Self {
        let (output, errors) = parser().parse(text).into_output_errors();

        let report_source = file_path
            .as_ref()
            .map(|path| path.to_str().unwrap_or("default"))
            .unwrap_or("default");
        let reports = errors.into_iter().map(|err| {
            let span = (report_source, err.span().into_range());

            Report::build(ReportKind::Error, span.clone())
                .with_label(Label::new(span).with_message(err.reason()))
                .finish()
        });

        reports.for_each(|report| report.print((report_source, Source::from(text))).unwrap());

        dbg!(&output);

        Self {
            file_path,
            tokens: output.unwrap_or_default(),
        }
    }

    pub fn yield_token(&mut self) -> Option<Token<'a>> {
        self.tokens.pop_front()
    }
}
