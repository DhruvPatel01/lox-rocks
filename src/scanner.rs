use crate::loxerr;
use crate::token::{Token, TokenType};
use TokenType::*;

pub struct Scanner<'a> {
    source: &'a str,
    chars: Vec<char>,
    source_current: usize,
    chars_current: usize,
    tokens: Vec<Token>,
    start: usize,
    line: usize,
    pub has_error: bool,
}

fn keyword_or_identifier(lexeme: &str) -> TokenType {
    match lexeme {
        "and" => And,
        "class" => Class,
        "else" => Else,
        "false" => False,
        "for" => For,
        "fun" => Fun,
        "if" => If,
        "nil" => Nil,
        "or" => Or,
        "print" => Print,
        "return" => Return,
        "super" => Super,
        "this" => This,
        "true" => True,
        "var" => Var,
        "while" => While,
        _ => Identifier,
    }
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Scanner {
        Scanner {
            source,
            source_current: 0,
            chars: source.chars().collect(),
            chars_current: 0,
            tokens: Vec::new(),
            start: 0, // in bytes, not unicode code points
            line: 1,
            has_error: false,
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        while !self.is_at_end() {
            self.start = self.source_current;
            self.scan_token();
        }

        self.tokens.push(Token::new(Eof, "", self.line));

        &self.tokens
    }

    fn is_at_end(&self) -> bool {
        self.source_current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.chars_current];
        self.chars_current += 1;
        self.source_current += c.len_utf8();
        c
    }

    fn is_match(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.chars[self.chars_current] != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.chars_current]
        }
    }

    fn peek_next(&self) -> char {
        if self.chars_current + 1 >= self.chars.len() {
            '\0'
        } else {
            self.chars[self.chars_current + 1]
        }
    }

    fn scan_token(&mut self) {
        let c = self.advance();
        match c {
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),
            '<' => {
                let token = if self.is_match('=') { LessEqual } else { Less };
                self.add_token(token);
            }
            '!' => {
                let token = if self.is_match('=') { BangEqual } else { Bang };
                self.add_token(token);
            }
            '>' => {
                let token = if self.is_match('=') {
                    GreaterEqual
                } else {
                    Greater
                };
                self.add_token(token);
            }
            '=' => {
                let token = if self.is_match('=') {
                    EqualEqual
                } else {
                    Equal
                };
                self.add_token(token);
            }
            '/' => {
                if self.is_match('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(Slash)
                }
            }
            '\t' | '\r' | ' ' => {}
            '\n' => self.line += 1,
            '"' => self.string(),
            '0'..='9' => self.number(),
            _ => {
                if is_alpha(c) {
                    self.identifier();
                } else {
                    self.error(&format!("Unexpected character {}", c))
                }
            }
        }
    }

    fn add_token(&mut self, token_type: TokenType) {
        let lexeme = &self.source[self.start..self.source_current];
        let t = Token {
            token_type,
            lexeme: lexeme.to_owned(),
            line: self.line,
        };
        self.tokens.push(t);
    }

    fn error(&mut self, msg: &str) {
        loxerr::error(self.line, msg);
        self.has_error = true;
    }

    fn string(&mut self) {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.error("Unterminated string.");
            return;
        }

        self.advance(); //swallow the terminating "
        let s = &self.source[self.start + 1..self.source_current - 1];
        self.add_token(StringLiteral(s.to_owned()));
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let lexeme = &self.source[self.start..self.source_current];
        let number = lexeme.parse::<f64>().unwrap();
        self.add_token(Number(number));
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let lexeme = &self.source[self.start..self.source_current];
        self.add_token(keyword_or_identifier(lexeme));
    }
}

#[inline]
fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

#[inline]
fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

#[inline]
fn is_alpha_numeric(c: char) -> bool {
    is_alpha(c) || is_digit(c)
}
