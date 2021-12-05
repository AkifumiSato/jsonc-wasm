extern crate wasm_bindgen;

use std::cmp::{max, min};
use wasm_bindgen::prelude::*;

/// Location情報
/// (start, end)で保持する
/// ```
/// let a = Location(start, end);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
struct Location(i32, i32);

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

enum TokenKind {
    OpenBrace, // `{`
    CloseBrace, // `}`
    OpenBracket, // `[`
    CloseBracket, // `]`
    String(String),
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
        Self::new(TokenKind::String(value.to_string()), location)
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
}
