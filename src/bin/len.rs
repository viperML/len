use std::io::{Write, self};

use chumsky::{Parser, ParseResult};
use len::{parser, eval::eval};


fn main() {
    let mut stdout = io::stdout();
    let mut stdin = io::stdin();

    println!("Welcome to the len repl");

    loop {
        print!("len> ");
        stdout.flush().unwrap();

        let mut buf = String::new();
        stdin.read_line(&mut buf).unwrap();

        let input = buf.trim();
        println!("{}", input);

        let tokens = len::lexer().parse(&input);

        if ! tokens.has_errors() {
            let tokens = tokens.output().unwrap();
            let ast = parser().parse(&tokens);

            if !ast.has_errors() {
                let ast = ast.output().unwrap();
                println!("{:#?}", ast);




            } else {
                println!("{:#?}", ast);
            }
        } else {
            println!("{:#?}", tokens);
        }
    }
}