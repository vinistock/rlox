use std::io::Write;

use crate::ast::Node;
use scanner::Scanner;
use vm::Vm;

mod ast;
mod parser;
mod scanner;
mod token;
mod visitor;
mod vm;

fn main() {
    let mut args = std::env::args();

    match args.nth(1) {
        Some(arg) => run_file(arg),
        None => run_interactively(),
    }
}

fn run_file(path: String) {
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Error reading file {}: {}", path, err);
            std::process::exit(1);
        }
    };
    run(contents);
}

fn run_interactively() {
    loop {
        print!("ilox> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();

        match std::io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let command = input.trim();
                if command == "exit" || command == "quit" {
                    break;
                }

                run(input);
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                break;
            }
        }
    }
}

fn run(code: String) {
    let mut errors: Vec<String> = Vec::new();
    let tokens = scan(code, &mut errors);
    let expression = parse(tokens, errors);

    // println!("=> {}", expression.accept(&visitor::AstPrinter));

    let result = expression.accept(&Vm);
    match result {
        Ok(value) => println!("=> {}", value),
        Err(err) => {
            eprintln!("Runtime error: {}", err);
            std::process::exit(1);
        }
    }
}

fn parse(tokens: Vec<token::Token>, mut errors: Vec<String>) -> ast::Expr {
    let mut parser = parser::Parser::new(tokens, &mut errors);
    let expression = parser.parse();

    if !errors.is_empty() {
        for error in errors {
            eprintln!("Parse error: {}", error);
        }
        std::process::exit(1);
    }
    expression
}

fn scan(code: String, errors: &mut Vec<String>) -> Vec<token::Token> {
    let tokens = {
        let mut scanner = Scanner::new(&code, errors);
        scanner.scan();
        scanner.into_tokens()
    };

    if !errors.is_empty() {
        for error in &*errors {
            eprintln!("Scanning error: {}", error);
        }
        std::process::exit(1);
    }

    tokens
}
