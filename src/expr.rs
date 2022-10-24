use crate::token::Token;

pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Value),
    Unary(Token, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Nil,
    String(String),
}
