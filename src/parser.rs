use crate::token::Token;
use anyhow::{ensure, Result};
use std::collections::HashMap;
use std::iter::Peekable;
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
    #[error("Unexpected Token")]
    UnexpectedToken,
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
        let result = match token {
            Token::StringValue(value) => Node::StringValue(value),
            Token::Number(value) => Node::Number(value),
            Token::Boolean(value) => Node::Boolean(value),
            Token::Null => Node::Null,
            Token::OpenBrace => todo!("Object parse"),
            Token::OpenBracket => todo!("Array parse"),
            _ => return self.parse(),
        };
        ensure!(self.tokens.next().is_none(), ParseError::UnexpectedToken);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_value() {
        let data_expect_list = vec![
            (
                vec![Token::StringValue("test".to_string())],
                Node::StringValue("test".to_string()),
            ),
            (
                vec![Token::Number("100".to_string())],
                Node::Number("100".to_string()),
            ),
            (
                vec![Token::BreakLine, Token::Number("100".to_string())],
                Node::Number("100".to_string()),
            ),
            (vec![Token::Boolean(true)], Node::Boolean(true)),
            (vec![Token::Null], Node::Null),
        ];
        for (data, expect) in data_expect_list.iter() {
            let mut parser = Parser::new(data);
            let result = parser
                .parse()
                .expect("[parse_single_value]Parseに失敗しました。");
            assert_eq!(*expect, result)
        }
    }

    #[test]
    fn parse_single_value_error() {
        let data = vec![
            Token::StringValue("test".to_string()),
            Token::StringValue("test".to_string()),
        ];
        let mut parser = Parser::new(&data);
        let result = parser.parse();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(
            ParseError::UnexpectedToken,
            *err.downcast_ref::<ParseError>().unwrap()
        )
    }
}
