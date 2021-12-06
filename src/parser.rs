extern crate wasm_bindgen;

use std::iter::{Enumerate, Peekable};
use std::str::Chars;

use wasm_bindgen::prelude::*;

use crate::token::{LexerError, LexerErrorKind, Location, Token};
use crate::utils::is_number_token_char;

struct Lexer<'a> {
    input: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().enumerate().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some((index, char)) = self.input.next() {
            match char {
                '{' => tokens.push(Token::open_brace(Location(index, index + 1))),
                '}' => tokens.push(Token::close_brace(Location(index, index + 1))),
                '[' => tokens.push(Token::open_bracket(Location(index, index + 1))),
                ']' => tokens.push(Token::close_bracket(Location(index, index + 1))),
                '"' => {
                    let token = self.parse_string_token()?;
                    tokens.push(token);
                }
                c if is_number_token_char(c) => {
                    let token = self.parse_number_token(c)?;
                    tokens.push(token);
                }
                ':' => tokens.push(Token::colon(Location(index, index + 1))),
                ',' => tokens.push(Token::comma(Location(index, index + 1))),
                _ => (),
            };
        }

        Ok(tokens)
    }

    fn parse_string_token(&mut self) -> Result<Token, LexerError> {
        let mut value = String::new();
        let mut times = 0;

        while let Some((index, c)) = self.input.next() {
            match c {
                '"' => {
                    return Ok(Token::string(&value, Location(index - times, index)));
                }
                _ => {
                    value.push(c);
                    times += 1;
                } // todo escape文字列の処理
            }
        }
        Err(LexerError::new(
            LexerErrorKind::NotExistTerminalSymbol,
            None,
        ))
    }

    fn parse_number_token(&mut self, first: char) -> Result<Token, LexerError> {
        let mut value = String::new();
        let mut times = 0;
        value.push(first);

        while let Some((index, c)) = self.input.next() {
            if is_number_token_char(c) {
                value.push(c);
                times += 1;
            } else {
                let start = index - times;
                return Ok(Token::number(&value, Location(start, index)));
            }
        }
        Err(LexerError::new(
            LexerErrorKind::NotExistTerminalSymbol,
            None,
        ))
    }
}

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    let message = format!("Hello, {}!!!", name);
    message.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenKind;

    #[test]
    fn greet_name() {
        assert_eq!(greet("world"), "Hello, world!!!");
    }

    #[test]
    fn lexer_should_success_parse() {
        let mut lexer = Lexer::new(
            "{'
    \"name\": \"sato\",\
    \"age\": 20
}",
        );
        let result = lexer.tokenize().expect("lexerは配列を返します。");
        assert_eq!(
            Token::open_brace(Location(0, 1)),
            result[0],
            "tokenの0番目が想定外です。"
        );
        assert_eq!(
            Token::string("name", Location(8, 12)),
            result[1],
            "tokenの1番目が想定外です。"
        );
        assert_eq!(
            Token::colon(Location(13, 14)),
            result[2],
            "tokenの2番目が想定外です。"
        );
        assert_eq!(
            Token::string("sato", Location(16, 20)),
            result[3],
            "tokenの3番目が想定外です。"
        );
        assert_eq!(
            Token::comma(Location(21, 22)),
            result[4],
            "tokenの4番目が想定外です。"
        );
        assert_eq!(
            Token::string("age", Location(23, 26)),
            result[5],
            "tokenの5番目が想定外です。"
        );
        assert_eq!(
            Token::colon(Location(27, 28)),
            result[6],
            "tokenの6番目が想定外です。"
        );
        assert_eq!(
            Token::number("20", Location(30, 31)),
            result[7],
            "tokenの7番目が想定外です。"
        );
        assert_eq!(
            Token::close_brace(Location(32, 33)),
            result[8],
            "tokenの8番目が想定外です。"
        );
        assert_eq!(9, result.len(), "token配列長が想定外です。");
    }
}
