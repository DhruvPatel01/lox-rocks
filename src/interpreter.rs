use std::cell::RefCell;

use crate::expr::{Expr, Value};
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};
use crate::loxerr::RuntimeError;
use crate::env::Environment;
use Value::*;
use TokenType::*;


pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(value) => *value,
        _ => true,
    }
}

fn err_numeric_operand(token: &Token) -> Result<Value, RuntimeError> {
    Err(RuntimeError{token: token.clone(), error: "operand must be a number.".to_owned()})
}

fn err_numstr_operand(token: &Token) -> Result<Value, RuntimeError> {
    Err(RuntimeError{token: token.clone(), error: "operand must be a numbers or strings.".to_owned()})
}

pub struct Interpreter{
    env: RefCell<Environment>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            env: RefCell::new(Environment::new())
        }
    }
    
    fn evaluate(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(val) => Ok(val.clone()),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Variable(token) => self.env.borrow().get(token),
            Expr::Assign(token, expr) => {
                let val = self.evaluate(expr)?;
                self.env.borrow_mut().assign(token, val.clone())?;
                Ok(val)
            }
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
                        (Value::String(ref l), Value::String(ref r)) => Ok(Value::String(format!("{}{}", l, r))),
                        _ => err_numstr_operand(op)
                    }
                    _ => unreachable!(),
    
                } 
            }
        }
    }

    fn execute(&self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Expression(e) => {
                self.evaluate(e)?;
            }

            Stmt::Print(e) => {
                let res = self.evaluate(e)?;
                println!("{}", res);
            }

            Stmt::Var(token, init) => {
                let value = init
                    .as_ref()
                    .map(|x| self.evaluate(x))
                    .unwrap_or(Ok(Value::Nil))?;
                self.env.borrow_mut().define(&token.lexeme, value);
            }       
            
            Stmt::Null => (),
        };
        Ok(())
    }
    

    pub fn interpret(&self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            match self.execute(stmt) {
                Err(e) => {e.error(); break;},
                _ => (),
            }
        }
    }
}



