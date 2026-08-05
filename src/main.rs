mod lexer;
mod parser;

use clap::Parser;
use crate::lexer::Token;
use crate::lexer::Lexer;
use crate::parser::{Expr, Node};

#[derive(Debug, Parser)]
struct Args {
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let file_content: String = std::fs::read_to_string(args.file).expect("File not found");
    let tokens: Vec<Token> = Lexer::new(&file_content).tokenize();
    let nodes: Vec<Node> = parser::Parser::new(tokens).parse();
    println!("{:#?}", nodes);
    Ok(())
}
