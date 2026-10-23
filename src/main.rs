mod eval;
mod lexer;
mod parser;

use crate::parser::Parser;
use crate::{eval::Evaluator, lexer::Lexer};
use std::collections::HashMap;
use std::{env, fs, process};

struct Config {
    file: String,
    initial_arg: Option<i64>,
    print_tokens: bool,
    print_ast: bool,
}

fn run_program(
    program: &str,
    state: &mut HashMap<String, i64>,
    print_tokens: bool,
    print_ast: bool,
) -> Result<(), String> {
    let mut lexer = Lexer::new(program);
    let tokens = lexer.lex().map_err(|e| format!("{e:?}"))?;

    if print_tokens {
        println!("TOKENS:\n{tokens:?}");
    }

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse().map_err(|e| format!("{e:?}"))?;

    if print_ast {
        println!("AST\n{ast:#?}");
    }

    let mut evaluator = Evaluator::new(state);
    evaluator.execute_statement(&ast);
    println!("Terminal state:\n{:?}", state);

    Ok(())
}

fn parse_args() -> Result<Config, String> {
    let mut print_tokens = false;
    let mut print_ast = false;
    let mut positional: Vec<String> = Vec::new();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--print-tokens" => print_tokens = true,
            "--print-ast" => print_ast = true,
            _ => positional.push(arg),
        }
    }

    if !(1..=2).contains(&positional.len()) {
        return Err(
            "Usage: cargo run -- <file> [while_arg] [--print-tokens] [--print-ast]".to_string(),
        );
    }

    let file = positional[0].clone();
    let initial_arg = positional
        .get(1)
        .map(|s| s.parse::<i64>())
        .transpose()
        .map_err(|_| {
            format!(
                "Unable to parse {:?}. Argument must be an integer.",
                positional.get(1)
            )
        })?;

    Ok(Config {
        file,
        initial_arg,
        print_tokens,
        print_ast,
    })
}

fn main() {
    let config = parse_args().unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1);
    });

    let contents = fs::read_to_string(config.file).unwrap_or_else(|error| {
        eprintln!("{error}");
        process::exit(1);
    });

    let mut state: HashMap<String, i64> = HashMap::new();
    if let Some(arg) = config.initial_arg {
        state.insert(String::from("a"), arg);
    }

    run_program(&contents, &mut state, config.print_tokens, config.print_ast).unwrap_or_else(|e| {
        eprintln!("{e}");
    });
}
