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
    #[error("Unexpected consumed up Token")]
    UnexpectedConsumedUpToken,
    #[error("Un closed Token")]
    UnClosedToken,
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
        if self.tokens.len() == 0 {
            return Err(ParseError::NotFoundToken.into());
        }
        let result = self.parse_value()?;
        ensure!(self.next_grammar().is_none(), ParseError::UnexpectedToken);
        Ok(result)
    }

    fn parse_value(&mut self) -> Result<Node> {
        let token = self
            .next_grammar()
            .ok_or(ParseError::UnexpectedConsumedUpToken)?;
        match token {
            Token::StringValue(value) => Ok(Node::StringValue(value)),
            Token::Number(value) => Ok(Node::Number(value)),
            Token::Boolean(value) => Ok(Node::Boolean(value)),
            Token::Null => Ok(Node::Null),
            Token::OpenBrace => self.parse_object(),
            Token::OpenBracket => todo!("Array parse"),
            _ => Err(ParseError::UnexpectedToken.into()),
        }
    }

    fn parse_object(&mut self) -> Result<Node> {
        let mut member = HashMap::new();
        loop {
            let mut first_token = self.next_grammar().ok_or(ParseError::UnClosedToken)?;
            if first_token == Token::CloseBrace {
                println!("[debug] end");
                // closeでないならkey stringのみ
                break;
            } else if first_token == Token::Comma {
                first_token = self.next_grammar().ok_or(ParseError::UnClosedToken)?;
            };

            match (first_token, self.next_grammar(), self.parse_value()?) {
                (Token::StringValue(key), Some(Token::Colon), node) => {
                    member.insert(key, node);
                }
                _ => return Err(ParseError::UnexpectedConsumedUpToken.into()),
            }
        }
        Ok(Node::Object(member))
    }

    /// 次のgrammarまで読み飛ばす
    fn next_grammar(&mut self) -> Option<Token> {
        // todo nextするのかどうか、検討の余地あり
        while let Some(token) = self.tokens.next() {
            match token {
                Token::BreakLine => { /* skip */ }
                Token::WhiteSpaces(_) => { /* skip */ }
                Token::CommentBlock(_) => { /* skip */ }
                Token::CommentLine(_) => { /* skip */ }
                _ => return Some(token.clone()),
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_parse(data: Vec<(Vec<Token>, Node)>) {
        for (tokens, expect) in data.iter() {
            let mut parser = Parser::new(tokens);
            let result = parser.parse();
            match result {
                Ok(node) => assert_eq!(*expect, node),
                Err(e) => panic!("{}", e),
            }
        }
    }

    fn assert_parse_err(data: Vec<Token>, expect: ParseError) {
        let mut parser = Parser::new(&data);
        let result = parser.parse();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(expect, *err.downcast_ref::<ParseError>().unwrap());
    }

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
                vec![
                    Token::BreakLine,
                    Token::Number("100".to_string()),
                    Token::WhiteSpaces(4),
                ],
                Node::Number("100".to_string()),
            ),
            (vec![Token::Boolean(true)], Node::Boolean(true)),
            (vec![Token::Null], Node::Null),
        ];
        assert_parse(data_expect_list);
    }

    #[test]
    fn parse_single_value_no_token_error() {
        let data = vec![];
        let mut parser = Parser::new(&data);
        let result = parser.parse();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(
            ParseError::NotFoundToken,
            *err.downcast_ref::<ParseError>().unwrap()
        )
    }

    #[test]
    fn parse_single_value_error() {
        let data = vec![
            Token::StringValue("test".to_string()),
            Token::StringValue("test".to_string()),
        ];
        assert_parse_err(data, ParseError::UnexpectedToken);
    }

    #[test]
    fn parse_object_value() {
        let data_expect_list = vec![(
            vec![
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
                Token::BreakLine,
                Token::CloseBrace,
            ],
            Node::Object(HashMap::from([
                ("name".to_string(), Node::StringValue("sato".to_string())),
                ("age".to_string(), Node::Number("20".to_string())),
            ])),
        )];
        assert_parse(data_expect_list);
    }

    #[test]
    fn parse_object_value_not_closed() {
        let data = vec![
            Token::OpenBrace,
            Token::BreakLine,
            Token::WhiteSpaces(4),
            Token::StringValue("name".to_string()),
            Token::Colon,
            Token::WhiteSpaces(1),
            Token::StringValue("sato".to_string()),
            Token::BreakLine,
            // Token::CloseBrace,
        ];
        assert_parse_err(data, ParseError::UnClosedToken);
    }

    #[test]
    fn parse_object_value_invalid() {
        let data = vec![
            Token::OpenBrace,
            Token::BreakLine,
            Token::WhiteSpaces(4),
            // Token::StringValue("name".to_string()),
            Token::Colon,
            Token::WhiteSpaces(1),
            Token::StringValue("sato".to_string()),
            Token::Comma,
            Token::CloseBrace,
        ];
        assert_parse_err(data, ParseError::UnexpectedToken);
    }

    #[test]
    fn parse_object_value_no_value() {
        let data = vec![
            Token::OpenBrace,
            Token::BreakLine,
            Token::WhiteSpaces(4),
            Token::StringValue("name".to_string()),
            Token::Colon,
            Token::CloseBrace,
        ];

        assert_parse_err(data, ParseError::UnexpectedToken);
    }
}
