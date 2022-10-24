use crate::expr::{Expr, Value};
use crate::token::{Token, TokenType};
use TokenType::*;
use crate::loxerr::{self, ParseError};

pub struct Parser<'a> {
    current: usize,
    tokens: &'a [Token],
    pub has_error: bool,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser { current: 0, tokens, has_error: false}
    }

    pub fn parse(&mut self) -> Option<Expr> {
        self.expression().ok()
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, token_type: &TokenType) -> bool {
        // TODO: this does not work for string, number types.
        !self.is_at_end() && self.peek().token_type == *token_type
    }

    fn consume(&mut self, typ: TokenType, msg: &'a str) -> Result<&Token, ParseError> {
        if self.check(&typ) {Ok(self.advance())}
        else {
            loxerr::parse_error(&self.peek(), msg);
            self.has_error = true;
            Err(ParseError)
        }

    }

    fn is_match(&mut self, types: &[TokenType]) -> bool {
        for token in types {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().token_type == Semicolon {
                return;
            }

            match self.peek().token_type {
                Class | Fun | Var | For | If | While |
                Print | Return => return,
                _ => {self.advance();}
            }
        }
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.is_match(&vec![BangEqual, EqualEqual]) {
            let op = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.is_match(&vec![Greater, GreaterEqual, Less, LessEqual]) {
            let op = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.is_match(&vec![Plus, Minus]) {
            let op = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;
        while self.is_match(&vec![Slash, Star]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.is_match(&vec![Bang, Minus]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(op, Box::new(right)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match &self.peek().token_type {
            True => {
                self.advance();
                Ok(Expr::Literal(Value::Bool(true)))
            }
            False => {
                self.advance();
                Ok(Expr::Literal(Value::Bool(false)))
            }
            Nil => {
                self.advance();
                Ok(Expr::Literal(Value::Nil))
            }
            Number(x) => {
                let l = Expr::Literal(Value::Number(*x));
                self.advance();
                Ok(l)
            }
            StringLiteral(x) => {
                let l = Expr::Literal(Value::String(x.clone()));
                self.advance();
                Ok(l)
            }
            LeftParen => {
                self.advance();
                let e = self.expression()?;
                self.consume(RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Box::new(e)))
            }
            _ => todo!()
        }
    }
}
