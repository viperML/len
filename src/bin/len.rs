#[cfg(empty)]
fn main() {
    let mut stdout = io::stdout();
    let stdin = io::stdin();

    println!("Welcome to the len repl");

    loop {
        print!("len> ");
        stdout.flush().unwrap();

        let mut buf = String::new();
        let exit = stdin.read_line(&mut buf);

        match exit {
            Ok(0) => {
                println!("\nGoodbye");
                return;
            }
            Ok(_) => {}
            err @ Err(_) => {
                err.unwrap();
            }
        }
        println!("buf:  {:?}", buf);

        let input = buf.trim();
        println!("{:?}", input);

        let tokens = len::lexer::lexer().parse(input);

        if !tokens.has_errors() {
            let tokens = tokens.output().unwrap();
            let ast = ast::expression_parser::<extra::Err<Rich<_>>>().parse(tokens);

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

fn main() {}
