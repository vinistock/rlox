use crate::ast::Node;
use scanner::scan;

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
        let mut input = String::new();
        print!("> ");
        match std::io::stdin().read_line(&mut input) {
            Ok(0) => break, // EOF
            Ok(_) => run(input),
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                break;
            }
        }
    }
}

fn run(code: String) {
    let mut errors: Vec<String> = Vec::new();
    let tokens = scan(&code).unwrap_or_else(|errors| {
        eprintln!("Error scanning code: {:?}", errors);
        std::process::exit(1);
    });

    let mut parser = parser::Parser::new(tokens, &mut errors);
    let expression = parser.parse();

    if errors.is_empty() {
        let printer = visitor::AstPrinter;
        println!("{}", expression.accept(&printer));
    } else {
        for error in errors {
            eprintln!("Error: {}", error);
        }
    }
}
