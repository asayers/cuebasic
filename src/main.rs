use cuebasic::*;
use logos::Logos;
use std::path::PathBuf;

#[derive(clap::Parser)]
struct Opts {
    file: PathBuf,
    #[clap(long)]
    tokens: bool,
    #[clap(long)]
    unmerged: bool,
    #[clap(long)]
    last_write_wins: bool,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let opts = <Opts as clap::Parser>::parse();
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
