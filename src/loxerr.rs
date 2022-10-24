use crate::token::{Token, TokenType};

fn report(line: usize, wher: &str, msg: &str) {
    println!("[line {}] Error {} where: {}", line, wher, msg);
}

pub fn error(line: usize, msg: &str) {
    report(line, "", msg);
}

pub struct ParseError;

pub fn parse_error(token: &Token, msg: &str) {
    if matches!(token.token_type, TokenType::Eof) {
        error(token.line, &format!(" at end {}", msg));
    } else {
        error(token.line, &format!(" at '{}' {}", token.lexeme, msg));
    }
}