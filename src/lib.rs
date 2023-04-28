mod lex;
mod merge;
mod parse;

pub use crate::lex::Token;
pub use crate::merge::merge;
pub use crate::parse::parse;
use itertools::Itertools;
use logos::Logos;
use num_bigint::BigInt;
use scientific::Scientific;
use std::collections::BTreeMap;
use std::fmt;

pub fn from_str(s: &str) -> anyhow::Result<Value> {
    merge(parse(Token::lexer(s))?, false)
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Path(Vec<PathSegment>);
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum PathSegment {
    Object(String),
    Array(usize),
}
pub enum PathTarget {
    Null,
    Bool(bool),
    Int(BigInt),
    Float(Scientific),
    String(String),
    EmptyArray,
    EmptyObject,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Int(BigInt),
    Float(Scientific),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.iter().format("/"))
    }
}
impl fmt::Display for PathSegment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathSegment::Object(x) => write!(f, "{x}"),
            PathSegment::Array(x) => write!(f, "{x}"),
        }
    }
}
impl fmt::Display for PathTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathTarget::Null => write!(f, "null"),
            PathTarget::Bool(x) => write!(f, "{x}"),
            PathTarget::Int(x) => write!(f, "{x}"),
            PathTarget::Float(x) => write!(f, "{x}"),
            PathTarget::String(x) => write!(f, "\"{x}\""),
            PathTarget::EmptyArray => write!(f, "[]"),
            PathTarget::EmptyObject => write!(f, "{{}}"),
        }
    }
}
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(x) => write!(f, "{x}"),
            Value::Int(x) => write!(f, "{x}"),
            Value::Float(x) => {
                write!(f, "{x}")?;
                if x.decimals() <= 0 {
                    f.write_str(".0")?;
                }
                Ok(())
            }
            Value::String(x) => write!(f, "\"{x}\""),
            Value::Array(xs) => write!(f, "[{}]", xs.iter().format(",")),
            // FIXME: Perf
            Value::Object(xs) => write!(
                f,
                "{{{}}}",
                xs.iter().map(|(k, v)| format!("\"{k}\":{v}")).format(",")
            ),
        }
    }
}

impl TryFrom<PathTarget> for Value {
    type Error = anyhow::Error;
    fn try_from(value: PathTarget) -> Result<Self, Self::Error> {
        Ok(match value {
            PathTarget::Null => Value::Null,
            PathTarget::Bool(x) => Value::Bool(x),
            PathTarget::Int(x) => Value::Int(x),
            PathTarget::Float(x) => Value::Float(x),
            PathTarget::String(x) => Value::String(x),
            PathTarget::EmptyArray => Value::Array(vec![]),
            PathTarget::EmptyObject => Value::Object(BTreeMap::new()),
        })
    }
}

impl PartialEq<serde_json::Value> for Value {
    fn eq(&self, other: &serde_json::Value) -> bool {
        // let sanitize = |s: &str| s.replace(['\\', '\t', '\r', '\n'], "");
        match (self, other) {
            (Value::Null, serde_json::Value::Null) => true,
            (Value::Bool(x), serde_json::Value::Bool(y)) => x == y,
            (Value::Int(x), serde_json::Value::Number(y)) => {
                let y: BigInt = y.to_string().parse().unwrap();
                x == &y
            }
            (Value::Float(x), serde_json::Value::Number(y)) => {
                let y: Scientific = y.to_string().parse().unwrap();
                x == &y
            }
            (Value::String(x), serde_json::Value::String(y)) => x == y,
            (Value::Array(x), serde_json::Value::Array(y)) => x == y,
            (Value::Object(x), serde_json::Value::Object(y)) => {
                // TODO: sort
                x.len() == y.len() && x.iter().zip(y).all(|(x, y)| x.0 == y.0 && x.1 == y.1)
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty() {
        assert_eq!(from_str("'':''").unwrap(), json!({"": ""}));
        assert_eq!(from_str("''\n:2\n").unwrap(), json!({"": 2}));
    }
}
