use crate::{Path, PathSegment, PathTarget, Token};
use anyhow::{anyhow, bail};
use logos::Lexer;
use serde_json::Value;
use tracing::*;

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
                debug!("package {x}");
            }
            Token::Import => {
                // TODO: Outside the preamble, this should be converted into an identifier
                next_token!(Token::Ident(x));
                next_token!(Token::Newline);
                debug!("import {x}");
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
                if let Some(token) = prev.take() {
                    store_token!(token)
                }
                scope.pop();
            }
            _ => error!("Unhandled token: {token:?}"),
        }
    }

    Ok(ret)
}
