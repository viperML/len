use eyre::Result;
use len::{
    ast, chumsky,
    eval::RawScope,
    lexer::{lexer, Token},
};
use std::io::{self, Write};
use tracing::info;
use tracing_subscriber::{prelude::*, EnvFilter};

fn main() -> Result<()> {
    color_eyre::install()?;

    let layer_fmt = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .without_time()
        .with_line_number(true)
        .compact();

    let layer_error = tracing_error::ErrorLayer::default();

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(layer_error)
        .with(layer_fmt)
        .init();

    let mut stdout = io::stdout();
    let stdin = io::stdin();

    info!("Welcome to the len repl");

    let mut scope = None;

    loop {
        print!("len> ");
        stdout.flush().unwrap();

        let mut buf = String::new();
        let exit = stdin.read_line(&mut buf);
        if !buf.contains("\n") {
            println!();
        }

        match exit {
            Ok(0) => {
                info!("Goodbye");
                return Ok(());
            }
            err @ Err(_) => {
                err.unwrap();
            }
            Ok(_) => {
                scope = Some(len::complete::complete(&buf, scope));
            }
        }
    }
}
