use crate::{Path, PathSegment, PathTarget, Token};
use anyhow::anyhow;
use logos::Lexer;
use serde_json::Value;

pub fn parse<'a>(mut lexer: Lexer<'a, Token<'a>>) -> anyhow::Result<Vec<(Path, PathTarget)>> {
    let mut ret = vec![];
    let mut scope: Vec<Vec<PathSegment>> = vec![];
    let mut path = vec![];
    let mut prev = None;
    let mut reference = vec![];
    let mut seen_newline = false;

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

    while let Some(token) = lexer
        .next()
        .transpose()
        .map_err(|e| anyhow!("Lexer error: {e}"))?
    {
        if seen_newline && !matches!(token, Token::Colon | Token::Newline) {
            if let Some(token) = prev.take() {
                store_token!(token)
            }
        }
        seen_newline = false;
        match token {
            Token::Comment => {
                let rem = lexer.remainder();
                let n = rem.find('\n').unwrap_or_else(|| rem.len());
                lexer.bump(n); // leave the newline
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
                x => panic!("{x:?}"),
            },
            Token::Period => match prev.take() {
                Some(Token::String(x)) => reference.push(PathSegment::Object(x.to_string())),
                _ => panic!(),
            },
            Token::Newline => seen_newline = true,
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
                if let Some(token) = prev.take() {
                    store_token!(token)
                }
                scope.pop();
            }
        }
    }

    if let Some(token) = prev.take() {
        store_token!(token)
    }

    Ok(ret)
}
