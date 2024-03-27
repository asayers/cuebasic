use bpaf::{Bpaf, Parser};
use cuebasic::*;
use logos::Logos;
use std::{io::Read, path::PathBuf};

#[derive(Bpaf)]
struct Opts {
    /// Lex the file and dump the tokens
    tokens: bool,
    /// Parse the file and dump the path->value mapping
    unmerged: bool,
    /// Don't error on duplicate fields with conflicting values
    last_write_wins: bool,
    /// The source file.  Reads from stdin if not specified
    #[bpaf(positional("PATH"))]
    file: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let opts = opts().to_options().run();
    let source = match opts.file {
        Some(path) => std::fs::read_to_string(path)?,
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            buf
        }
    };
    if opts.tokens {
        for token in Token::lexer(&source) {
            eprintln!("{token:?}");
        }
    }
    let tree = parse(Token::lexer(&source))?;
    if opts.unmerged {
        for (path, target) in &tree {
            eprintln!("{path}: {target}");
        }
    }
    if !opts.unmerged && !opts.tokens {
        let value = merge(tree, !opts.last_write_wins)?;
        println!("{value}");
    }
    Ok(())
}
