use crate::token::{Token, TokenType};

fn report(line: usize, wher: &str, msg: &str) {
    eprintln!("[line {}] Error {} where: {}", line, wher, msg);
}

pub fn error(line: usize, msg: &str) {
    report(line, "", msg);
}

pub struct ParseError;
    
pub struct RuntimeError {
    pub token: Token,
    pub error: &'static str,
}

pub fn parse_error(token: &Token, msg: &str) {
    if matches!(token.token_type, TokenType::Eof) {
        error(token.line, &format!(" at end {}", msg));
    } else {
        error(token.line, &format!(" at '{}' {}", token.lexeme, msg));
    }
}

impl RuntimeError {
    pub fn error(&self) {
        eprintln!("{}\n[line {} ]", self.error, self.token.line);
    }
}
