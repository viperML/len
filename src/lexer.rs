use std::ops::Not;

use chumsky::input::StrInput;
use chumsky::text::Char;

use chumsky::Parser;
use chumsky::{input::ValueInput, prelude::*};
use crate::Int;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token<'src> {
    Number(Int),
    Bind,
    String(&'src str),
    Ident(&'src str),
    RightParenthesis,
    LeftParenthesis,
}


fn is_reserved_char(c: &char) -> bool {
    r#"(),;[]`{}_:"'"#.chars().any(|reserved| reserved == c.to_char())
}

fn unicode_ident<'a, I: StrInput<'a, C>, C: Char>() -> impl Parser<'a, I, &'a C::Str> + Copy + Clone
{
    let valid_ident =
        any().filter(|c: &C| !c.to_char().is_whitespace() && !is_reserved_char(&c.to_char()));

    valid_ident.then(valid_ident.repeated()).to_slice()
}

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Token<'src>>> {
    let number = text::int(10).map(|s: &str| Token::Number(s.parse().unwrap()));

    let string = any()
        .filter(|c| *c != '"')
        .repeated()
        .to_slice()
        .map(Token::String)
        .delimited_by(just('"'), just('"'));

    let right_parens = just(')').map(|_| Token::RightParenthesis);
    let left_parens = just('(').map(|_| Token::LeftParenthesis);

    let ident = unicode_ident().to_slice().map(Token::Ident);

    number
        .or(string)
        .or(right_parens)
        .or(left_parens)
        .or(ident)
        // .map_with_span(|token, span| (token, span))
        .padded()
        .repeated()
        .collect()
}


