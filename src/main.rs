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
        Some(arg) if arg == "--help" => print_help(),
        Some(arg) if !arg.starts_with("--") => run_file(arg),
        Some(arg) => run_interactively(Some(arg)),
        None => run_interactively(None),
    }
}

fn print_help() {
    println!("Usage: [file_path] [--print-tokens | --print-ast]");
}

fn run_file(path: String) {
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Error reading file {}: {}", path, err);
            std::process::exit(1);
        }
    };
    run(contents, &None);
}

fn run_interactively(arg: Option<String>) {
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

                run(input, &arg);
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                break;
            }
        }
    }
}

fn run(code: String, arg: &Option<String>) {
    let mut errors: Vec<String> = Vec::new();

    // Scanning
    let tokens = scan(code, &mut errors);
    match arg {
        Some(arg) if arg == "--print-tokens" => {
            println!("{:?}", tokens);
            return;
        }
        _ => {}
    }

    // Parsing
    let expression = parse(tokens, errors);
    match arg {
        Some(arg) if arg == "--print-ast" => {
            println!("=> {}", expression.accept(&visitor::AstPrinter));
            return;
        }
        _ => {}
    }

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
