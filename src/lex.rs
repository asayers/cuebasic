use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t]+")]
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
    Int(i64),
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
    #[regex(r"(_?#)?([a-zA-Z_$])([a-zA-Z_$0-9])*")]
    Ident(&'a str),

    #[token("//")]
    Comment,
    #[regex(r"[\n\r]+")]
    Newline,
    /*
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

fn from_decimal<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Option<u64> {
    lexer.slice().parse().ok()
}
fn from_si<'a>(lexer: &mut Lexer<'a, Token<'a>>) -> Option<u64> {
    let xs = lexer.slice();
    let (head, tail) = xs.split_at(xs.len() - if xs.ends_with('i') { 2 } else { 1 });
    let man: f64 = head.parse().ok()?;
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
        x => panic!("Bad SI suffix: {x}"),
    };
    Some((man * exp) as u64)
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
