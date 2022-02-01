use crate::node::Node;
use crate::token::Token;
use anyhow::{ensure, Result};
use std::collections::BTreeMap;
use std::iter::Peekable;
use std::slice::Iter;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    #[error("Not found token")]
    NotFoundToken,
    #[error("Unexpected Token: `{0}`")]
    UnexpectedToken(String),
    #[error("Unexpected consumed up Token")]
    UnexpectedConsumedUpToken,
    #[error("Un closed Token")]
    UnClosedToken,
}

pub struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<Token>) -> Self {
        Parser {
            tokens: tokens.iter().peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Node> {
        if self.tokens.len() == 0 {
            return Err(ParseError::NotFoundToken.into());
        }
        let result = self.parse_value()?;
        ensure!(
            self.next_grammar().is_none(),
            ParseError::UnexpectedToken("contains multiple values".to_string())
        );
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
            Token::OpenBracket => self.parse_array(),
            _ => Err(ParseError::UnexpectedToken(
                "contains a token other than the value".to_string(),
            )
            .into()),
        }
    }

    fn parse_object(&mut self) -> Result<Node> {
        let mut times = 0;
        let mut member = BTreeMap::new();
        loop {
            // close,comma,stringのいづれか
            let first_token = self.next_grammar().ok_or(ParseError::UnClosedToken)?;
            let key = match first_token {
                Token::CloseBrace => break, // ループを終了
                Token::Comma => {
                    // 0回目の時はcommaはなし
                    if times == 0 {
                        return Err(ParseError::UnexpectedToken(
                            "first comma is not allowed".to_string(),
                        )
                        .into());
                    } else {
                        let token = self.next_grammar().ok_or(ParseError::UnexpectedToken(
                            "found a Token that cannot be a key".to_string(),
                        ))?;
                        match token {
                            Token::CloseBrace => break, // ループを終了
                            Token::StringValue(key) => key,
                            _ => {
                                return Err(ParseError::UnexpectedToken(
                                    "found a Token that cannot be a key".to_string(),
                                )
                                .into());
                            }
                        }
                    }
                }
                Token::StringValue(key) => key, // key tokenはstringのみ許容 https://www.rfc-editor.org/rfc/rfc8259#section-4
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        "found a Token that cannot be a key".to_string(),
                    )
                    .into());
                }
            };

            match (key, self.next_grammar(), self.parse_value()?) {
                (key, Some(Token::Colon), node) => {
                    member.insert(key, node);
                }
                _ => return Err(ParseError::UnexpectedConsumedUpToken.into()),
            }

            times += 1;
        }
        Ok(Node::Object(member))
    }

    fn parse_array(&mut self) -> Result<Node> {
        let mut times = 0;
        let mut result = vec![];
        loop {
            let first_token = self.next_grammar().ok_or(ParseError::UnClosedToken)?;
            let value = match first_token {
                Token::CloseBracket => break,
                Token::Comma => {
                    // 0回目の時はcommaはなし
                    if times == 0 {
                        return Err(ParseError::UnexpectedToken(
                            "first comma is not allowed".to_string(),
                        )
                        .into());
                    } else {
                        let token = self.next_grammar().ok_or(ParseError::UnClosedToken)?;
                        if token == Token::CloseBracket {
                            break;
                        };
                        token
                    }
                }
                _ => first_token,
            };

            times += 1;

            match value {
                Token::StringValue(value) => result.push(Node::StringValue(value)),
                Token::Number(value) => result.push(Node::Number(value)),
                Token::Boolean(value) => result.push(Node::Boolean(value)),
                Token::Null => result.push(Node::Null),
                Token::OpenBrace => result.push(self.parse_object()?),
                Token::OpenBracket => result.push(self.parse_array()?),
                _ => {
                    return Err(ParseError::UnexpectedToken(
                        "found an unexpected token while parsing the array".to_string(),
                    )
                    .into())
                }
            }
        }
        Ok(Node::Array(result))
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
    use std::collections::BTreeMap;

    fn assert_parse(data: Vec<(Vec<Token>, Node)>) {
        for (tokens, expect) in data.iter() {
            let mut parser = Parser::new(tokens);
            let result = parser.parse();
            match result {
                Ok(node) => assert_eq!(*expect, node),
                Err(e) => panic!("[assert_parse]: {}", e),
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
        assert_parse_err(
            data,
            ParseError::UnexpectedToken("contains multiple values".to_string()),
        );
    }

    #[test]
    fn parse_object_value() {
        let data_expect_list = vec![
            // flat object
            (
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
                Node::Object(BTreeMap::from([
                    ("name".to_string(), Node::StringValue("sato".to_string())),
                    ("age".to_string(), Node::Number("20".to_string())),
                ])),
            ),
            // nested
            (
                vec![
                    Token::OpenBrace,
                    Token::StringValue("user".to_string()),
                    Token::Colon,
                    Token::WhiteSpaces(1),
                    Token::OpenBrace,
                    Token::StringValue("name".to_string()),
                    Token::Colon,
                    Token::StringValue("sato".to_string()),
                    Token::CloseBrace,
                    Token::CloseBrace,
                ],
                Node::Object(BTreeMap::from([(
                    "user".to_string(),
                    Node::Object(BTreeMap::from([(
                        "name".to_string(),
                        Node::StringValue("sato".to_string()),
                    )])),
                )])),
            ),
            // trailing comma
            (
                vec![
                    Token::OpenBrace,
                    Token::StringValue("name".to_string()),
                    Token::Colon,
                    Token::WhiteSpaces(1),
                    Token::StringValue("sato".to_string()),
                    Token::Comma,
                    Token::CloseBrace,
                ],
                Node::Object(BTreeMap::from([(
                    "name".to_string(),
                    Node::StringValue("sato".to_string()),
                )])),
            ),
        ];
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
        assert_parse_err(
            data,
            ParseError::UnexpectedToken("found a Token that cannot be a key".to_string()),
        );
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

        assert_parse_err(
            data,
            ParseError::UnexpectedToken("contains a token other than the value".to_string()),
        );
    }

    #[test]
    fn parse_array_value() {
        let data_expect_list = vec![
            (
                // has object
                vec![
                    Token::OpenBracket,
                    Token::BreakLine,
                    Token::WhiteSpaces(4),
                    Token::StringValue("hoge".to_string()),
                    Token::Comma,
                    Token::BreakLine,
                    Token::WhiteSpaces(4),
                    Token::Number("999".to_string()),
                    Token::Comma,
                    Token::BreakLine,
                    Token::WhiteSpaces(4),
                    Token::OpenBrace,
                    Token::StringValue("name".to_string()),
                    Token::Colon,
                    Token::StringValue("sato".to_string()),
                    Token::CloseBrace,
                    Token::Comma,
                    Token::BreakLine,
                    Token::OpenBracket,
                    Token::Number("123".to_string()),
                    Token::CloseBracket,
                    Token::CloseBracket,
                ],
                Node::Array(vec![
                    Node::StringValue("hoge".to_string()),
                    Node::Number("999".to_string()),
                    Node::Object(BTreeMap::from([(
                        "name".to_string(),
                        Node::StringValue("sato".to_string()),
                    )])),
                    Node::Array(vec![Node::Number("123".to_string())]),
                ]),
            ),
            // trailing comma
            (
                vec![
                    Token::OpenBracket,
                    Token::BreakLine,
                    Token::WhiteSpaces(4),
                    Token::StringValue("hoge".to_string()),
                    Token::Comma,
                    Token::BreakLine,
                    Token::WhiteSpaces(4),
                    Token::Number("999".to_string()),
                    Token::Comma,
                    Token::CloseBracket,
                ],
                Node::Array(vec![
                    Node::StringValue("hoge".to_string()),
                    Node::Number("999".to_string()),
                ]),
            ),
        ];
        assert_parse(data_expect_list);
    }

    #[test]
    fn parse_array_value_invalid() {
        let data = vec![
            Token::OpenBracket,
            Token::BreakLine,
            Token::WhiteSpaces(4),
            Token::StringValue("hoge".to_string()),
            Token::Comma,
        ];
        assert_parse_err(data, ParseError::UnClosedToken);
    }
}
