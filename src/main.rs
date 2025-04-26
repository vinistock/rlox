use scanner::scan;

mod ast;
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
    match scan(&code) {
        Ok(tokens) => {
            println!("{:?}", tokens);
        }
        Err(errors) => {
            eprintln!("Error scanning code: {:?}", errors);
        }
    }
}
