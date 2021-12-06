extern crate wasm_bindgen;

use std::cmp::{max, min};
use std::iter::Peekable;
use std::str::Bytes;
use wasm_bindgen::prelude::*;

/// Location情報
/// (start, end)で保持する
/// ```
/// let a = Location(start, end);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
struct Location(usize, usize);

impl Location {
    fn merge(&self, target: Location) -> Location {
        Location(min(self.0, target.0), max(self.1, target.1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Annotation<T> {
    value: T,
    location: Location,
}

impl<T> Annotation<T> {
    fn new(value: T, location: Location) -> Self {
        Annotation { value, location }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    OpenBrace,    // `{`
    CloseBrace,   // `}`
    OpenBracket,  // `[`
    CloseBracket, // `]`
    StringValue(String),
    Number(String), // 浮動少数誤差を扱わないため、String
    Boolean(bool),
    Null,
    CommentLine(String),
    CommentBlock(String),
    Comma,
    Colon,
    WhiteSpaces(i32), // Length
}

type Token = Annotation<TokenKind>;

impl Token {
    fn open_brace(location: Location) -> Self {
        Self::new(TokenKind::OpenBrace, location)
    }

    fn close_brace(location: Location) -> Self {
        Self::new(TokenKind::CloseBrace, location)
    }

    fn open_bracket(location: Location) -> Self {
        Self::new(TokenKind::OpenBracket, location)
    }

    fn close_bracket(location: Location) -> Self {
        Self::new(TokenKind::CloseBracket, location)
    }

    fn string(value: &str, location: Location) -> Self {
        Self::new(TokenKind::StringValue(value.to_string()), location)
    }

    fn number(value: &str, location: Location) -> Self {
        Self::new(TokenKind::Number(value.to_string()), location)
    }

    fn boolean(value: bool, location: Location) -> Self {
        Self::new(TokenKind::Boolean(value), location)
    }

    fn null(location: Location) -> Self {
        Self::new(TokenKind::Null, location)
    }

    fn comment_line(value: &str, location: Location) -> Self {
        Self::new(TokenKind::CommentLine(value.to_string()), location)
    }

    fn comment_block(value: &str, location: Location) -> Self {
        Self::new(TokenKind::CommentBlock(value.to_string()), location)
    }

    fn comma(location: Location) -> Self {
        Self::new(TokenKind::Comma, location)
    }

    fn colon(location: Location) -> Self {
        Self::new(TokenKind::Colon, location)
    }

    fn white_spaces(length: i32, location: Location) -> Self {
        Self::new(TokenKind::WhiteSpaces(length), location)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LexerErrorKind {
    InvalidChars(String),
}

type LexerError = Annotation<LexerErrorKind>;

impl LexerError {
    fn invalid_chars(chars: String, location: Location) -> Self {
        Annotation::new(LexerErrorKind::InvalidChars(chars), location)
    }
}

struct Lexer<'a> {
    input: Peekable<Bytes<'a>>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            input: input.bytes().peekable(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        while let Some((value, start, end)) = self.next_byte_info() {
            let value: &str = &value;
            match value {
                "{" => tokens.push(Token::open_brace(Location(start, end))),
                "}" => tokens.push(Token::close_brace(Location(start, end))),
                ":" => tokens.push(Token::colon(Location(start, end))),
                "," => tokens.push(Token::comma(Location(start, end))),
                // todo 文字列や数字の読み込みで特定のバイト数まで一気に読み込めるよう実装
                // ループ条件を見直してiteratorにした方がやりやすいかも
                "\"" => {
                    let token = self.parse_string_token()?;
                    tokens.push(token);
                },
                _ => (),
            };
        }

        Ok(tokens)
    }

    fn next_byte_info(&mut self) -> Option<(String, usize, usize)> {
        use std::str::from_utf8;

        let input = &mut self.input;

        if let Some(b) = input.next() {
            // 以下実装Todo
            // 1. 直前に読み込んでた文字列が連続的な値を持つTokenかどうか
            // 2. そうならpeekした値とくっつけてmatch
            // 3. matchしないなら再帰
            // 4. 想定外のTokenとマッチするようならエラー
            let start = self.pos;
            let value = from_utf8(&[b]).unwrap().to_string();
            self.pos += 1;
            return Some((value, start, self.pos))
        };
        None
    }

    fn parse_string_token(&mut self) -> Result<Token, LexerError> {
        use std::str::from_utf8;

        let mut value = String::new();
        let start = self.pos;

        while let Some(byte) = self.input.next() {
            let target = &[byte].clone();
            let char = from_utf8(target).unwrap();

            match char {
                "\"" => {
                    return Ok(Token::string(&value, Location(start, self.pos)));
                },
                _ => {
                    value.push_str(char);
                    self.pos += 1;
                }
                // todo escape文字列の処理
            }
        }
        Err(LexerError::new(LexerErrorKind::InvalidChars(value), Location(start, self.pos)))
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
    fn location_merge() {
        let location1 = Location(0, 1);
        let location2 = Location(1, 10);
        let merge_result = location1.merge(location2);
        assert_eq!(merge_result.0, 0);
        assert_eq!(merge_result.1, 10);
    }

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
            Token::colon(Location(12, 13)),
            Token::string("sato", Location(15, 19)),
            Token::close_brace(Location(20, 21)),
        ];
        assert_eq!(5, result.len());
        assert_eq!(TokenKind::OpenBrace, result[0].value);
        assert_eq!(TokenKind::StringValue("name".to_string()), result[1].value);
        assert_eq!(TokenKind::Colon, result[2].value);
        assert_eq!(TokenKind::StringValue("sato".to_string()), result[3].value);
        assert_eq!(TokenKind::CloseBrace, result[4].value);
        assert_eq!(expected, result);
    }
}
