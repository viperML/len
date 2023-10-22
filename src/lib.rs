// #![allow(dead_code, unused_imports)]
#[macro_use]
extern crate educe;

pub mod eval;
pub mod lexer;
pub mod ty;

use std::ops::Not;

use chumsky::input::StrInput;
use chumsky::text::Char;

use chumsky::Parser;
use chumsky::{input::ValueInput, prelude::*};
use lexer::Token;

pub type Int = num::BigInt;

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

fn is_infix(ident: &str) -> bool {
    ident
        .chars()
        .next()
        .unwrap_or_default()
        .is_alphabetic()
        .not()
}

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
