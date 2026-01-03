mod lexer;
mod parser;

use crate::lexer::Lexer;
use crate::parser::Parser;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- \"<input>\"");
        process::exit(1);
    }

    let mut lexer = Lexer::new(&args[1]);
    let tokens = match lexer.lex() {
        Ok(tokens) => {
            println!("{tokens:?}");
            tokens
        }
        Err(e) => {
            eprintln!("{e:?}");
            process::exit(1);
        }
    };

    let mut parser = Parser::new(&tokens);
    match parser.parse() {
        Ok(tree) => println!("{tree:?}"),
        Err(e) => {
            eprintln!("{e}");
            process::exit(1);
        }
    }
}
