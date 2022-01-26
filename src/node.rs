use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Node {
    StringValue(String),
    Number(String), // 浮動少数誤差を扱わないため、String
    Boolean(bool),
    Null,
    Object(HashMap<String, Node>),
    Array(Vec<Node>),
}
