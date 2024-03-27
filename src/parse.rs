use crate::{Path, PathSegment, PathTarget, Token};
use anyhow::{anyhow, ensure};
use logos::Lexer;
use tracing::*;

pub fn parse<'a>(mut lexer: Lexer<'a, Token<'a>>) -> anyhow::Result<Vec<(Path, PathTarget)>> {
    let mut ret = vec![];
    let mut scope: Vec<Vec<PathSegment>> = vec![];
    let mut path = vec![];
    let mut prev = None;
    let mut seen_newline = false;
    let mut scope_is_empty = true;

    macro_rules! store_token {
        ($token: expr) => {{
            match $token {
                Token::String(x) => {
                    store_value!(PathTarget::String(x.to_string()))
                }
                Token::Null => store_value!(PathTarget::Null),
                Token::Bool(x) => store_value!(PathTarget::Bool(x)),
                Token::Int(x) => store_value!(PathTarget::Int(x)),
                Token::Float(x) => store_value!(PathTarget::Float(x)),
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
                $value,
            ));
            path.clear();
            scope_is_empty = false;
        }};
    }

    while let Some(token) = lexer
        .next()
        .transpose()
        .map_err(|e| anyhow!("Lexer error: {e:?}\n{}", lexer.slice()))?
    {
        debug!("{token:?}");
        if seen_newline && !matches!(token, Token::Colon | Token::Newline) {
            if let Some(token) = prev.take() {
                store_token!(token)
            }
        }
        seen_newline = false;
        match token {
            Token::Comment => {
                let rem = lexer.remainder();
                let n = rem.find('\n').unwrap_or(rem.len());
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
            Token::Newline => seen_newline = true,
            Token::Comma => {
                if let Some(token) = prev.take() {
                    store_token!(token)
                }
                if let Some(PathSegment::Array(x)) = scope.last_mut().and_then(|x| x.last_mut()) {
                    *x += 1;
                }
            }
            Token::OpenBrace => {
                scope.push(path.drain(..).collect::<Vec<_>>());
                scope_is_empty = true;
            }
            Token::CloseBrace => {
                if let Some(token) = prev.take() {
                    store_token!(token)
                }
                if scope_is_empty {
                    store_value!(PathTarget::EmptyObject);
                }
                scope.pop();
            }
            Token::OpenBracket => {
                scope.push(
                    path.drain(..)
                        .chain([PathSegment::Array(0)])
                        .collect::<Vec<_>>(),
                );
                scope_is_empty = true;
            }
            Token::CloseBracket => {
                if let Some(token) = prev.take() {
                    store_token!(token)
                }
                if scope_is_empty {
                    let x = scope.last_mut().and_then(|x| x.pop());
                    ensure!(x == Some(PathSegment::Array(0)));
                    store_value!(PathTarget::EmptyArray);
                }
                scope.pop();
            }
        }
    }

    if let Some(token) = prev.take() {
        store_token!(token);
        if scope_is_empty {
            store_value!(PathTarget::EmptyObject);
            assert!(!scope_is_empty);
        }
    }

    Ok(ret)
}
