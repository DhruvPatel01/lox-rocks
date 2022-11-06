use std::{fs, process};

use rustyline::error::ReadlineError;
use rustyline::Editor;

mod expr;
// mod ast_printer;
mod loxerr;
mod parser;
mod stmt;
mod scanner;
mod token;
mod interpreter;
mod env;

use crate::interpreter::Interpreter;

struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    fn new() -> Self {
        Lox{interpreter: Interpreter::new()}
    }

    fn run_prompt(&mut self) {
        let mut rl = Editor::<()>::new();
        rl.load_history("history.txt").unwrap();
    
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    rl.add_history_entry(line.as_str());
                    self.run(line.as_str());
                }
                Err(ReadlineError::Interrupted) => (),
                Err(ReadlineError::Eof) => break,
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        rl.save_history("history.txt").unwrap();
    }

    fn run_file(&mut self, file_name: &str) {
        let file = fs::read_to_string(file_name).expect("Error while reading the file");
        let ran_successfully = self.run(&file);
        if !ran_successfully {
            process::exit(65);
        }
    }
    
    fn run(&mut self, line: &str) -> bool {
        let mut scanner = scanner::Scanner::new(line);
        let tokens = scanner.scan_tokens();
        
        let mut parser = parser::Parser::new(&tokens);
        let stmts = parser.parse();
    
        self.interpreter.interpret(&stmts);
        true
        
    }
    
}




fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut lox = Lox::new();

    match args.len() {
        1 => lox.run_prompt(),
        2 => lox.run_file(&args[1]),
        _ => {
            println!("Usage: rlox [script_name]");
            process::exit(64);
        }
    };
}
