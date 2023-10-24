use crate::{
    ast,
    eval::{Object, Scope},
    lexer::{lexer, Token},
};
use chumsky::{error::Rich, extra, Parser};
use std::io::{self, Write};
use tracing::{debug, info};

pub fn complete(input: &str, scope: Option<Scope>) -> Scope {
    let tokens = lexer::<extra::Err<Rich<_>>>().parse(input);
    debug!("tokens={:#?}", tokens);

    let mut scope = match scope {
        None => Scope::std(),
        Some(s) => s,
    };

    if !tokens.has_errors() {
        let tokens = tokens
            .into_output()
            .unwrap()
            .into_iter()
            .map(|t| t.kind)
            .collect::<Vec<_>>();

        let ast = ast::ast_parser::<extra::Err<Rich<_>>>().parse(&tokens);

        // let scope = Scope::std();
        // let mut scope = None;

        if !ast.has_errors() {
            let ast = ast.into_output().unwrap();
            debug!("ast={:#?}", ast);

            let res = crate::eval::eval(ast, &scope);
            if let Some(new_scope) = res {
                scope = new_scope;
            }
        } else {
            println!("{:#?}", ast);
        }
    } else {
        println!("{:#?}", tokens);
    }

    return scope;
}
