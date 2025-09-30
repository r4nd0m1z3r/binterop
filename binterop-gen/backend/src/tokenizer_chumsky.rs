use ariadne::{Label, Report, ReportKind, Source};
use chumsky::prelude::*;
use std::{collections::VecDeque, path::PathBuf};

#[derive(Debug)]
pub enum Type<'a> {
    Named(&'a str),
    Array(&'a str, usize),
    Pointer(&'a str),
}

#[derive(Debug)]
pub enum Token<'a> {
    Struct(&'a str),
    Field(&'a str, Type<'a>),
}

pub fn parser<'a>(
    text: &'a str,
) -> impl Parser<'a, &'a str, Vec<Vec<Token<'a>>>, extra::Err<Rich<'a, char>>> {
    let struct_decl = text::keyword("struct")
        .padded()
        .ignore_then(text::ident().padded())
        .map(|name| Token::Struct(name));

    let array_parser = just('[')
        .ignore_then(text::ident())
        .then_ignore(just(':'))
        .then(text::int(10))
        .then_ignore(just(']'))
        .try_map(|(inner_type_name, size): (&str, &str), span| {
            let size = size.parse().map_err(|e| Rich::custom(span, e))?;
            Ok(Type::Array(inner_type_name, size))
        });
    let pointer_parser = text::ident().then_ignore(just('*')).map(Type::Pointer);
    let type_parser = choice((text::ident().map(Type::Named), array_parser, pointer_parser));

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
            fields.into()
        })
        .repeated()
        .collect()
}

pub struct Tokenizer<'a> {
    file_path: Option<PathBuf>,
    text: &'a str,
    tokens: Vec<Token<'a>>,
}
impl<'a> Tokenizer<'a> {
    pub fn new(file_path: Option<PathBuf>, text: &'a str) -> Self {
        let parser = parser(text);
        let (output, errors) = parser.parse(text).into_output_errors();

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
