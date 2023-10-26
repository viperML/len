mod utils;

use chumsky::extra;
use len::chumsky::{self, prelude::Simple, Parser};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, len-web!");
}

#[wasm_bindgen]
pub fn inc(input: i32) -> i32 {
    input + 1
}

#[wasm_bindgen]
pub fn lexer(input: &str) -> String {
    let p = len::lexer::lexer::<extra::Err<Simple<_>>>();

    let res = p.parse(input);

    format!("{:#?}", res)
}
