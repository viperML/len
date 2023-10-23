use crate::{
    ast,
    eval::{Object, Scope},
    lexer::{lexer, Token},
};
use chumsky::{error::Rich, extra, Parser};
use tracing::{debug, info};
use std::io::{self, Write};

pub fn complete(input: &str) {
    let tokens = lexer::<extra::Err<Rich<_>>>().parse(input);
    debug!("tokens={:#?}", tokens);

    if !tokens.has_errors() {
        let tokens = tokens
            .into_output()
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        let ast = ast::expression_parser::<extra::Err<Rich<_>>>().parse(&tokens);

        if !ast.has_errors() {
            let ast = ast.output().unwrap();
            debug!("ast={:#?}", ast);

            let scope = Scope::std();
            let res = crate::eval::eval(ast.clone(), &scope);
            info!("{:#?}", res);
        } else {
            println!("{:#?}", ast);
        }
    } else {
        println!("{:#?}", tokens);
    }
}
