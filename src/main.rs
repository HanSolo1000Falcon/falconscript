mod lexer;
mod parser;
mod interpreter;

use std::io::{Stdout, Write};
use clap::Parser;
use crate::interpreter::Interpreter;
use crate::lexer::{Lexer, Token};
use crate::parser::Node;

#[derive(Debug, Parser)]
struct Args {
    file: String,
    program_args: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let file_content: String = std::fs::read_to_string(args.file).expect("File not found");
    let tokens: Vec<Token> = Lexer::new(&file_content).tokenize();
    let nodes: Vec<Node> = parser::Parser::new(tokens).parse();
    let mut interpreter: Interpreter<Stdout> = Interpreter::new(nodes, std::io::stdout());
    interpreter.run();
    Ok(())
}
