use crate::expr::{Expr, Value};
use crate::token::{Token, TokenType};
use crate::loxerr::RuntimeError;
use Value::*;
use TokenType::*;

pub type Result = std::result::Result<Value, RuntimeError>;

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(value) => *value,
        _ => true,
    }
}

fn err_numeric_operand(token: &Token) -> Result {
    Err(RuntimeError{token: token.clone(), error: "operand must be a number."})
}

fn err_numstr_operand(token: &Token) -> Result {
    Err(RuntimeError{token: token.clone(), error: "operand must be a numbers or strings."})
}

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Interpreter{}
    }
    
    fn evaluate(&self, expr: &Expr) -> Result {
        match expr {
            Expr::Literal(val) => Ok(val.clone()),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Unary(op, expr) => {
                let rhs = self.evaluate(expr)?;
                match op.token_type {
                    Bang => Ok(Bool(!is_truthy(&rhs))),
                    Minus => match rhs {
                            Value::Number(n) => Ok(Value::Number(-n)),
                            _ => err_numeric_operand(op),
                        }
                    _ => unreachable!(),
                }
            }
            Expr::Binary(e1, op, e2) => {
                let l = self.evaluate(e1)?;
                let r = self.evaluate(e2)?;
                match op.token_type {
                    EqualEqual => Ok(Value::Bool(l == r)),
                    BangEqual => Ok(Value::Bool(l != r)),
                    Greater => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                        _ => err_numeric_operand(op)
                    }
                    GreaterEqual => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                        _ => err_numeric_operand(op)
                    }
                    Less => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                        _ => err_numeric_operand(op)
                    }
                    LessEqual => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                        _ => err_numeric_operand(op)
                    }
                    Minus => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        _ => err_numeric_operand(op)
                    }
                    Slash => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                        _ => err_numeric_operand(op)
                    }
                    Star => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        _ => err_numeric_operand(op)
                    }
                    Plus => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(ref l), Value::String(ref r)) => Ok(Value::String(format!("{} {}", l, r))),
                        _ => err_numstr_operand(op)
                    }
                    _ => unreachable!(),
    
                } 
            }
        }
    }

    pub fn interpret(&self, expr: &Expr) -> bool {
        match self.evaluate(expr) {
            Err(e) => {
                e.error();
                false
            }
            Ok(e) => {
                println!("{}", e);
                true
            }
        }
    }
}



