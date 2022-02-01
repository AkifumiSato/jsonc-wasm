extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

mod lexer;
mod node;
mod parser;
mod token;
mod utils;

#[wasm_bindgen(js_name = toJsonString)]
pub fn to_json_string(data: String) -> Result<String, String> {
    let mut lexer = Lexer::new(&data);
    let token = lexer.tokenize().or_else(|e| Err(e.to_string()))?;
    let mut parser = Parser::new(&token);
    let res = parser.parse().or_else(|e| Err(e.to_string()))?;
    Ok(res.to_json_string())
}
