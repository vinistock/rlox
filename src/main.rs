use std::io::Write;

use crate::ast::Node;
use scanner::Scanner;

mod ast;
mod parser;
mod scanner;
mod token;
mod visitor;

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

    let tokens = {
        let mut scanner = Scanner::new(&code, &mut errors);
        scanner.scan();
        scanner.into_tokens()
    };

    if !errors.is_empty() {
        for error in &errors {
            eprintln!("Error: {}", error);
        }
        std::process::exit(1);
    }

    let mut parser = parser::Parser::new(tokens, &mut errors);
    let expression = parser.parse();

    if errors.is_empty() {
        println!("=> {}", expression.accept(&visitor::AstPrinter));
    } else {
        for error in errors {
            eprintln!("Error: {}", error);
        }
    }
}
