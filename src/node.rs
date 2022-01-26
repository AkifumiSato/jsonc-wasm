use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Node {
    StringValue(String),
    Number(String), // 浮動少数誤差を扱わないため、String
    Boolean(bool),
    Null,
    Object(BTreeMap<String, Node>),
    Array(Vec<Node>),
}

impl Node {
    pub fn to_json_string(&self) -> String {
        match self {
            Node::StringValue(value) => format!(r#""{}""#, value).to_string(),
            Node::Number(value) => value.clone(),
            Node::Boolean(value) => {
                if *value {
                    "true".to_string()
                } else {
                    "false".to_string()
                }
            }
            Node::Null => "null".to_string(),
            Node::Array(items) => {
                let values: Vec<String> = items.iter().map(|item| item.to_json_string()).collect();
                format!("[{}]", values.join(",")).to_string()
            }
            Node::Object(members) => {
                let mut key_values = vec![];
                for (key, value) in members.iter() {
                    key_values.push(format!(r#""{}":{}"#, key, value.to_json_string()));
                }
                format!("{{{}}}", key_values.join(",")).to_string()
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::Node;
    use std::collections::BTreeMap;

    #[test]
    fn string_node_should_be_value() {
        let node = Node::StringValue("test".to_string());
        assert_eq!(r#""test""#.to_string(), node.to_json_string());
    }

    #[test]
    fn number_node_should_be_value() {
        let node = Node::Number("999.99".to_string());
        assert_eq!(r#"999.99"#.to_string(), node.to_json_string());
    }

    #[test]
    fn bool_node_should_be_value() {
        let node = Node::Boolean(false);
        assert_eq!(r#"false"#.to_string(), node.to_json_string());
        let node = Node::Boolean(true);
        assert_eq!(r#"true"#.to_string(), node.to_json_string());
    }

    #[test]
    fn null_node_should_be_value() {
        let node = Node::Null;
        assert_eq!(r#"null"#.to_string(), node.to_json_string());
    }

    #[test]
    fn object_node_to_string() {
        let node = Node::Object(BTreeMap::from([(
            "key".to_string(),
            Node::StringValue("value".to_string()),
        )]));
        assert_eq!(r#"{"key":"value"}"#.to_string(), node.to_json_string());
        let node = Node::Object(BTreeMap::from([
            ("a".to_string(), Node::Null),
            ("b".to_string(), Node::Number("999.99".to_string())),
            ("c".to_string(), Node::Boolean(true)),
        ]));
        assert_eq!(
            r#"{"a":null,"b":999.99,"c":true}"#.to_string(),
            node.to_json_string()
        );
        let node = Node::Object(BTreeMap::from([(
            "a".to_string(),
            Node::Array(vec![
                Node::Number("111".to_string()),
                Node::Number("222".to_string()),
            ]),
        )]));
        assert_eq!(r#"{"a":[111,222]}"#.to_string(), node.to_json_string());
    }

    #[test]
    fn array_node_to_string() {
        let node = Node::Array(vec![Node::StringValue("first".to_string())]);
        assert_eq!(r#"["first"]"#.to_string(), node.to_json_string());
        let node = Node::Array(vec![
            Node::StringValue("first".to_string()),
            Node::Number("2".to_string()),
            Node::Boolean(false),
            Node::Null,
        ]);
        assert_eq!(
            r#"["first",2,false,null]"#.to_string(),
            node.to_json_string()
        );
    }
}
