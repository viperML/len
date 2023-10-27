mod utils;

use chumsky::extra;
use len::chumsky::{self, error::Rich, Parser};
use wasm_bindgen::prelude::*;

type Extra<'a, T> = extra::Err<Rich<'a, T>>;

#[wasm_bindgen(getter_with_clone)]
pub struct EvalResult {
    pub lexer: String,
    pub ast: String,
}

#[wasm_bindgen]
pub fn main(input: String) -> EvalResult {
    let lexer_res = len::lexer::lexer::<Extra<_>>().parse(&input);

    let lexer_res_str = format!("{:#?}", lexer_res);

    let prev = lexer_res
        .into_output()
        .unwrap_or_default()
        .into_iter()
        .map(|t| t.kind)
        .collect::<Vec<_>>();

    let ast_res = len::ast::ast_parser::<Extra<_>>().parse(&prev);
    let ast_res_str = format!("{:#?}", ast_res);

    EvalResult {
        lexer: lexer_res_str,
        ast: ast_res_str,
    }
}
