mod lexer;

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    let file_content: String = std::fs::read_to_string(args.file).expect("File not found");
    let tokens: Vec<lexer::Token> = lexer::Lexer::new(&file_content).tokenize();
    println!("{:#?}", tokens);
    Ok(())
}
