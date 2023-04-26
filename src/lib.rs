mod lex;
mod merge;
mod parse;

pub use crate::lex::Token;
pub use crate::merge::merge;
pub use crate::parse::parse;
use itertools::Itertools;
use serde_json::Value;
use std::fmt;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Path(Vec<PathSegment>);
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum PathSegment {
    Object(String),
    Array(usize),
}
pub enum PathTarget {
    Value(Value),
    Ref(Path),
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
            PathTarget::Value(x) => write!(f, "{x}"),
            PathTarget::Ref(x) => write!(f, "->{x}"),
        }
    }
}
