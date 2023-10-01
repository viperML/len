use chumsky::prelude::*;
use insta::assert_debug_snapshot;
use tracing::debug;
use tracing_test::traced_test;

#[test]
#[traced_test]
fn unicode_ident() {
    use super::unicode_ident;

    assert_debug_snapshot!(unicode_ident().padded().parse(" + "));
    assert_debug_snapshot!(unicode_ident().padded().parse(" abc "));
    assert_debug_snapshot!(unicode_ident().parse(" "));
    assert_debug_snapshot!(unicode_ident().padded().parse(" ( ) "));
}

#[test]
#[traced_test]
fn lexer() {
    use super::*;

    assert_debug_snapshot!(lexer().parse(r#"1 + 2 = 1 ("hello" = hello)"#));
    assert_debug_snapshot!(lexer().parse(r#"== !="#));
}

#[test]
#[traced_test]
fn parser() {
    use super::*;

    let tokens = lexer().parse(" f 1 + 1 ").into_result().unwrap();
    debug!(?tokens);
    // assert_debug_snapshot!(parser().parse(&tokens));
    let result = parser().parse(&tokens);
    debug!("{:#?}", result);
    todo!();
}
