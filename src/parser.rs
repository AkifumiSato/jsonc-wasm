use crate::token::Token;
use anyhow::Result;
use std::collections::HashMap;
use std::iter::{Enumerate, Peekable};
use std::slice::Iter;
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum Node {
    StringValue(String),
    Number(String), // 浮動少数誤差を扱わないため、String
    Boolean(bool),
    Null,
    Object(HashMap<String, Node>),
    Array(Vec<Node>),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error("Not found token")]
    NotFoundToken,
}

struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a Vec<Token>) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Node> {
        let token = self.tokens.next().ok_or(ParseError::NotFoundToken)?.clone();
        match token {
            Token::StringValue(value) => Ok(Node::StringValue(value)),
            _ => todo!("他値のParse"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_string() {
        let data = vec![Token::StringValue("test".to_string())];
        let mut parser = Parser::new(&data);
        let result = parser.parse().expect("[parse_string]Parseに失敗しました。");
        assert_eq!(Node::StringValue("test".to_string()), result)
    }
}
