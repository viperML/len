use chumsky::{
    combinator::To,
    input::{Stream, ValueInput},
    prelude::*,
};
use tracing::{debug, info};
use tracing_test::traced_test;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
enum Token<'src> {
    Number(i32),
    Bind,
    String(&'src str),
    Ident(&'src str),
}

type Span = SimpleSpan;
type Spanned<T> = (T, Span);

fn lexer<'src>() -> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>> {
    let number = text::int(10).map(|s: &str| Token::Number(s.parse().unwrap()));

    let string = any()
        .filter(|c| *c != '"')
        .repeated()
        .map_slice(Token::String)
        .delimited_by(just('"'), just('"'));

    let bind = just('=').map(|_| Token::Bind);

    let ident = text::ident().map_slice(|s| Token::Ident(s));

    bind.or(number)
        .or(string)
        .or(ident)
        .map_with_span(|token, span| (token, span))
        .padded()
        .repeated()
        .collect()
}

#[derive(Debug)]
enum Expression<'src> {
    Ident(&'src str),
    String(&'src str),
    Number(i32),
    List(Vec<Self>),
}

mod test {
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn lexer() {
        use super::*;
        let input = r#"1 2 = 1 "hello" = hello "#;

        let output = vec![
            Token::Number(1),
            Token::Number(2),
            Token::Bind,
            Token::Number(1),
            Token::String("hello"),
            Token::Bind,
            Token::Ident("hello"),
        ];

        let output2 = lexer()
            .parse(input)
            .into_result()
            .map(|res| res.into_iter().map(|(v, _)| v).collect());

        assert_eq!(output2, Ok(output));
    }

    #[test]
    #[traced_test]
    fn parser() {
        use super::*;

        let input = vec![Token::Ident("Hello"), Token::Number(1)];

        let result = parser().parse(&input);
        debug!(?result);
        todo!();
    }
}

fn parser<'src>() -> impl Parser<'src, &'src [Token<'src>], Expression<'src>> {
    let mut recursive_step = 0;
    recursive(|expr| {
        let atom = select! {
            Token::Number(x) => Expression::Number(x),
            Token::String(x) => Expression::String(x),
        };

        let list = expr.repeated().collect().map(Expression::List);
        let res = atom.or(list);

        recursive_step += 1;
        if recursive_step > 20 {
            panic!();
        }

        res
    })
}
