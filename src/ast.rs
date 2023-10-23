use crate::lexer::TokenKind;
use crate::Int;
use chumsky::extra::ParserExtra;
use chumsky::pratt::{infix, left, prefix};
use chumsky::prelude::*;
use chumsky::Parser;
use std::borrow::Cow;
use std::ops::Not;

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

pub fn expression_parser<'src, E: ParserExtra<'src, &'src [TokenKind<'src>]>>(
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
            TokenKind::Ident(s) => Identifier { name: s.to_string() },
        }
        .map(Ast::Identifier);

        let symbol = select! {
            TokenKind::Symbol(s) => Identifier { name: s.to_string() },
        }
        .map(Ast::Identifier);

        let grouping = expr.clone().delimited_by(
            just(TokenKind::LeftParenthesis),
            just(TokenKind::RightParenthesis),
        );

        let op = |c| {
            select! {
                TokenKind::Symbol(s) if c == s => Identifier { name: s.to_string() },
            }
            .map(Ast::Identifier)
        };

        let atom = literal.or(ident).or(grouping).labelled("atom");

        let infix_fold = |left: Ast, op: Ast, right: Ast| {
            let first_op = Ast::FunctionCall(FunctionCall {
                function: Box::new(op),
                argument: Box::new(left),
            });
            Ast::FunctionCall(FunctionCall {
                function: Box::new(first_op),
                argument: Box::new(right),
            })
        };

        atom.clone().pratt((
            prefix(3, atom, |op: Ast, o: Ast| {
                Ast::FunctionCall(FunctionCall {
                    function: Box::new(op),
                    argument: Box::new(o),
                })
            }),
            infix(left(2), symbol, infix_fold),
            infix(left(1), op("+"), infix_fold),
            infix(left(1), op("-"), infix_fold),
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chumsky::error;
    use insta::assert_debug_snapshot;
    use rstest::rstest;
    use tracing_test::traced_test;
    type TestExtra = extra::Err<error::Cheap>;

    #[rstest]
    #[traced_test]
    fn test_expression<'src>(
        #[values(
            ("empty", &[][..]),
            ("ident", &[TokenKind::Ident("foo")][..]),
            ("int", &[TokenKind::Number(1.into())][..]),
            ("app", &[
                TokenKind::Ident("foo"),
                TokenKind::Number(1.into())
            ][..]),
            ("app_multiple", &[
                TokenKind::Ident("foo"),
                TokenKind::Ident("foo"),
                TokenKind::Ident("foo"),
                TokenKind::Ident("foo"),
            ][..]),
            ("grouping", &[
                TokenKind::LeftParenthesis,
                TokenKind::Ident("foo"),
                TokenKind::RightParenthesis,
            ][..]),
            ("bad-grouping", &[
                TokenKind::LeftParenthesis,
                TokenKind::Ident("foo"),
            ][..]),
            ("grouping_app", &[
                TokenKind::Ident("foo"),
                TokenKind::LeftParenthesis,
                TokenKind::Ident("bar"),
                TokenKind::Ident("baz"),
                TokenKind::RightParenthesis,
            ][..]),
            ("grouping_app_alt", &[
                TokenKind::LeftParenthesis,
                TokenKind::Ident("foo"),
                TokenKind::Ident("bar"),
                TokenKind::RightParenthesis,
                TokenKind::Ident("baz"),
            ][..]),
            ("infix", &[
                TokenKind::Number(1.into()),
                TokenKind::Symbol("+"),
                TokenKind::Number(1.into()),
            ][..]),
            ("infix_grouping", &[
                TokenKind::LeftParenthesis,
                TokenKind::Ident("a"),
                TokenKind::Symbol("+"),
                TokenKind::Ident("c"),
                TokenKind::RightParenthesis,
                TokenKind::Symbol("+"),
                TokenKind::LeftParenthesis,
                TokenKind::Ident("a"),
                TokenKind::Symbol("+"),
                TokenKind::Ident("c"),
                TokenKind::RightParenthesis,
            ][..]),
            ("assoc", &[
                TokenKind::Ident("a"),
                TokenKind::Symbol("+"),
                TokenKind::Ident("b"),
                TokenKind::Symbol("*"),
                TokenKind::Ident("c"),
            ][..])
        )]
        input: (&str, &[TokenKind<'src>]),
    ) {
        let p = expression_parser::<TestExtra>();

        assert_debug_snapshot!(input.0, p.parse(input.1));
    }
}
