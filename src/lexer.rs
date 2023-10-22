use std::ops::Not;

use chumsky::input::StrInput;
use chumsky::text::Char;

use crate::Int;
use chumsky::Parser;
use chumsky::{input::ValueInput, prelude::*};

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

#[cfg(test)]
mod tests {
    use super::*;
    use insta::assert_debug_snapshot;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_unicode_ident() {
        assert_debug_snapshot!(unicode_ident().padded().parse(" + "));
        assert_debug_snapshot!(unicode_ident().padded().parse(" abc "));
        assert_debug_snapshot!(unicode_ident().parse(" "));
        assert_debug_snapshot!(unicode_ident().padded().parse(" ( ) "));
    }

    #[test]
    #[traced_test]
    fn test_lexer() {
        assert_debug_snapshot!(lexer().parse(r#"1 + 2 = 1 ("hello" = hello)"#));
        assert_debug_snapshot!(lexer().parse(r#"== !="#));
    }

    // #[test]
    // #[traced_test]
    // fn parser() {
    //     use super::*;

    //     let tokens = lexer::lexer().parse(" f 1 + 1 ").into_result().unwrap();
    //     debug!(?tokens);
    //     // assert_debug_snapshot!(parser().parse(&tokens));
    //     let result = parser().parse(&tokens);
    //     debug!("{:#?}", result);
    //     // todo!();

    //     let input = vec![
    //         Token::Number(1.into()),
    //         Token::Ident("+"),
    //         Token::Number(2.into()),
    //     ];
    //     assert_debug_snapshot!("1 + 2", parser().parse(&input));

    //     let input = vec![
    //         Token::Number(1.into()),
    //         Token::Ident("+"),
    //         Token::LeftParenthesis,
    //         Token::Number(2.into()),
    //         Token::Ident("^^"),
    //         Token::Number(3.into()),
    //         Token::RightParenthesis,
    //     ];
    //     assert_debug_snapshot!("1 + (2 ^^ 3)", parser().parse(&input));

    //     let input = vec![
    //         Token::Ident("map"),
    //         Token::LeftParenthesis,
    //         Token::Ident("x"),
    //         Token::Ident("y"),
    //         Token::RightParenthesis,
    //         Token::Ident("bar"),
    //     ];
    //     assert_debug_snapshot!("map (x y) bar", parser().parse(&input));
    // }
}
