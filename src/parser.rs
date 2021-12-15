extern crate wasm_bindgen;

use std::iter::{Enumerate, Peekable};
use std::str::Chars;

use wasm_bindgen::prelude::*;

use crate::token::{LexerError, Location, Token};
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
                }
                'f' => {
                    let token = self.parse_bool_token(false, index)?;
                    tokens.push(token);
                }
                'n' => {
                    let token = self.parse_null_token(index)?;
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
                '\\' => {
                    let (_, c2) = self
                        .input
                        .next()
                        .ok_or(LexerError::not_exist_terminal_symbol())?;
                    match c2 {
                        'u' => {
                            let hex = self.take_chars_with(4);
                            if hex.len() != 4 && hex.parse::<f64>().is_ok() {
                                return Err(LexerError::not_exist_terminal_symbol());
                            }

                            times += 6;
                            value.push_str(&format!("\\u{}", hex));
                        }
                        _ => todo!("escape文字列の処理"),
                    }
                }
                _ => {
                    value.push(c);
                    times += 1;
                }
            }
        }
        Err(LexerError::not_exist_terminal_symbol())
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
        Err(LexerError::not_exist_terminal_symbol())
    }

    fn parse_bool_token(&mut self, expect_bool: bool, index: usize) -> Result<Token, LexerError> {
        let s: String;
        let (s, end) = if expect_bool {
            // すでに最初の`t`は消費されている前提なので残り文字を精査
            s = "t".to_string() + &self.take_chars_with(3);
            (s, index + 3)
        } else {
            // すでに最初の`f`は消費されている前提なので残り文字を精査
            s = "f".to_string() + &self.take_chars_with(4);
            (s, index + 4)
        };
        let location = Location(index, end);
        match &s as &str {
            "true" => Ok(Token::boolean(true, location)),
            "false" => Ok(Token::boolean(false, location)),
            _ => Err(LexerError::not_exist_terminal_symbol()),
        }
    }

    fn parse_null_token(&mut self, index: usize) -> Result<Token, LexerError> {
        // `null`かどうか文字を取得
        let s = "n".to_string() + &self.take_chars_with(3);
        let location = Location(index, index + 3);
        if s == "null" {
            Ok(Token::null(location))
        } else {
            Err(LexerError::invalid_chars(s.to_string(), Some(location)))
        }
    }

    fn take_chars_with(&mut self, times: i32) -> String {
        let chars = (0..times)
            .filter_map(|_| self.input.next().map(|(_index, c)| c))
            .collect::<String>();
        chars
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
    \"flag\": false,\
    \"attr\": null
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
            Token::comma(Location(44, 45)),
            Token::string("attr", Location(46, 50)),
            Token::colon(Location(51, 52)),
            Token::null(Location(53, 56)),
            Token::close_brace(Location(58, 59)),
        ];
        for (index, expect) in expected.iter().enumerate() {
            assert_eq!(expect, &result[index], "tokenの{}番目が想定外です。", index,);
        }
        assert_eq!(16, result.len(), "token配列長が想定外です。");
    }

    #[test]
    fn parse_string_token_should_return_token() {
        let mut lexer = Lexer::new("name123\"");
        let token = lexer
            .parse_string_token()
            .expect("[parse_string_token_should_return_token]\"name\"のparseに失敗しました。");
        assert_eq!(Token::string("name123", Location(0, 7)), token);

        let mut lexer = Lexer::new("あいうえお\"");
        let token = lexer.parse_string_token().expect(
            "[parse_string_token_should_return_token]\"あいうえお\"のparseに失敗しました。",
        );
        assert_eq!(Token::string("あいうえお", Location(0, 5)), token);

        let mut lexer = Lexer::new(r#"\u3042\u3044\u3046abc""#);
        let token = lexer
            .parse_string_token()
            .expect("[parse_string_token_should_return_token]\"あいうabc\"のparseに失敗しました。");
        assert_eq!(
            Token::string("\\u3042\\u3044\\u3046abc", Location(0, 21)),
            token
        );

        let mut lexer = Lexer::new(r#"\ud83d\ude00\ud83d\udc4d""#);
        let token = lexer
            .parse_string_token()
            .expect("[parse_string_token_should_return_token]\"😀👍\"のparseに失敗しました。");
        assert_eq!(
            Token::string("\\ud83d\\ude00\\ud83d\\udc4d", Location(0, 24)),
            token
        );

        let mut lexer = Lexer::new("😀👍\"");
        let token = lexer
            .parse_string_token()
            .expect("[parse_string_token_should_return_token]\"😀👍\"のparseに失敗しました。");
        assert_eq!(Token::string("😀👍", Location(0, 2)), token);
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
            panic!("[parse_bool_token]がErrを返しました。");
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

    #[test]
    fn parse_null_token_should_return_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":null,");
        // 最初の`f`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        if let Ok(token) = lexer.parse_null_token(index) {
            assert_eq!(Token::null(Location(1, 4)), token);
        } else {
            panic!("[parse_null_token]がErrを返しました。");
        };
    }

    #[test]
    fn parse_null_token_should_err() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":nu");
        // 最初の`n`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        assert!(lexer.parse_null_token(index).is_err());
    }
}
