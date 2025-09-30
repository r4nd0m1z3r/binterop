use ariadne::{Label, Report, ReportKind, Source};
use chumsky::prelude::*;
use std::{collections::VecDeque, path::PathBuf};

#[derive(Debug)]
pub enum Type<'a> {
    Named(&'a str),
    Array(&'a str, usize),
    Vector(&'a str),
    Pointer(&'a str),
}

#[derive(Debug)]
pub enum Token<'a> {
    Struct(&'a str),
    Field(&'a str, Type<'a>),
    Enum(&'a str),
    Union(&'a str),
    Variant(&'a str),
}

pub fn struct_parser<'a>(
) -> impl Parser<'a, &'a str, VecDeque<Token<'a>>, extra::Err<Rich<'a, char>>> {
    let struct_decl = text::keyword("struct")
        .padded()
        .ignore_then(text::ident().padded())
        .map(|name| Token::Struct(name));

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
        .map(|(field_name, ty)| Token::Field(field_name, ty));

    let fields = field
        .separated_by(just(','))
        .allow_trailing()
        .collect::<VecDeque<_>>()
        .delimited_by(just('{').padded(), just('}').padded());

    struct_decl
        .then(fields)
        .map(|(struct_decl, mut fields)| {
            fields.push_front(struct_decl);
            fields
        })
        .padded()
}

pub fn variants_parser<'a>(
) -> impl Parser<'a, &'a str, VecDeque<Token<'a>>, extra::Err<Rich<'a, char>>> {
    text::ident()
        .padded()
        .map(Token::Variant)
        .separated_by(just(','))
        .allow_trailing()
        .collect::<VecDeque<_>>()
        .delimited_by(just('{').padded(), just('}').padded())
}

pub fn enum_parser<'a>() -> impl Parser<'a, &'a str, VecDeque<Token<'a>>, extra::Err<Rich<'a, char>>>
{
    let enum_decl = text::keyword("enum")
        .padded()
        .ignore_then(text::ident().padded())
        .map(Token::Enum);

    enum_decl
        .then(variants_parser())
        .map(|(enum_decl, mut variants)| {
            variants.push_front(enum_decl);
            variants.into()
        })
        .padded()
}

pub fn union_parser<'a>(
) -> impl Parser<'a, &'a str, VecDeque<Token<'a>>, extra::Err<Rich<'a, char>>> {
    let union_decl = text::keyword("union")
        .padded()
        .ignore_then(text::ident().padded())
        .map(Token::Union);

    union_decl
        .then(variants_parser())
        .map(|(union_decl, mut variants)| {
            variants.push_front(union_decl);
            variants
        })
        .padded()
}

pub fn parser<'a>() -> impl Parser<'a, &'a str, Vec<VecDeque<Token<'a>>>, extra::Err<Rich<'a, char>>>
{
    let parser = choice((struct_parser(), enum_parser(), union_parser()));

    parser.repeated().collect()
}

pub struct Tokenizer<'a> {
    file_path: Option<PathBuf>,
    text: &'a str,
    tokens: Vec<Token<'a>>,
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

        dbg!(output);

        Self {
            file_path,
            text,
            tokens: vec![Token::Struct("TODO")],
        }
    }

    pub fn yield_token(&mut self) -> Option<Token<'a>> {
        self.tokens.pop()
    }
}
