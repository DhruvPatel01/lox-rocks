use crate::token::{Token, TokenType};
use crate::expr::Value;

fn report(line: usize, wher: &str, msg: &str) {
    eprintln!("[line {}] Error{}: {}", line, wher, msg);
}

pub fn error(line: usize, msg: &str) {
    report(line, "", msg);
}

pub struct ParseError;
    

pub enum RuntimeException {
    RuntimeError {
        token: Token,
        error: String,
    },
    Return(Value),
}

pub fn parse_error(token: &Token, msg: &str) {
    if matches!(token.token_type, TokenType::Eof) {
        report(token.line, " at end", msg);
    } else {
        report(token.line, &format!(" at '{}'", token.lexeme), msg);
    }
}

impl RuntimeException {
    pub fn error(&self) {
        match &self {
            RuntimeException::RuntimeError { token, error } => {
                eprintln!("{}\n[line {}]", error, token.line);
            }
            _ => unreachable!()
        }
       
    }
}
