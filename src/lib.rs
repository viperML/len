use chumsky::prelude::*;
use tracing::{debug, info};
use tracing_test::traced_test;

#[derive(Debug, PartialEq, PartialOrd)]
enum Token {
    Number(i32),
    Bind,
    String(String),
    // Ident(String),
}

type Span = SimpleSpan;
type Spanned<T> = (T, Span);

fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Spanned<Token>>> {
    let number = text::int(10).map(|s: &str| Token::Number(s.parse().unwrap()));

    let string = just('"')
        .ignore_then(any().filter(|c| *c != '"').repeated())
        .then_ignore(just('"'))
        .map_slice(|s| Token::String(String::from(s)));

    let bind = just('=').map(|_| Token::Bind);

    bind.or(number)
        .or(string)
        .map_with_span(|token, span| (token, span))
        .padded()
        .repeated()
        .collect()
}

mod test {
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn lexer() {
        use super::*;
        let input = r#"1 2 = 1 "hello" = "#;
        let output = vec![
            Token::Number(1),
            Token::Number(2),
            Token::Bind,
            Token::Number(1),
            Token::String(String::from("hello")),
            Token::Bind,
        ];

        let output2 = lexer()
            .parse(input)
            .into_result()
            .map(|res| res.into_iter().map(|(v, _)| v).collect());

        assert_eq!(output2, Ok(output));
    }
}
