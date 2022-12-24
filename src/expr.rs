use std::fmt;
use std::rc::Rc;

use crate::token::Token;
use crate::loxcallables::LoxCallable;

#[derive(Clone, Debug)]
pub enum Expr {
    Assign(Token, Rc<Expr>),
    Binary(Rc<Expr>, Token, Rc<Expr>),
    Call(Rc<Expr>, Token, Vec<Rc<Expr>>),
    Grouping(Rc<Expr>),
    Literal(Value),
    Logical(Rc<Expr>, Token, Rc<Expr>),
    Unary(Token, Rc<Expr>),
    Variable(Token),
}

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Nil,
    String(String),
    Callable(Rc<dyn LoxCallable>)
}

impl Value {
    pub fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            (Value::String(l), Value::String(r)) => l == r,
            _ => false
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Nil => write!(f, "nil"),
            Value::Callable(c) => write!(f, "{}", c),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Nil => write!(f, "nil"),
            Value::Callable(c) => write!(f, "{}", c),
        }
    }
}
