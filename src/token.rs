use thiserror::Error;

/// Location情報
/// (start, end)で保持する
/// ```
/// let a = Location(start, end);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location(pub usize, pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
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
    BreakLine,
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LexerError {
    #[error("Invalid chars `{0}`")]
    InvalidChars(String, Location),
    #[error("Not exist terminal symbol char")]
    NotExistTerminalSymbol, // 終端記号が不在
    #[error("Not escape string")]
    NotEscapeString,
}
