use anyhow::{anyhow, bail, ensure};
use itertools::Itertools;
use logos::{Lexer, Logos};
use serde_json::Value;
use std::fmt;

#[derive(Logos, Debug)]
#[logos(skip r"[ \t]+")]
pub enum Token<'a> {
    // Keywords
    #[token("package")]
    Package,
    #[token("import")]
    Import,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("if")]
    If,
    #[token("let")]
    Let,
    #[regex(r"__(#|_#)?([a-zA-Z]|_)([a-zA-Z]|_|[0-9])*")]
    Keyword(&'a str),

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Asterisk, // Could be unary "default" or binary "times"
    #[token("/")]
    Div,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("&")]
    Unify,
    #[token("|")]
    Disjunct,
    #[token("==")]
    TestEq,
    #[token("!=")]
    TestNotEq,
    #[token("=~")]
    TestSimilar,
    #[token("!~")]
    TestNotSimilar,
    #[token("<")]
    TestLT,
    #[token(">")]
    TestGT,
    #[token("<=")]
    TestLE,
    #[token(">=")]
    TestGE,
    #[token("=")]
    Equals,
    #[token(":")]
    Colon,
    #[token("?")]
    Optional,
    #[token("!")]
    Required,
    #[token("(")]
    OpenParens,
    #[token(")")]
    CloseParens,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token("_|_")]
    Bottom,
    #[token("...")]
    Elipsis,
    #[token(",")]
    Comma,
    #[token(".")]
    Period,

    // Literals
    #[token("null")]
    Null,
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Bool(bool),
    #[regex(r"0|([1-9]([0-9_])*)", from_decimal)]
    #[regex(r"[0-9]([0-9_])*(\.[0-9]([0-9_])*)?[KMGTP]i?", from_si)]
    #[regex(r"\.[0-9][0-9]([0-9_])*[KMGTP]i?", from_si)]
    #[regex(r"0b[01][01_]*", from_binary)]
    #[regex(r"0o[0-7][0-7_]*", from_octal)]
    #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*", from_hex)]
    Int(u64),
    #[regex(r"[0-9][0-9_]*\.([0-9][0-9_]*)?([eE][+-][0-9][0-9_]*)?", from_float)]
    #[regex(r"[0-9][0-9_]*[eE][+-][0-9][0-9_]*", from_float)]
    #[regex(r"\.[0-9][0-9_]*([eE][+-][0-9][0-9_]*)?", from_float)]
    Float(f64),
    // TODO: Multiline strings
    // TODO: Escape sequences
    #[regex("\"[^\"]*\"", from_string)]
    #[regex("'[^']*'", from_string)]
    String(&'a str),

    // TODO: Use the unicode Letter and Digit classes
    // TODO: Allow '$' in identifiers
    #[regex(r"(_?#)?([a-zA-Z_$])([a-zA-Z_$0-9])*")]
    Ident(&'a str),

    #[token(r"//.*[\n\r]+")] // Comments act like newlines
    #[regex(r"[\n\r]+")]
    Newline,
}

fn from_decimal<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Option<u64> {
    lexer.slice().parse().ok()
}
fn from_si<'a>(_: &mut Lexer<'a, Token<'a>>) -> Option<u64> {
    todo!()
}
fn from_binary<'a>(_: &mut Lexer<'a, Token<'a>>) -> Option<u64> {
    todo!()
}
fn from_octal<'a>(_: &mut Lexer<'a, Token<'a>>) -> Option<u64> {
    todo!()
}
fn from_hex<'a>(_: &mut Lexer<'a, Token<'a>>) -> Option<u64> {
    todo!()
}
fn from_float<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Option<f64> {
    lexer.slice().parse().ok()
}
fn from_string<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> &'a str {
    let xs = lexer.slice();
    &xs[1..xs.len() - 1]
}

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

