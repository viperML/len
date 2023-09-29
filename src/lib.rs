#![allow(dead_code, unused_imports)]

#[cfg(test)]
mod test;
pub mod eval;

use chumsky::extra::ParserExtra;
use chumsky::input::StrInput;
use chumsky::text::Char;
use chumsky::util::MaybeRef;
use chumsky::Parser;
use chumsky::{
    combinator::To,
    input::{Stream, ValueInput},
    prelude::*,
};
use num::BigInt;
use tracing::{debug, info};
use tracing_test::traced_test;

type Int = num::BigInt;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Token<'src> {
    Number(Int),
    Bind,
    String(&'src str),
    Ident(&'src str),
    RightParenthesis,
    LeftParenthesis,
}

type Span = SimpleSpan;
type Spanned<T> = (T, Span);

fn is_reserved_char(c: &char) -> bool {
    r#"(),;[]`{}_:"'"#.chars().any(|reserved| reserved == c.to_char())
}

fn unicode_ident<'a, I: StrInput<'a, C>, C: Char>() -> impl Parser<'a, I, &'a C::Str> + Copy + Clone
{
    let valid_ident =
        any().filter(|c: &C| !c.to_char().is_whitespace() && !is_reserved_char(&c.to_char()));

    valid_ident.then(valid_ident.repeated()).slice()
}

pub fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Token<'src>>> {
    let number = text::int(10).map(|s: &str| Token::Number(s.parse().unwrap()));

    let string = any()
        .filter(|c| *c != '"')
        .repeated()
        .map_slice(Token::String)
        .delimited_by(just('"'), just('"'));


    let right_parens = just(')').map(|_| Token::RightParenthesis);
    let left_parens = just('(').map(|_| Token::LeftParenthesis);

    let ident = unicode_ident().map_slice(|s| Token::Ident(s));

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

#[allow(dead_code)]
#[derive(Debug)]
pub enum Expression<'src> {
    String(&'src str),
    Number(BigInt),
    Boolean(bool),
    Binary {
        op: &'src str,
        left: Box<Self>,
        right: Box<Self>,
    },
}

#[derive(Debug, PartialEq)]
pub struct Spanned2<T>(T, SimpleSpan<usize>);

pub fn parser<'src>(
) -> impl Parser<'src, &'src [Token<'src>], Expression<'src>, extra::Err<Rich<'src, Token<'src>>>> {
    recursive(|expr| {
        let literal = select! {
            Token::Ident("true") => Expression::Boolean(true),
            Token::Ident("false") => Expression::Boolean(false),
            Token::Number(x) => Expression::Number(x),
            Token::String(x) => Expression::String(x),
        };

        let grouping = expr
            .clone()
            .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis));

        let atom = literal.or(grouping).labelled("atom");

        // let factor = atom
        //     .clone()
        //     .then(select! {
        //         Token::Ident("*") => "*",
        //         Token::Ident("/") => "/",
        //     }.then(atom).repeated()).foldl(|x, y| {});

        let factor = atom.clone().foldl(
            select! {
                Token::Ident(x @ "*") => x,
                Token::Ident(x @ "/") => x,
            }
            .then(atom.clone())
            .repeated(),
            |lhs, (op, rhs)| Expression::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            },
        );

        let sum = factor.clone().foldl(
            select! {
                Token::Ident(x @ "+") => x,
                Token::Ident(x @ "-") => x,
            }
            .then(factor)
            .repeated(),
            |lhs, (op, rhs)| Expression::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            },
        );

        let equality = sum.clone().foldl(
            select! {
                Token::Ident(x @ "==") => x,
                Token::Ident(x @ "!=") => x,
            }
            .then(sum)
            .repeated(),
            |lhs, (op, rhs)| Expression::Binary {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            },
        );

        equality
    })
}
