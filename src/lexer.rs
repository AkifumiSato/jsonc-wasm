use crate::token::{LexerError, Location, Token};
use crate::utils::is_number_token_char;
use anyhow::Result;
use std::iter::{Enumerate, Peekable};
use std::str::Chars;

pub struct Lexer<'a> {
    input: Peekable<Enumerate<Chars<'a>>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.chars().enumerate().peekable(),
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = vec![];

        while let Some((index, c)) = self.input.next() {
            match c {
                '{' => tokens.push(Token::OpenBrace),
                '}' => tokens.push(Token::CloseBrace),
                '[' => tokens.push(Token::OpenBracket),
                ']' => tokens.push(Token::CloseBracket),
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
                ':' => tokens.push(Token::Colon),
                ',' => tokens.push(Token::Comma),
                '/' => {
                    let token = self.scan_comment_token()?;
                    tokens.push(token);
                }
                ' ' => {
                    let token = self.scan_whitespaces()?;
                    tokens.push(token);
                }
                '\n' => tokens.push(Token::BreakLine),
                _ => (),
            };
        }

        Ok(tokens)
    }

    fn scan_string_token(&mut self) -> Result<Token> {
        let mut value = String::new();

        while let Some((_index, c)) = self.input.next() {
            match c {
                '"' => {
                    return Ok(Token::StringValue(value));
                }
                '\\' => {
                    let (_, c2) = self
                        .input
                        .next()
                        .ok_or(LexerError::NotExistTerminalSymbol)?;
                    match c2 {
                        'u' => {
                            let hex = self.take_chars_with(4);
                            if hex.len() != 4 && hex.parse::<f64>().is_ok() {
                                return Err(LexerError::NotExistTerminalSymbol.into());
                            }

                            value.push_str(&format!("\\u{}", hex));
                        }
                        '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => {
                            value.push_str(&format!("\\{}", c2));
                        }
                        _ => {
                            return Err(LexerError::NotEscapeString.into());
                        }
                    }
                }
                _ => {
                    value.push(c);
                }
            }
        }
        Err(LexerError::NotExistTerminalSymbol.into())
    }

    fn scan_number_token(&mut self, first: char) -> Result<Token> {
        let mut value = String::new();
        value.push(first);

        while let Some((_index, c)) = self.input.peek() {
            if is_number_token_char(*c) {
                let (_, c) = self.input.next().unwrap();
                value.push(c);
            } else {
                return Ok(Token::Number(value));
            }
        }
        Err(LexerError::NotExistTerminalSymbol.into())
    }

    fn scan_bool_token(&mut self, expect_bool: bool, index: usize) -> Result<Token> {
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
            "true" => Ok(Token::Boolean(true)),
            "false" => Ok(Token::Boolean(false)),
            other => Err(LexerError::InvalidChars(other.to_string(), location).into()),
        }
    }

    fn scan_null_token(&mut self, index: usize) -> Result<Token> {
        // `null`かどうか文字を取得
        let s = "n".to_string() + &self.take_chars_with(3);
        let location = Location(index, index + 3);
        if s == "null" {
            Ok(Token::Null)
        } else {
            Err(LexerError::InvalidChars(s.to_string(), location).into())
        }
    }

    fn scan_comment_token(&mut self) -> Result<Token> {
        let (second_slash, next_char) = self
            .input
            .next()
            .ok_or(LexerError::NotExistTerminalSymbol)?;
        match next_char {
            '/' => {
                let mut value = String::new();
                while let Some((_index, c)) = self.input.peek() {
                    if c == &'\n' {
                        return Ok(Token::CommentLine(value));
                    } else {
                        // peekしてるのでunwrap
                        let (_, c) = self.input.next().unwrap();
                        value.push(c);
                    }
                }
            }
            '*' => {
                let mut value = String::new();
                let mut asterisk_buffer = String::new();
                let mut prev_asterisk = false;
                while let Some((_index, c)) = self.input.next() {
                    match c {
                        '*' => {
                            prev_asterisk = true;
                            asterisk_buffer.push(c);
                        }
                        '/' => {
                            if prev_asterisk {
                                return Ok(Token::CommentBlock(value));
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
                return Err(LexerError::InvalidChars(
                    format!("/{}", c).to_string(),
                    Location(second_slash, second_slash + 1),
                )
                .into())
            }
        }
        Err(LexerError::NotExistTerminalSymbol.into())
    }

    fn scan_whitespaces(&mut self) -> Result<Token> {
        let mut length: usize = 1; // 呼び出し時点で1
        while let Some((_index, c)) = self.input.peek() {
            let c = *c;
            match c {
                ' ' => {
                    self.input.next().unwrap();
                    length += 1
                }
                _ => {
                    return Ok(Token::WhiteSpaces(length as i32));
                }
            }
        }
        Err(LexerError::NotExistTerminalSymbol.into())
    }

    fn take_chars_with(&mut self, times: i32) -> String {
        let chars = (0..times)
            .filter_map(|_| self.input.next().map(|(_index, c)| c))
            .collect::<String>();
        chars
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

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
            Token::OpenBrace,
            Token::BreakLine,
            Token::WhiteSpaces(4),
            Token::StringValue("name".to_string()),
            Token::Colon,
            Token::WhiteSpaces(1),
            Token::StringValue("sato".to_string()),
            Token::Comma,
            Token::BreakLine,
            Token::WhiteSpaces(4),
            Token::StringValue("age".to_string()),
            Token::Colon,
            Token::WhiteSpaces(1),
            Token::Number("20".to_string()),
            Token::Comma,
            Token::BreakLine,
            Token::WhiteSpaces(4),
            Token::StringValue("flag".to_string()),
            Token::Colon,
            Token::WhiteSpaces(1),
            Token::Boolean(false),
            Token::Comma,
            Token::BreakLine,
            Token::WhiteSpaces(4),
            Token::StringValue("attr".to_string()),
            Token::Colon,
            Token::WhiteSpaces(1),
            Token::Null,
            Token::BreakLine,
            Token::WhiteSpaces(4),
            Token::CommentLine(" line".to_string()),
            Token::BreakLine,
            Token::WhiteSpaces(4),
            Token::CommentBlock(
                r#"*
     * block
     "#
                .to_string(),
            ),
            Token::BreakLine,
            Token::CloseBrace,
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
        assert_eq!(Token::StringValue("name123".to_string()), token);

        let mut lexer = Lexer::new(r#""あいうえお""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect("[scan_string_token_should_return_token]\"あいうえお\"のscanに失敗しました。");
        assert_eq!(Token::StringValue("あいうえお".to_string()), token);

        let mut lexer = Lexer::new(r#""\u3042\u3044\u3046abc""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect("[scan_string_token_should_return_token]\"あいうabc\"のscanに失敗しました。");
        assert_eq!(
            Token::StringValue("\\u3042\\u3044\\u3046abc".to_string()),
            token
        );

        let mut lexer = Lexer::new(r#""\ud83d\ude00\ud83d\udc4d""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect("[scan_string_token_should_return_token]\"😀👍\"のscanに失敗しました。");
        assert_eq!(
            Token::StringValue("\\ud83d\\ude00\\ud83d\\udc4d".to_string()),
            token
        );

        let mut lexer = Lexer::new(r#""😀👍""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect("[scan_string_token_should_return_token]\"😀👍\"のscanに失敗しました。");
        assert_eq!(Token::StringValue("😀👍".to_string()), token);

        let mut lexer = Lexer::new(r#""test\"\/\\\b\n\f\r\t""#);
        // 最初の"まで進める
        lexer.input.next();
        let token = lexer
            .scan_string_token()
            .expect(r#"[scan_string_token_should_return_token]"test\"\/\\\b\n\f\r\t""のscanに失敗しました。"#);
        assert_eq!(
            Token::StringValue(r#"test\"\/\\\b\n\f\r\t"#.to_string()),
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
            assert_eq!(Token::Number("100".to_string()), token);
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
            assert_eq!(Token::Boolean(true), token);
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
            assert_eq!(Token::Boolean(false), token);
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
            assert_eq!(Token::Null, token);
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
            assert_eq!(Token::CommentLine(" comment ".to_string()), token);
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
                Token::CommentBlock(
                    r#"
**
test comment
**
"#
                    .to_string()
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
            assert_eq!(Token::WhiteSpaces(3), token);
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
