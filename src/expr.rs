use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use crate::class::LoxClass;
use crate::token::Token;
use crate::loxcallables::LoxCallable;
use crate::instance::LoxInstance;

#[derive(Clone, Debug)]
pub enum Expr {
    Assign(Token, Rc<Expr>),
    Binary(Rc<Expr>, Token, Rc<Expr>),
    Call(Rc<Expr>, Token, Vec<Rc<Expr>>),
    Get(Rc<Expr>, Token),
    Grouping(Rc<Expr>),
    Literal(Value),
    Logical(Rc<Expr>, Token, Rc<Expr>),
    Set(Rc<Expr>, Token, Rc<Expr>),
    Super(Token, Token),
    This(Token),
    Unary(Token, Rc<Expr>),
    Variable(Token),
}

#[derive(Clone)]
pub enum Value {
    Bool(bool),
    Number(f64),
    Nil,
    String(String),
    Callable(Rc<dyn LoxCallable>),
    Class(Rc<LoxClass>),
    Instance(Rc<RefCell<LoxInstance>>),
}

impl Value {
    pub fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Bool(l), Value::Bool(r)) => l == r,
            (Value::Number(l), Value::Number(r)) => l == r,
            (Value::Nil, Value::Nil) => true,
            (Value::String(l), Value::String(r)) => l == r,
            (Value::Class(l), Value::Class(r)) => {
                l.to_string() == r.to_string()
            }
            (Value::Callable(l), Value::Callable(r)) => l.to_string() == r.to_string(),
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
            Value::Class(c) =>  write!(f, "{}", c),
            Value::Instance(i) => write!(f, "{}", (**i).borrow())
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
            Value::Class(c) =>  write!(f, "{}", c),
            Value::Instance(i) => write!(f, "{}", (**i).borrow())
        }
    }
}
