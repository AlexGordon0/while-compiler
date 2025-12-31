mod lexer;

use crate::lexer::Lexer;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Expected argument");
    }
    let mut lexer = Lexer::new(args[1].clone());
    let tokens = lexer.lex();
    println!("{:?}", tokens);
}
