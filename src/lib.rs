#![allow(dead_code, unused_imports)]
#[macro_use]
extern crate educe;

pub mod eval;
#[cfg(test)]
mod test;

use std::ops::Not;

use bumpalo::Bump;
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

fn is_infix(ident: &str) -> bool {
    ident
        .chars()
        .next()
        .unwrap_or_default()
        .is_alphabetic()
        .not()
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

    let ident = unicode_ident().map_slice(Token::Ident);

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
    Number(Int),
    Boolean(bool),
    Binary {
        op: &'src str,
        left: &'src Self,
        right: &'src Self,
    },
}

#[derive(Debug, Clone)]
pub enum Ast {
    Literal(Literal),
    FunctionCall(FunctionCall),
    Identifier(Identifier),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(Int),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    function: Box<Ast>,
    argument: Box<Ast>,
}

#[derive(Debug, Clone)]
pub struct Identifier {
    name: String,
}

#[derive(Debug, PartialEq)]
pub struct Spanned2<T>(T, SimpleSpan<usize>);

pub fn parser<'src>(
) -> impl Parser<'src, &'src [Token<'src>], Ast, extra::Err<Rich<'src, Token<'src>>>> {
    recursive(|expr| {
        let literal = select! {
            Token::Ident("true") => Literal::Boolean(true),
            Token::Ident("false") => Literal::Boolean(false),
            Token::Number(x) => Literal::Integer(x),
            Token::String(x) => Literal::String(x.to_string()),
        }
        .map(Ast::Literal);

        let ident = select! {
            Token::Ident(x) => x,
        }
        .filter(|s| is_infix(s).not())
        .map(|s| {
            Ast::Identifier(Identifier {
                name: s.to_string(),
            })
        });

        let grouping = expr
            .clone()
            .delimited_by(just(Token::LeftParenthesis), just(Token::RightParenthesis));

        let atom = literal.or(ident).or(grouping).labelled("non-infix atom");

        let func = atom.clone().foldl(atom.clone().repeated(), |x, y| {
            Ast::FunctionCall(FunctionCall {
                function: Box::new(x),
                argument: Box::new(y),
            })
        });

        func.foldl(
            select! {
                Token::Ident(s) => {
                    if is_infix(s) {
                        Some( Ast::Identifier(Identifier {
                            name: s.to_string(),
                        }))
                    } else {
                        None
                    }
                },
            }
            .filter(|ast| ast.is_some())
            .map(|ast| ast.unwrap())
            .then(atom)
            .repeated(),
            |first, (op, second)| {
                let first_op = Ast::FunctionCall(FunctionCall {
                    function: Box::new(op),
                    argument: Box::new(first),
                });
                Ast::FunctionCall(FunctionCall {
                    function: Box::new(first_op),
                    argument: Box::new(second),
                })
            },
        )
    })
}
