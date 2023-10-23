use chumsky::extra::ParserExtra;
use chumsky::input::StrInput;

use chumsky::text::Char;

use crate::Int;
use chumsky::prelude::*;
use chumsky::Parser;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TokenKind<'src> {
    Number(Int),
    Bind,
    String(&'src str),
    Ident(&'src str),
    Symbol(&'src str),
    RightParenthesis,
    LeftParenthesis,
}

#[derive(Debug)]
pub struct Token<'src> {
    pub kind: TokenKind<'src>,
    pub span: SimpleSpan,
}

fn is_reserved_char(c: &char) -> bool {
    r#"(),;[]`{}_:"'"#.chars().any(|reserved| reserved == c.to_char())
}

#[must_use]
fn symbol<'a, I: StrInput<'a, C>, C: Char, E: ParserExtra<'a, I>>(
) -> impl Parser<'a, I, &'a C::Str, E> + Copy + Clone {
    let f = |c: &C| {
        let c = c.to_char();
        !c.is_whitespace() && !c.is_alphanumeric() && !is_reserved_char(&c)
    };

    any().filter(f).then(any().filter(f).repeated()).to_slice()
}

pub type LexerI<'a> = &'a str;
pub type LexerO<'a> = Vec<Token<'a>>;

#[must_use]
pub fn lexer<'s, E: ParserExtra<'s, LexerI<'s>>>() -> impl Parser<'s, LexerI<'s>, LexerO<'s>, E> {
    let number = text::int(10).map(|s: &str| TokenKind::Number(s.parse().unwrap()));

    let string = any()
        .filter(|c| *c != '"')
        .repeated()
        .to_slice()
        .map(TokenKind::String)
        .delimited_by(just('"'), just('"'));

    let right_parens = just(')').map(|_| TokenKind::RightParenthesis);
    let left_parens = just('(').map(|_| TokenKind::LeftParenthesis);

    let symbol = symbol().to_slice().map(TokenKind::Symbol);

    let ident = chumsky::text::unicode::ident().map(TokenKind::Ident);

    number
        .or(right_parens)
        .or(left_parens)
        .or(symbol)
        .or(string)
        .or(ident)
        .padded()
        .map_with(|t: TokenKind, e| Token {
            kind: t,
            span: e.span(),
        })
        .repeated()
        .collect()
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
    fn test_symbol(
        #[values(
            //-
            " + ",
            " + ",
            " +2",
            " ++ ",
            " // ",
            " - - - -- - - ",
            " () , . ",
        )]
        input: &str,
    ) {
        let p = symbol::<_, _, TestExtra>()
            .padded()
            .repeated()
            .collect::<Vec<_>>();

        assert_debug_snapshot!(input, (input, p.parse(input)));
    }

    #[rstest]
    #[traced_test]
    fn test_lexer(
        #[values(
            ("empty", ""),
            ("int", "1 23 313 1"),
            ("symbols", "+ == != -"),
            ("symbols_split", r#"+3+1--3//3&(s++)"#),
            ("string", r#" "foo" "bar" "#),
            ("bad s", r#" "foo "#),
            ("parens", r#"(12 +23)()("foo")(1+1)"#),
            ("ident", "foo bar foo_bar foo-bar (foo+1)"),
        )]
        input: (&str, &str),
    ) {
        let p = lexer::<TestExtra>().padded();

        assert_debug_snapshot!(input.0, (input.0, input.1, p.parse(input.1)));
    }
}
