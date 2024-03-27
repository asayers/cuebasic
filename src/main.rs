use bpaf::{Bpaf, Parser};
use cuebasic::*;
use logos::Logos;
use std::path::PathBuf;

#[derive(Bpaf)]
struct Opts {
    tokens: bool,
    unmerged: bool,
    last_write_wins: bool,
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
