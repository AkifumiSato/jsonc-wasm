extern crate wasm_bindgen;

use std::iter::{Enumerate, Peekable};
use std::str::Chars;

use wasm_bindgen::prelude::*;

use crate::token::{LexerError, LexerErrorKind, Location, Token};

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

        while let Some((index, char)) = self.input.next() {
            match char {
                '"' => {
                    return Ok(Token::string(&value, Location(index - times, index)));
                }
                _ => {
                    value.push(char);
                    times += 1;
                } // todo escape文字列の処理
            }
        }
        let current = self.input.peek();
        if current.is_none() {
            Err(LexerError::new(
                LexerErrorKind::NotExistTerminalSymbol,
                None,
            ))
        } else {
            let current = current.unwrap();
            Err(LexerError::new(
                LexerErrorKind::InvalidChars(value),
                Some(Location(current.0.clone(), times)),
            ))
        }
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
    \"name\": \"sato\"
}",
        );
        let result = lexer.tokenize().expect("lexerは配列を返します。");
        let expected: Vec<Token> = vec![
            Token::open_brace(Location(0, 1)),
            Token::string("name", Location(8, 12)),
            Token::colon(Location(13, 14)),
            Token::string("sato", Location(16, 20)),
            Token::close_brace(Location(22, 23)),
        ];
        assert_eq!(5, result.len(), "token配列の長さが想定外です。");
        assert_eq!(
            TokenKind::OpenBrace,
            result[0].value,
            "tokenの0番目が想定外です。"
        );
        assert_eq!(
            TokenKind::StringValue("name".to_string()),
            result[1].value,
            "tokenの1番目が想定外です。"
        );
        assert_eq!(
            TokenKind::Colon,
            result[2].value,
            "tokenの2番目が想定外です。"
        );
        assert_eq!(
            TokenKind::StringValue("sato".to_string()),
            result[3].value,
            "tokenの3番目が想定外です。"
        );
        assert_eq!(
            TokenKind::CloseBrace,
            result[4].value,
            "tokenの4番目が想定外です。"
        );
        assert_eq!(expected, result, "token配列のequalsが想定外です。");
    }
}
