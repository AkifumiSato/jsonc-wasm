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
                '{' => tokens.push(Token::open_brace(Location(index, index))),
                '}' => tokens.push(Token::close_brace(Location(index, index))),
                '[' => tokens.push(Token::open_bracket(Location(index, index))),
                ']' => tokens.push(Token::close_bracket(Location(index, index))),
                '"' => {
                    let token = self.scan_string_token()?;
                    tokens.push(token);
                }
                c if is_number_token_char(c) => {
                    let token = self.scan_number_token(c)?;
                    tokens.push(token);
                }
                't' => {
                    let token = self.scan_bool_token(true, index)?;
                    tokens.push(token);
                }
                'f' => {
                    let token = self.scan_bool_token(false, index)?;
                    tokens.push(token);
                }
                'n' => {
                    let token = self.scan_null_token(index)?;
                    tokens.push(token);
                }
                ':' => tokens.push(Token::colon(Location(index, index))),
                ',' => tokens.push(Token::comma(Location(index, index))),
                '/' => {
                    let token = self.scan_comment_token()?;
                    tokens.push(token);
                }
                ' ' => {
                    let token = self.scan_whitespaces()?;
                    tokens.push(token);
                }
                '\n' => tokens.push(Token::break_line(Location(index, index))),
                _ => (),
            };
        }

        Ok(tokens)
    }

    fn scan_string_token(&mut self) -> Result<Token, LexerError> {
        let mut value = String::new();
        let mut length = 1; // 最初の"で1

        while let Some((index, c)) = self.input.next() {
            match c {
                '"' => {
                    return Ok(Token::string(&value, Location(index - length, index)));
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

                            length += 6;
                            value.push_str(&format!("\\u{}", hex));
                        }
                        '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => {
                            length += 2;
                            value.push_str(&format!("\\{}", c2));
                        }
                        _ => {
                            return Err(LexerError::not_escape_string());
                        }
                    }
                }
                _ => {
                    value.push(c);
                    length += 1;
                }
            }
        }
        Err(LexerError::not_exist_terminal_symbol())
    }

    fn scan_number_token(&mut self, first: char) -> Result<Token, LexerError> {
        let mut value = String::new();
        let mut times = 0;
        value.push(first);

        while let Some((index, c)) = self.input.peek() {
            if is_number_token_char(*c) {
                let (_, c) = self.input.next().unwrap();
                value.push(c);
                times += 1;
            } else {
                let start = index - times;
                return Ok(Token::number(&value, Location(start, *index)));
            }
        }
        Err(LexerError::not_exist_terminal_symbol())
    }

    fn scan_bool_token(&mut self, expect_bool: bool, index: usize) -> Result<Token, LexerError> {
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

    fn scan_null_token(&mut self, index: usize) -> Result<Token, LexerError> {
        // `null`かどうか文字を取得
        let s = "n".to_string() + &self.take_chars_with(3);
        let location = Location(index, index + 3);
        if s == "null" {
            Ok(Token::null(location))
        } else {
            Err(LexerError::invalid_chars(s.to_string(), Some(location)))
        }
    }

    fn scan_comment_token(&mut self) -> Result<Token, LexerError> {
        let (second_slash, next_char) = self
            .input
            .next()
            .ok_or(LexerError::not_exist_terminal_symbol())?;
        match next_char {
            '/' => {
                let start = second_slash - 1;
                let mut value = String::new();
                while let Some((index, c)) = self.input.peek() {
                    if c == &'\n' {
                        return Ok(Token::comment_line(&value, Location(start, *index)));
                    } else {
                        // peekしてるのでunwrap
                        let (_, c) = self.input.next().unwrap();
                        value.push(c);
                    }
                }
            }
            '*' => {
                let start = second_slash - 1;
                let mut value = String::new();
                let mut asterisk_buffer = String::new();
                let mut prev_asterisk = false;
                while let Some((index, c)) = self.input.next() {
                    match c {
                        '*' => {
                            prev_asterisk = true;
                            asterisk_buffer.push(c);
                        }
                        '/' => {
                            if prev_asterisk {
                                return Ok(Token::comment_block(&value, Location(start, index)));
                            }
                        }
                        _ => {
                            if prev_asterisk {
                                value.push_str(&asterisk_buffer);
                                asterisk_buffer.clear();
                            }
                            prev_asterisk = false;
                            value.push(c);
                        }
                    };
                }
            }
            c => {
                return Err(LexerError::invalid_chars(
                    format!("/{}", c).to_string(),
                    Some(Location(second_slash, second_slash + 1)),
                ))
            }
        }
        Err(LexerError::not_exist_terminal_symbol())
    }

    fn scan_whitespaces(&mut self) -> Result<Token, LexerError> {
        let mut length: usize = 1; // 呼び出し時点で1
        while let Some((index, c)) = self.input.peek() {
            let c = *c;
            match c {
                ' ' => {
                    self.input.next().unwrap();
                    length += 1
                }
                _ => {
                    let index = *index;
                    return Ok(Token::white_spaces(
                        length as i32,
                        Location(index - length, index - 1),
                    ));
                }
            }
        }
        Err(LexerError::not_exist_terminal_symbol())
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
    fn lexer_should_success_scan() {
        let mut lexer = Lexer::new(
            r#"{
    "name": "sato",
    "age": 20,
    "flag": false,
    "attr": null
    // line
    /**
     * block
     */
}"#,
        );
        let result = lexer.tokenize().expect("lexerは配列を返します。");
        let expected = [
            Token::open_brace(Location(0, 0)),
            Token::break_line(Location(1, 1)),
            Token::white_spaces(4, Location(2, 5)),
            Token::string("name", Location(6, 11)),
            Token::colon(Location(12, 12)),
            Token::white_spaces(1, Location(13, 13)),
            Token::string("sato", Location(14, 19)),
            Token::comma(Location(20, 20)),
            Token::break_line(Location(21, 21)),
            Token::white_spaces(4, Location(22, 25)),
            Token::string("age", Location(26, 30)),
            Token::colon(Location(31, 31)),
            Token::white_spaces(1, Location(32, 32)),
            Token::number("20", Location(34, 35)),
            Token::comma(Location(35, 35)),
            Token::break_line(Location(36, 36)),
            Token::white_spaces(4, Location(37, 40)),
            Token::string("flag", Location(41, 46)),
            Token::colon(Location(47, 47)),
            Token::white_spaces(1, Location(48, 48)),
            Token::boolean(false, Location(49, 53)),
            Token::comma(Location(54, 54)),
            Token::break_line(Location(55, 55)),
            Token::white_spaces(4, Location(56, 59)),
            Token::string("attr", Location(60, 65)),
            Token::colon(Location(66, 66)),
            Token::white_spaces(1, Location(67, 67)),
            Token::null(Location(68, 71)),
            Token::break_line(Location(72, 72)),
            Token::white_spaces(4, Location(73, 76)),
            Token::comment_line(" line", Location(77, 84)),
            Token::break_line(Location(84, 84)),
            Token::white_spaces(4, Location(85, 88)),
            Token::comment_block(
                r#"*
     * block
     "#,
                Location(89, 112),
            ),
            Token::break_line(Location(113, 113)),
            Token::close_brace(Location(114, 114)),
        ];
        for (index, expect) in expected.iter().enumerate() {
            assert_eq!(expect, &result[index], "tokenの{}番目が想定外です。", index,);
        }
        assert_eq!(36, result.len(), "token配列長が想定外です。");
    }

    #[test]
    fn scan_string_token_should_return_token() {
        let mut lexer = Lexer::new(r#""name123""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect("[scan_string_token_should_return_token]\"name\"のscanに失敗しました。");
        assert_eq!(Token::string("name123", Location(0, 8)), token);

        let mut lexer = Lexer::new(r#""あいうえお""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect("[scan_string_token_should_return_token]\"あいうえお\"のscanに失敗しました。");
        assert_eq!(Token::string("あいうえお", Location(0, 6)), token);

        let mut lexer = Lexer::new(r#""\u3042\u3044\u3046abc""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect("[scan_string_token_should_return_token]\"あいうabc\"のscanに失敗しました。");
        assert_eq!(
            Token::string("\\u3042\\u3044\\u3046abc", Location(0, 22)),
            token
        );

        let mut lexer = Lexer::new(r#""\ud83d\ude00\ud83d\udc4d""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect("[scan_string_token_should_return_token]\"😀👍\"のscanに失敗しました。");
        assert_eq!(
            Token::string("\\ud83d\\ude00\\ud83d\\udc4d", Location(0, 25)),
            token
        );

        let mut lexer = Lexer::new(r#""😀👍""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect("[scan_string_token_should_return_token]\"😀👍\"のscanに失敗しました。");
        assert_eq!(Token::string("😀👍", Location(0, 3)), token);

        let mut lexer = Lexer::new(r#""test\"\/\\\b\n\f\r\t""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect(r#"[scan_string_token_should_return_token]"test\"\/\\\b\n\f\r\t""のscanに失敗しました。"#);
        assert_eq!(
            Token::string(r#"test\"\/\\\b\n\f\r\t"#, Location(0, 21)),
            token
        );
    }

    #[test]
    fn scan_string_token_should_err() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new("name");
        assert!(lexer.scan_string_token().is_err());
    }

    #[test]
    fn scan_number_token_should_return_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":100,");
        // 最初の`"`まで進める
        lexer.input.next();
        let (_, first) = lexer.input.next().unwrap();
        if let Ok(token) = lexer.scan_number_token(first) {
            assert_eq!(Token::number("100", Location(2, 4)), token);
        } else {
            panic!("[scan_string_token]がErrを返しました。");
        };
    }

    #[test]
    fn scan_number_token_should_err() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":100");
        // 最初の`"`まで進める
        lexer.input.next();
        let (_, first) = lexer.input.next().unwrap();
        assert!(lexer.scan_number_token(first).is_err());
    }

    #[test]
    fn scan_bool_token_should_return_true_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":true,");
        // 最初の`t`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        if let Ok(token) = lexer.scan_bool_token(true, index) {
            assert_eq!(Token::boolean(true, Location(1, 4)), token);
        } else {
            panic!("[scan_string_token]がErrを返しました。");
        };
    }

    #[test]
    fn scan_bool_token_should_err_with_true() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":tru");
        // 最初の`t`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        assert!(lexer.scan_bool_token(true, index).is_err());
    }

    #[test]
    fn scan_bool_token_should_return_false_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":false,");
        // 最初の`f`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        if let Ok(token) = lexer.scan_bool_token(false, index) {
            assert_eq!(Token::boolean(false, Location(1, 5)), token);
        } else {
            panic!("[scan_bool_token]がErrを返しました。");
        };
    }

    #[test]
    fn scan_bool_token_should_err_with_false() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":fal");
        // 最初の`f`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        assert!(lexer.scan_bool_token(false, index).is_err());
    }

    #[test]
    fn scan_null_token_should_return_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":null,");
        // 最初の`f`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        if let Ok(token) = lexer.scan_null_token(index) {
            assert_eq!(Token::null(Location(1, 4)), token);
        } else {
            panic!("[scan_null_token]がErrを返しました。");
        };
    }

    #[test]
    fn scan_null_token_should_err() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(":nu");
        // 最初の`n`まで進める
        lexer.input.next();
        let (index, _) = lexer.input.next().unwrap();
        assert!(lexer.scan_null_token(index).is_err());
    }

    #[test]
    fn scan_comment_line_token_should_return_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(",// comment \n}");
        // 最初の`/`まで進める
        lexer.input.next();
        lexer.input.next();
        if let Ok(token) = lexer.scan_comment_token() {
            assert_eq!(Token::comment_line(" comment ", Location(1, 12)), token);
        } else {
            panic!("[scan_comment_token]がErrを返しました。");
        };
    }

    #[test]
    fn scan_comment_block_token_should_return_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(
            r#"/*
**
test comment
**
*/"#,
        );
        // 最初の`/`まで進める
        lexer.input.next();
        if let Ok(token) = lexer.scan_comment_token() {
            assert_eq!(
                Token::comment_block(
                    r#"
**
test comment
**
"#,
                    Location(0, 23)
                ),
                token
            );
        } else {
            panic!("[scan_comment_token]がErrを返しました。");
        };
    }

    #[test]
    fn scan_comment_token_should_err() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new("/,");
        lexer.input.next().unwrap();
        assert!(lexer.scan_comment_token().is_err());
    }

    #[test]
    fn scan_whitespaces_token_should_return_token() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(r#"   ""#);
        // 最初の` `まで進める
        lexer.input.next();
        if let Ok(token) = lexer.scan_whitespaces() {
            assert_eq!(Token::white_spaces(3, Location(0, 2)), token);
        } else {
            panic!("[scan_whitespaces]がErrを返しました。");
        };
    }

    #[test]
    fn scan_whitespaces_token_should_err() {
        // 部分的なテストのためのinvalid json
        let mut lexer = Lexer::new(r#"  "#);
        lexer.input.next().unwrap();
        assert!(lexer.scan_whitespaces().is_err());
    }
}
