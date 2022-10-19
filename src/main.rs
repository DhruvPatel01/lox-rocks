use std::{env, process, fs};

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod scanner;
mod token;
mod loxerr;

fn run_prompt() {
    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").unwrap();

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                run(line.as_str());
            },
            Err(ReadlineError::Interrupted) => (), 
            Err(ReadlineError::Eof) => {
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}


fn run_file(file_name: &str) {
    let file = fs::read_to_string(file_name).expect("Error while reading the file");
    let ran_successfully = run(&file);
    if !ran_successfully {
        process::exit(65);
    }
}

fn run(line: &str) -> bool {
    let mut scanner = scanner::Scanner::new(line);
    let tokens = scanner.scan_tokens();
    
    for token in tokens {
        println!("{:#?}", token);
    }
    scanner.has_error
}


fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => run_prompt(),
        2 => run_file(&args[1]),
        _ => {println!("Usage: rlox [script_name]"); process::exit(64);}
    };

}
