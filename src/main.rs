use cuebasic::*;
use logos::Logos;
use std::path::PathBuf;

#[derive(clap::Parser)]
struct Opts {
    file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let opts = <Opts as clap::Parser>::parse();
    let file = std::fs::read_to_string(opts.file)?;
    let tree = parse(Token::lexer(&file))?;
    for (path, target) in &tree {
        eprintln!("{path}: {target}");
    }
    let value = merge(tree)?;
    println!("{value}");
    Ok(())
}
