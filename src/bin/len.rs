use std::io::{self, Write};

use chumsky::Parser;
use len::{eval::Scope, parser};

fn main() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    println!("Welcome to the len repl");

    loop {
        print!("len> ");
        stdout.flush().unwrap();

        let mut buf = String::new();
        stdin.read_line(&mut buf).unwrap();

        let input = buf.trim();
        println!("{}", input);

        let tokens = len::lexer().parse(input);

        if !tokens.has_errors() {
            let tokens = tokens.output().unwrap();
            let ast = parser().parse(tokens);

            if !ast.has_errors() {
                let ast = ast.output().unwrap();
                // println!("{:#?}", ast);

                let scope = Scope::std();
                let res = len::eval::eval(ast.clone(), &scope);
                println!("{:#?}", res);
            } else {
                println!("{:#?}", ast);
            }
        } else {
            println!("{:#?}", tokens);
        }
    }
}
