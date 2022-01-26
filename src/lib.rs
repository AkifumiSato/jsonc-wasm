extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

pub mod lexer;
mod node;
pub mod parser;
mod token;
mod utils;

#[wasm_bindgen]
pub fn parse(data: String) -> String {
    data
}
