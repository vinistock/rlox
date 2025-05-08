use std::io::Write;

use crate::ast::Stmt;
use ast::Statement;
use scanner::Scanner;
use vm::Vm;

mod ast;
mod environment;
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
    let mut vm = Vm::new();
    let contents = match std::fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(err) => {
            eprintln!("Error reading file {}: {}", path, err);
            std::process::exit(1);
        }
    };
    run(contents, &None, &mut vm);
}

fn run_interactively(arg: Option<String>) {
    let mut vm = Vm::new();

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

                run(input, &arg, &mut vm);
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                break;
            }
        }
    }
}

fn run(code: String, arg: &Option<String>, vm: &mut Vm) {
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
    let statements = parse(tokens, errors);
    match arg {
        Some(arg) if arg == "--print-ast" => {
            let formatted = statements
                .iter()
                .map(|stmt| stmt.accept(&mut visitor::AstPrinter))
                .collect::<Vec<_>>()
                .join("\n");

            println!("=> {}", formatted);
            return;
        }
        _ => {}
    }

    for statement in statements {
        statement.accept(vm).unwrap_or_else(|err| {
            eprintln!("Runtime error: {}", err);
            std::process::exit(1);
        });
    }
}

fn parse(tokens: Vec<token::Token>, mut errors: Vec<String>) -> Vec<Statement> {
    let mut parser = parser::Parser::new(tokens, &mut errors);
    let statements = parser.parse();

    if !errors.is_empty() {
        for error in errors {
            eprintln!("Parse error: {}", error);
        }
        std::process::exit(1);
    }
    statements
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
