extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;
use crate::lexer::Lexer;
use crate::parser::Parser;

mod lexer;
mod node;
mod parser;
mod token;
mod utils;

#[wasm_bindgen]
pub fn parse(data: String) -> String {
    let mut lexer = Lexer::new(&data);
    let token = lexer.tokenize().unwrap(); // todo remove unwrap
    let mut parser = Parser::new(&token);
    let res = parser.parse().unwrap(); // todo remove unwrap
    res.to_json_string()
    // todo ここでJSON.parseをcall
}
