use chumsky::prelude::*;
use insta::assert_debug_snapshot;
use tracing::debug;
use tracing_test::traced_test;

// #[test]
// #[traced_test]
// fn unicode_ident() {
//     // use super::lexer::;

//     assert_debug_snapshot!(unicode_ident().padded().parse(" + "));
//     assert_debug_snapshot!(unicode_ident().padded().parse(" abc "));
//     assert_debug_snapshot!(unicode_ident().parse(" "));
//     assert_debug_snapshot!(unicode_ident().padded().parse(" ( ) "));
// }

#[test]
#[traced_test]
fn lexer() {
    use super::*;

    assert_debug_snapshot!(lexer::lexer().parse(r#"1 + 2 = 1 ("hello" = hello)"#));
    assert_debug_snapshot!(lexer::lexer().parse(r#"== !="#));
}

#[test]
#[traced_test]
fn parser() {
    use super::*;

    let tokens = lexer::lexer().parse(" f 1 + 1 ").into_result().unwrap();
    debug!(?tokens);
    // assert_debug_snapshot!(parser().parse(&tokens));
    let result = parser().parse(&tokens);
    debug!("{:#?}", result);
    // todo!();

    let input = vec![
        Token::Number(1.into()),
        Token::Ident("+"),
        Token::Number(2.into()),
    ];
    assert_debug_snapshot!("1 + 2", parser().parse(&input));

    let input = vec![
        Token::Number(1.into()),
        Token::Ident("+"),
        Token::LeftParenthesis,
        Token::Number(2.into()),
        Token::Ident("^^"),
        Token::Number(3.into()),
        Token::RightParenthesis,
    ];
    assert_debug_snapshot!("1 + (2 ^^ 3)", parser().parse(&input));

    let input = vec![
        Token::Ident("map"),
        Token::LeftParenthesis,
        Token::Ident("x"),
        Token::Ident("y"),
        Token::RightParenthesis,
        Token::Ident("bar"),
    ];
    assert_debug_snapshot!("map (x y) bar", parser().parse(&input));
}