pub fn parse<'a>(lexer: Lexer<'a, Token<'a>>) -> anyhow::Result<Vec<(Path, PathTarget)>> {
    let mut lexer = lexer.map(|token| token.map_err(|()| anyhow!("Lexer error")));

    let mut ret = vec![];
    let mut scope: Vec<Vec<PathSegment>> = vec![];
    let mut path = vec![];
    let mut prev = None;
    let mut reference = vec![];

    macro_rules! next_token {
        ($pat: pat) => {
            let Some($pat) = lexer.next().transpose()? else { bail!("Unexpected token") };
        };
    }
    macro_rules! store_token {
        ($token: expr) => {{
            match $token {
                Token::Ident(x) => store_ref!(PathSegment::Object(x.to_string())),
                Token::String(x) => {
                    if reference.is_empty() {
                        store_value!(Value::from(x));
                    } else {
                        store_ref!(PathSegment::Object(x.to_string()));
                    }
                }
                Token::Null => store_value!(Value::Null),
                Token::Bool(x) => store_value!(Value::from(x)),
                Token::Int(x) => store_value!(Value::from(x)),
                Token::Float(x) => store_value!(Value::from(x)),
                x => panic!("{x:?}"),
            }
        }};
    }
    macro_rules! store_value {
        ($value: expr) => {{
            ret.push((
                Path(
                    scope
                        .iter()
                        .cloned()
                        .flatten()
                        .chain(path.drain(..))
                        .collect(),
                ),
                PathTarget::Value($value),
            ));
            path.clear();
        }};
    }
    macro_rules! store_ref {
        ($value: expr) => {{
            ret.push((
                Path(
                    scope
                        .iter()
                        .cloned()
                        .flatten()
                        .chain(path.drain(..))
                        .collect(),
                ),
                PathTarget::Ref(Path(reference.drain(..).chain([$value]).collect())),
            ));
            reference.clear();
        }};
    }

    while let Some(token) = lexer.next().transpose()? {
        match token {
            Token::Package => {
                // TODO: Outside the preamble, this should be converted into an identifier
                next_token!(Token::Ident(x));
                next_token!(Token::Newline);
                eprintln!("package {x}");
            }
            Token::Import => {
                // TODO: Outside the preamble, this should be converted into an identifier
                next_token!(Token::Ident(x));
                next_token!(Token::Newline);
                eprintln!("import {x}");
            }
            Token::Ident(_)
            | Token::Null
            | Token::Bool(_)
            | Token::Int(_)
            | Token::Float(_)
            | Token::String(_) => {
                assert!(prev.is_none());
                prev = Some(token);
            }
            Token::Colon => match prev.take() {
                Some(Token::Ident(x)) => path.push(PathSegment::Object(x.to_string())),
                Some(Token::String(x)) => path.push(PathSegment::Object(x.to_string())),
                _ => panic!(),
            },
            Token::Period => match prev.take() {
                Some(Token::String(x)) => reference.push(PathSegment::Object(x.to_string())),
                _ => panic!(),
            },
            Token::Newline => {
                if let Some(token) = prev.take() {
                    store_token!(token)
                }
            }
            Token::Comma => {
                if let Some(token) = prev.take() {
                    store_token!(token)
                }
                if let Some(PathSegment::Array(x)) = scope.last_mut().and_then(|x| x.last_mut()) {
                    *x += 1;
                }
            }
            Token::OpenBrace => scope.push(path.drain(..).collect::<Vec<_>>()),
            Token::CloseBrace => {
                scope.pop();
            }
            Token::OpenBracket => {
                scope.push(
                    path.drain(..)
                        .chain([PathSegment::Array(0)])
                        .collect::<Vec<_>>(),
                );
            }
            Token::CloseBracket => {
                scope.pop();
            }
            _ => eprintln!("Unhandled token: {token:?}"),
        }
    }

    Ok(ret)
}

enum JsonEntry<'a> {
    Object(serde_json::map::Entry<'a>),
    Array(&'a mut Vec<Value>, usize),
}
impl<'a> JsonEntry<'a> {
    fn or_insert(self, x: Value) -> &'a mut Value {
        match self {
            JsonEntry::Object(entry) => entry.or_insert(x),
            JsonEntry::Array(arr, idx) => {
                if idx == arr.len() {
                    arr.push(x);
                }
                &mut arr[idx]
            }
        }
    }
}

pub fn merge(mut xs: Vec<(Path, PathTarget)>) -> anyhow::Result<Value> {
    use serde_json::{map::Entry, Map};
    xs.sort_by_key(|x| x.0.clone());
    let mut ret = Map::new();
    for (path, target) in xs {
        let mut segments = path.0.into_iter();
        let PathSegment::Object(first) = segments.next().unwrap() else {
            bail!("top-level must be an object");
        };
        let mut ptr = JsonEntry::Object(ret.entry(first));
        for segment in segments {
            match segment {
                PathSegment::Object(key) => {
                    let x = ptr.or_insert(Value::Object(Map::new()));
                    ptr = JsonEntry::Object(x.as_object_mut().unwrap().entry(key));
                }
                PathSegment::Array(idx) => {
                    let x = ptr.or_insert(Value::Array(Vec::new()));
                    let arr = x.as_array_mut().unwrap();
                    ptr = JsonEntry::Array(arr, idx);
                }
            }
        }
        match target {
            PathTarget::Value(target) => match ptr {
                JsonEntry::Object(Entry::Vacant(ptr)) => {
                    ptr.insert(target);
                }
                JsonEntry::Object(Entry::Occupied(ptr)) => {
                    ensure!(ptr.get() == &target);
                }
                JsonEntry::Array(arr, idx) => {
                    if idx == arr.len() {
                        arr.push(target);
                    } else {
                        ensure!(&arr[idx] == &target);
                    }
                }
            },
            PathTarget::Ref(_) => eprintln!("TODO"),
        }
    }
    Ok(Value::Object(ret))
}
