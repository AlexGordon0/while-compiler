mod lexer;

use crate::lexer::Lexer;
use std::{env, process};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- \"<input>\"");
        process::exit(1);
    }
    let mut lexer = Lexer::new(&args[1]);
    match lexer.lex() {
        Ok(tokens) => println!("{tokens:?}"),
        Err(e) => {
            eprintln!("{e:?}");
            process::exit(1);
        }
    };
}
