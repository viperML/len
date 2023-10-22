use std::ops::Not;

use chumsky::input::StrInput;
use chumsky::pratt::{infix, left, prefix};
use chumsky::text::Char;

use crate::lexer::TokenKind;
use crate::Int;
use chumsky::Parser;
use chumsky::{input::ValueInput, prelude::*};

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
    pub(crate) function: Box<Ast>,
    pub(crate) argument: Box<Ast>,
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub(crate) name: String,
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

pub fn expression_parser_old<'src>(
) -> impl Parser<'src, &'src [TokenKind<'src>], Ast, extra::Err<Rich<'src, TokenKind<'src>>>> {
    recursive(|expr| {
        let literal = select! {
            TokenKind::Ident("true") => Literal::Boolean(true),
            TokenKind::Ident("false") => Literal::Boolean(false),
            TokenKind::Number(x) => Literal::Integer(x),
            TokenKind::String(x) => Literal::String(x.to_string()),
        }
        .map(Ast::Literal);

        let ident = select! {
            TokenKind::Ident(x) => x,
        }
        .filter(|s| is_infix(s).not())
        .map(|s| {
            Ast::Identifier(Identifier {
                name: s.to_string(),
            })
        });

        let grouping = expr
            .clone()
            .delimited_by(just(TokenKind::LeftParenthesis), just(TokenKind::RightParenthesis));

        let atom = literal.or(ident).or(grouping).labelled("non-infix atom");

        let func = atom.clone().foldl(atom.clone().repeated(), |x, y| {
            Ast::FunctionCall(FunctionCall {
                function: Box::new(x),
                argument: Box::new(y),
            })
        });

        func.foldl(
            select! {
                TokenKind::Ident(s) => {
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

pub fn expression_parser<'src>(
) -> impl Parser<'src, &'src [TokenKind<'src>], Ast, extra::Err<Rich<'src, TokenKind<'src>>>> {
    let atom = recursive(|expr| {
        let literal = select! {
            TokenKind::Ident("true") => Literal::Boolean(true),
            TokenKind::Ident("false") => Literal::Boolean(false),
            TokenKind::Number(x) => Literal::Integer(x),
            TokenKind::String(x) => Literal::String(x.to_string()),
        }
        .map(Ast::Literal);

        let ident = select! {
            TokenKind::Ident(x) => x,
        }
        .filter(|s| is_infix(s).not())
        .map(|s| {
            Ast::Identifier(Identifier {
                name: s.to_string(),
            })
        });

        let grouping = expr
            .clone()
            .delimited_by(just(TokenKind::LeftParenthesis), just(TokenKind::RightParenthesis));

        literal.or(ident).or(grouping).labelled("non-infix atom")
    });

    let infix_ident = select! {
        TokenKind::Ident(s) => {
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
    .map(|ast| ast.unwrap());

    atom.clone().pratt((
        prefix(3, atom, |op: Ast, o: Ast| {
            Ast::FunctionCall(FunctionCall {
                function: Box::new(op),
                argument: Box::new(o),
            })
        }),
        infix(left(2), infix_ident, |left: Ast, op: Ast, right: Ast| {
            let first_op = Ast::FunctionCall(FunctionCall {
                function: Box::new(op),
                argument: Box::new(left),
            });
            Ast::FunctionCall(FunctionCall {
                function: Box::new(first_op),
                argument: Box::new(right),
            })
        }),
    ))
}
