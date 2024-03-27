use bpaf::{Bpaf, Parser};
use cuebasic::*;
use logos::Logos;
use std::path::PathBuf;

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
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let opts = opts().to_options().run();
    let file = std::fs::read_to_string(opts.file)?;
    if opts.tokens {
        for token in Token::lexer(&file) {
            eprintln!("{token:?}");
        }
    }
    let tree = parse(Token::lexer(&file))?;
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
