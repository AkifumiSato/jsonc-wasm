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

        while let Some((index, c)) = self.input.next() {
            match c {
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
                't' => {
                    let token = self.parse_bool_token(true, index)?;
                    tokens.push(token);
                },
                'f' => {
                    let token = self.parse_bool_token(false, index)?;
                    tokens.push(token);
                },
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

    fn parse_bool_token(&mut self, expect_bool: bool, index: usize) -> Result<Token, LexerError> {
        let mut s = String::new();
        let (s, end) = if expect_bool {
            // すでに最初の`t`は消費されている前提なので残り文字を精査
            s = "t".to_string() + &(0..3).filter_map(|_| self.input.next().map(|(index, c)| c)).collect::<String>();
            (s, index + 3)
        } else {
            // すでに最初の`f`は消費されている前提なので残り文字を精査
            s = "f".to_string() + &(0..4).filter_map(|_| self.input.next().map(|(index, c)| c)).collect::<String>();
            (s, index + 4)
        };
        let location = Location(index, end);
        let s: &str = &s;
        match s {
            "true" => Ok(Token::boolean(true, location)),
            "false" => Ok(Token::boolean(false, location)),
            _ => Err(LexerError::new(
                LexerErrorKind::InvalidChars(s.to_string()),
                Some(location),
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

    #[test]
    fn greet_name() {
        assert_eq!(greet("world"), "Hello, world!!!");
    }

    #[test]
    fn lexer_should_success_parse() {
        let mut lexer = Lexer::new(
            "{
    \"name\": \"sato\",\
    \"age\": 20,\
    \"flag\": false\
}",
        );
        let result = lexer.tokenize().expect("lexerは配列を返します。");
        let expected = [
            Token::open_brace(Location(0, 1)),
            Token::string("name", Location(7, 11)),
            Token::colon(Location(12, 13)),
            Token::string("sato", Location(15, 19)),
            Token::comma(Location(20, 21)),
            Token::string("age", Location(22, 25)),
            Token::colon(Location(26, 27)),
            Token::number("20", Location(29, 30)),
            Token::string("flag", Location(32, 36)),
            Token::colon(Location(37, 38)),
            Token::boolean(false, Location(39, 43)),
            Token::close_brace(Location(44, 45)),
        ];
        for (index, expect) in expected.iter().enumerate() {
            assert_eq!(
                expect,
                &result[index],
                "tokenの{}番目が想定外です。",
                index,
            );
        }
        assert_eq!(12, result.len(), "token配列長が想定外です。");
    }

    #[test]
    fn parse_string_token_should_return_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new("{name\"");
        // `{`の分のみ進める
        lexer.input.next();
        if let Ok(token) = lexer.parse_string_token() {
            assert_eq!(Token::string("name", Location(1, 5)), token);
        } else {
            panic!("[parse_string_token]がErrを返しました。");
        };
    }

    #[test]
    fn parse_string_token_should_err() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new("name");
        assert!(lexer.parse_string_token().is_err());
    }

    #[test]
    fn parse_number_token_should_return_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":100,");
        // 最初の`"`まで進める
        lexer.input.next();
        let (_, first) = lexer.input.next().unwrap();
        if let Ok(token) = lexer.parse_number_token(first) {
            assert_eq!(Token::number("100", Location(2, 4)), token);
        } else {
            panic!("[parse_string_token]がErrを返しました。");
        };
    }

    #[test]
    fn parse_number_token_should_err() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":100");
        // 最初の`"`まで進める
        lexer.input.next();
        let (_, first) = lexer.input.next().unwrap();
        assert!(lexer.parse_number_token(first).is_err());
    }

    #[test]
    fn parse_bool_token_should_return_true_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":true,");
        // 最初の`t`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        if let Ok(token) = lexer.parse_bool_token(true, index) {
            assert_eq!(Token::boolean(true, Location(1, 4)), token);
        } else {
            panic!("[parse_string_token]がErrを返しました。");
        };
    }

    #[test]
    fn parse_bool_token_should_err_with_true() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":tru");
        // 最初の`t`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        assert!(lexer.parse_bool_token(true, index).is_err());
    }

    #[test]
    fn parse_bool_token_should_return_false_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":false,");
        // 最初の`f`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        if let Ok(token) = lexer.parse_bool_token(false, index) {
            assert_eq!(Token::boolean(false, Location(1, 5)), token);
        } else {
            panic!("[parse_string_token]がErrを返しました。");
        };
    }

    #[test]
    fn parse_bool_token_should_err_with_false() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":fal");
        // 最初の`f`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        assert!(lexer.parse_bool_token(false, index).is_err());
    }
}
