use crate::expr::{Expr, Value};
use crate::stmt::Stmt;
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

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.declaration());
        }
        stmts
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

    fn declaration(&mut self) -> Stmt {
        let res = if self.is_match(&[Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match res {
            Ok(s) => s,
            Err(_e) => {
                self.synchronize();
                Stmt::Null
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume(Identifier, "Expect variable name.")?.clone();
        let init = if self.is_match(&[Equal]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(Semicolon, "Expect ';' after variable declaration.")?;

        Ok(Stmt::Var(name, init))
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.is_match(&[If]) {
            self.if_statement()
        } else if self.is_match(&[Print]) {
            self.print_statement()
        } else if self.is_match(&[LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else if self.is_match(&[While]) {
            self.while_statement()
        } else if self.is_match(&[For]) {
            self.for_statement()  
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(LeftParen, "Expect '(' after 'if'.")?;
        let expr = self.expression()?;
        self.consume(RightParen, "Expect ')' after if condition.")?;

        let if_stmt = Box::new(self.statement()?);

        let else_stmt = if self.is_match(&[Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If(expr, if_stmt, else_stmt))
    }

    fn print_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(Semicolon, &"Expect ';' after value.")?;
        Ok(Stmt::Print(expr))
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ')' after condition.")?;

        let body = self.statement()?;
        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(LeftParen,  "Expect '(' after 'for'.")?;

        let initializer = if self.is_match(&[Semicolon]) {
            Stmt::Null
        } else if self.is_match(&[Var]) {
            self.var_declaration()?
        } else {
            self.expression_statement()?
        };

        let condition = if !self.check(&Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Value::Bool(true))
        };
        self.consume(Semicolon, "Expect ';' after loop condition.")?;

        let increment = if !self.check(&RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(RightParen, "Expect ')' after for clauses.")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        body = Stmt::While(condition, Box::new(body));
        body = Stmt::Block(vec![initializer, body]);

        return Ok(body);
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(Semicolon, &"Expect ';' after expression.")?;
        Ok(Stmt::Expression(expr))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();

        while !self.check(&RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration());
        }

        self.consume(RightBrace, "Expect '}' after block.")?;
        Ok(stmts)
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.is_match(&[Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;
            
            if let Expr::Variable(t) = expr {
                return Ok(Expr::Assign(t, Box::new(value)));
            }

            loxerr::parse_error(&equals, "Invalid assignment target.");
            self.has_error = true;
            return Err(ParseError);
        }
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.and()?;
        while self.is_match(&[Or]) {
            let op = self.previous().clone();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.equality()?;
        while self.is_match(&[And]) {
            let op = self.previous().clone();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.is_match(&[BangEqual, EqualEqual]) {
            let op = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.term()?;
        while self.is_match(&[Greater, GreaterEqual, Less, LessEqual]) {
            let op = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;
        while self.is_match(&[Plus, Minus]) {
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

        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        loop {
            if self.is_match(&[LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut args = vec![];

        if !self.check(&RightParen) {
            loop {
                if args.len() >= 255 {
                    loxerr::parse_error(self.peek(), "Can't have more than 255 arguments.");
                    self.has_error = true;
                }
                args.push(self.expression()?);
                if !self.is_match(&[Comma]) {
                    break;
                }
            }
        }

        let paren = self.consume(RightParen, "Expect ')' after arguments.")?;

        Ok(Expr::Call(Box::new(callee), paren.clone(), args))

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
            Identifier => {
                self.advance();
                Ok(Expr::Variable(self.previous().clone()))
            }
            LeftParen => {
                self.advance();
                let e = self.expression()?;
                self.consume(RightParen, "Expect ')' after expression.")?;
                Ok(Expr::Grouping(Box::new(e)))
            }
            _ => {
                self.has_error = true;
                loxerr::parse_error(self.peek(), "Expect expression.");
                Err(ParseError)
            }
        }
    }
}
