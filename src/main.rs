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
mod resolver;
mod env;
mod loxcallables;
mod class;
mod instance;

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
        process::exit(self.run(&file));
    }
    
    fn run(&mut self, line: &str) -> i32 {
        let mut scanner = scanner::Scanner::new(line);
        let tokens = scanner.scan_tokens();

        let mut parser = parser::Parser::new(&tokens);
        let stmts = parser.parse();
        
        if parser.has_error || scanner.has_error {
            return 65;
        } 

        let mut resolver = resolver::Resolver::new(&mut self.interpreter);
        resolver.resolve(&stmts);
        if resolver.has_error {
            return 65;
        }

        if let Err(e) = self.interpreter.interpret(&stmts) {
            e.error();
            return 70;
        }
        0
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
