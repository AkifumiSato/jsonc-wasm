use core::cmp::{max, min};

/// Location情報
/// (start, end)で保持する
/// ```
/// let a = Location(start, end);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location(pub usize, pub usize);

impl Location {
    fn merge(&self, target: Location) -> Location {
        Location(min(self.0, target.0), max(self.1, target.1))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation<T> {
    pub value: T,
    location: Option<Location>,
}

impl<T> Annotation<T> {
    pub fn new(value: T, location: Option<Location>) -> Self {
        Annotation { value, location }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
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

pub type Token = Annotation<TokenKind>;

impl Token {
    pub fn open_brace(location: Location) -> Self {
        Self::new(TokenKind::OpenBrace, Some(location))
    }

    pub fn close_brace(location: Location) -> Self {
        Self::new(TokenKind::CloseBrace, Some(location))
    }

    pub fn open_bracket(location: Location) -> Self {
        Self::new(TokenKind::OpenBracket, Some(location))
    }

    pub fn close_bracket(location: Location) -> Self {
        Self::new(TokenKind::CloseBracket, Some(location))
    }

    pub fn string(value: &str, location: Location) -> Self {
        Self::new(TokenKind::StringValue(value.to_string()), Some(location))
    }

    pub fn number(value: &str, location: Location) -> Self {
        Self::new(TokenKind::Number(value.to_string()), Some(location))
    }

    pub fn boolean(value: bool, location: Location) -> Self {
        Self::new(TokenKind::Boolean(value), Some(location))
    }

    pub fn null(location: Location) -> Self {
        Self::new(TokenKind::Null, Some(location))
    }

    pub fn comment_line(value: &str, location: Location) -> Self {
        Self::new(TokenKind::CommentLine(value.to_string()), Some(location))
    }

    pub fn comment_block(value: &str, location: Location) -> Self {
        Self::new(TokenKind::CommentBlock(value.to_string()), Some(location))
    }

    pub fn comma(location: Location) -> Self {
        Self::new(TokenKind::Comma, Some(location))
    }

    pub fn colon(location: Location) -> Self {
        Self::new(TokenKind::Colon, Some(location))
    }

    pub fn white_spaces(length: i32, location: Location) -> Self {
        Self::new(TokenKind::WhiteSpaces(length), Some(location))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerErrorKind {
    InvalidChars(String),
    NotExistTerminalSymbol, // 終端記号が不在
}

pub type LexerError = Annotation<LexerErrorKind>;

impl LexerError {
    pub fn invalid_chars(chars: String, location: Option<Location>) -> Self {
        Annotation::new(LexerErrorKind::InvalidChars(chars), location)
    }
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
}
