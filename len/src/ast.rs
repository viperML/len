use crate::lexer::{Token, TokenKind};
use crate::Int;
use chumsky::extra::ParserExtra;
use chumsky::pratt::{infix, left, postfix, prefix};
use chumsky::prelude::*;
use chumsky::Parser;
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Not;
use tracing::span::Id;
use tracing::{debug, trace};
use tracing_test::traced_test;

#[derive(Debug, Clone)]
pub enum Ast {
    Expr(Expr),
    Binding { lhs: Identifier, rhs: Expr },
    Todo,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    FunctionCall(FunctionCall),
    Identifier(Identifier),
    Product(HashMap<String, Expr>),
    Lambda(Lambda),
    Todo,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Integer(Int),
    String(String),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub(crate) function: Box<Expr>,
    pub(crate) argument: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub(crate) name: String,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub from: Identifier,
    pub to: Box<Expr>,
}

#[derive(Debug, PartialEq)]
pub struct Spanned2<T>(T, SimpleSpan<usize>);

pub fn expression_parser<'s, E: ParserExtra<'s, &'s [TokenKind<'s>]>>(
) -> impl Parser<'s, &'s [TokenKind<'s>], Expr, extra::Err<Rich<'s, TokenKind<'s>>>> + Clone {
    recursive(|expr| {
        let literal = select! {
            TokenKind::Ident("true") => Literal::Boolean(true),
            TokenKind::Ident("false") => Literal::Boolean(false),
            TokenKind::Number(x) => Literal::Integer(x),
            TokenKind::String(x) => Literal::String(x.to_string()),
        }
        .map(Expr::Literal);

        let ident = select! {
            TokenKind::Ident(s) => Identifier { name: s.to_string() },
        }
        .map(Expr::Identifier);

        let grouping = expr.clone().delimited_by(
            just(TokenKind::LeftParenthesis),
            just(TokenKind::RightParenthesis),
        );

        let struct_elem = select! {
            TokenKind::Ident(s) => s.to_string(),
        }
        .then_ignore(just(TokenKind::Colon))
        .then(expr.clone());

        let r#struct = struct_elem
            .separated_by(just(TokenKind::Comma))
            .collect::<HashMap<_, _>>()
            .map(Expr::Product)
            .delimited_by(just(TokenKind::LeftCurly), just(TokenKind::RightCurly))
            .labelled("struct");

        let lambda = select! {
            TokenKind::Ident(s) => Identifier { name: s.to_string() },
        }
        .then_ignore(just(TokenKind::Arrow))
        .then(expr)
        .map(|(from, to)| Lambda {
            from,
            to: Box::new(to),
        })
        .map(Expr::Lambda)
        .labelled("lambda");

        let atom = choice((literal, r#struct, lambda, ident, grouping)).labelled("atom");

        // Left associative application
        let application = atom.clone().foldl(atom.repeated(), |op, o| {
            Expr::FunctionCall(FunctionCall {
                function: Box::new(op),
                argument: Box::new(o),
            })
        });

        let any_symbol = select! {
            TokenKind::Symbol(s) => Identifier { name: s.to_string() },
        }
        .map(Expr::Identifier);

        let mk_symbol = |c| {
            select! {
                TokenKind::Symbol(s) if c == s => Identifier { name: s.to_string() },
            }
            .map(Expr::Identifier)
        };

        let infix_fold = |left: Expr, op: Expr, right: Expr| {
            trace!("Creating infix");
            let first_op = Expr::FunctionCall(FunctionCall {
                function: Box::new(op),
                argument: Box::new(left),
            });
            Expr::FunctionCall(FunctionCall {
                function: Box::new(first_op),
                argument: Box::new(right),
            })
        };

        application.pratt((
            infix(left(2), any_symbol, infix_fold),
            infix(left(1), mk_symbol("+"), infix_fold),
            infix(left(1), mk_symbol("-"), infix_fold),
            infix(left(0), mk_symbol("$"), infix_fold),
        ))
    })
}

pub fn ast_parser<'s, E: ParserExtra<'s, &'s [TokenKind<'s>]>>(
) -> impl Parser<'s, &'s [TokenKind<'s>], Ast, extra::Err<Rich<'s, TokenKind<'s>>>> {
    let ep = expression_parser::<E>();

    let binding = select! {
        TokenKind::Ident(s) => Identifier { name: s.to_string() },
    }
    .then_ignore(just(TokenKind::Bind))
    .then(ep.clone())
    .map(|(lhs, rhs)| Ast::Binding { lhs, rhs });

    choice((binding, ep.map(Ast::Expr)))
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
            ("infix_assoc", &[
                TokenKind::Ident("a"),
                TokenKind::Symbol("+"),
                TokenKind::Ident("b"),
                TokenKind::Symbol("*"),
                TokenKind::Ident("c"),
            ][..]),
            ("struct_simple", &[
                TokenKind::LeftCurly,
                TokenKind::Ident("a"),
                TokenKind::Colon,
                TokenKind::Ident("b"),
                TokenKind::RightCurly
            ][..]),
        )]
        input: (&str, &[TokenKind<'src>]),
    ) {
        let p = expression_parser::<TestExtra>();

        assert_debug_snapshot!(input.0, p.parse(input.1));
    }

    #[rstest]
    #[traced_test]
    fn test_ast<'src>(
        #[values(
            ("empty", &[][..]),
            ("assign", &[
                TokenKind::Ident("a"),
                TokenKind::Bind,
                TokenKind::Ident("b"),
            ][..]),
        )]
        input: (&str, &[TokenKind<'src>]),
    ) {
        let p = ast_parser::<TestExtra>();

        assert_debug_snapshot!(input.0, p.parse(input.1));
    }
}
