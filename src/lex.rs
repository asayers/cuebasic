use logos::{Lexer, Logos};
use num_bigint::{BigInt, ParseBigIntError};
use num_traits::{FromPrimitive, Num};
use scientific::Scientific;
use std::num::ParseFloatError;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t]+")]
#[logos(error = String)]
pub enum Token<'a> {
    // Punctuation
    #[token(":")]
    Colon,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,
    #[token(",")]
    Comma,

    // Literals
    #[token("null")]
    Null,
    #[token("true", |_| true)]
    #[token("false", |_| false)]
    Bool(bool),
    #[regex(r"-?0", from_decimal)]
    #[regex(r"-?[1-9]([0-9_])*", from_decimal)]
    #[regex(r"-?[0-9]([0-9_])*(\.[0-9]([0-9_])*)?[KMGTP]i?", from_si)]
    #[regex(r"-?\.[0-9][0-9]([0-9_])*[KMGTP]i?", from_si)]
    #[regex(r"0b[01][01_]*", from_binary)]
    #[regex(r"0o[0-7][0-7_]*", from_octal)]
    #[regex(r"0[xX][0-9a-fA-F][0-9a-fA-F_]*", from_hex)]
    Int(BigInt),
    #[regex(r"-?[0-9][0-9_]*\.([0-9][0-9_]*)?([eE][+-]?[0-9][0-9_]*)?", from_float)]
    #[regex(r"-?[0-9][0-9_]*[eE][+-]?[0-9][0-9_]*", from_float)]
    #[regex(r"-?\.[0-9][0-9_]*([eE][+-]?[0-9][0-9_]*)?", from_float)]
    Float(Scientific),
    // TODO: Multiline strings
    // TODO: Escape sequences
    // #[regex(r#""[^"]*""#, from_string)]
    #[regex("\"(?:[^\"\\\\]|\\\\.)*\"", from_string)]
    #[regex("'[^']*'", from_string)]
    String(&'a str),

    // TODO: Use the unicode Letter and Digit classes
    #[regex(r"(_?#)?([a-zA-Z_$])([a-zA-Z_$0-9])*")]
    Ident(&'a str),

    #[regex(r"[\n\r\f]")]
    #[regex(r"//[^\n\r\f]*[\n\r\f]")] // Comments are treated as newlines
    Newline,
    /*
    #[token(".")]
    Period,

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
    #[regex(r"__(#|_#)?([a-zA-Z_])([a-zA-Z_0-9])*")]
    Keyword(&'a str),
    #[token("@([a-zA-Z_])([a-zA-Z_0-9])*")]
    Attribute,

    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus, // Could also be unary negation
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
    TestLT, // Could be a binary test, or a unary range constructor
    #[token(">")]
    TestGT, // Could be a binary test, or a unary range constructor
    #[token("<=")]
    TestLE, // Could be a binary test, or a unary range constructor
    #[token(">=")]
    TestGE, // Could be a binary test, or a unary range constructor
    #[token("=")]
    Equals,
    #[token("?")]
    Optional,
    #[token("!")]
    Required,
    #[token("(")]
    OpenParens,
    #[token(")")]
    CloseParens,
    CloseBracket,
    #[token("_|_")]
    Bottom,
    #[token("...")]
    Elipsis,
    */
}

fn from_decimal<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Result<BigInt, String> {
    lexer
        .slice()
        .parse()
        .map_err(|x: ParseBigIntError| x.to_string())
}
fn from_si<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Result<BigInt, String> {
    let xs = lexer.slice();
    let (head, tail) = xs.split_at(xs.len() - if xs.ends_with('i') { 2 } else { 1 });
    let man: f64 = head.parse().map_err(|x: ParseFloatError| x.to_string())?;
    let exp = match tail {
        "K" => 10_f64.powi(3),
        "M" => 10_f64.powi(6),
        "G" => 10_f64.powi(9),
        "T" => 10_f64.powi(12),
        "P" => 10_f64.powi(15),
        "Ki" => 2_f64.powi(10),
        "Mi" => 2_f64.powi(20),
        "Gi" => 2_f64.powi(30),
        "Ti" => 2_f64.powi(40),
        "Pi" => 2_f64.powi(50),
        x => return Err(format!("Bad SI suffix: {x}")),
    };
    let x = man * exp;
    BigInt::from_f64(x).ok_or_else(|| format!("Not an integer: {x}"))
}
fn from_binary<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Result<BigInt, String> {
    let digits = &lexer.slice()[2..];
    BigInt::from_str_radix(digits, 2).map_err(|x: ParseBigIntError| x.to_string())
}
fn from_octal<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Result<BigInt, String> {
    let digits = &lexer.slice()[2..];
    BigInt::from_str_radix(digits, 8).map_err(|x: ParseBigIntError| x.to_string())
}
fn from_hex<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Result<BigInt, String> {
    let digits = &lexer.slice()[2..];
    BigInt::from_str_radix(digits, 16).map_err(|x: ParseBigIntError| x.to_string())
}
fn from_float<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Result<Scientific, String> {
    lexer
        .slice()
        .parse()
        .map_err(|x: scientific::ConversionError| format!("{x:?}"))
}
fn from_string<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> &'a str {
    let xs = lexer.slice();
    &xs[1..xs.len() - 1]
}
