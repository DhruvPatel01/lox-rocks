use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::env::Environment;
use crate::expr::{Expr, Value};
use crate::loxcallables::{Native, self};
use crate::loxerr::RuntimeError;
use crate::stmt::Stmt;
use crate::token::{Token, TokenType};
use TokenType::*;
use Value::*;

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(value) => *value,
        _ => true,
    }
}

fn err_numeric_operand(token: &Token) -> Result<Value, RuntimeError> {
    Err(RuntimeError {
        token: token.clone(),
        error: "Operands must be numbers.".to_owned(),
    })
}

fn err_numstr_operand(token: &Token) -> Result<Value, RuntimeError> {
    Err(RuntimeError {
        token: token.clone(),
        error: "Operands must be two numbers or two strings.".to_owned(),
    })
}

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(Environment::new()));
        global.borrow_mut().define(
            "clock",
            Value::Callable(Rc::new(Native::new(0, |_| {
                Ok(Value::Number(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Could not retrieve time.")
                        .as_millis() as f64,
                ))
            }))),
        );

        Interpreter {
            globals: Rc::clone(&global),
            env: Rc::clone(&global),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(val) => Ok(val.clone()),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Variable(token) => self.env.borrow().get(token),
            Expr::Assign(token, expr) => {
                let val = self.evaluate(expr)?;
                self.env.borrow_mut().assign(token, val.clone())?;
                Ok(val)
            }
            Expr::Call(callee, paren, args) => {
                let callee = self.evaluate(callee)?;
                let mut args_evaluated = Vec::new();
                for arg in args {
                    args_evaluated.push(self.evaluate(arg)?);
                }

                match callee {
                    Value::Callable(callee) => {
                        if args_evaluated.len() != callee.arity() {
                            Err(RuntimeError {
                                token: paren.clone(),
                                error: format!(
                                    "Expected {} arguments but got {}.",
                                    callee.arity(),
                                    args_evaluated.len()
                                ),
                            })
                        } else {
                            callee.call(self, &args_evaluated)
                        }
                    }
                    _ => Err(RuntimeError {
                        token: paren.clone(),
                        error: "Can only call functions and classes.".to_owned(),
                    }),
                }
            }
            Expr::Unary(op, expr) => {
                let rhs = self.evaluate(expr)?;
                match op.token_type {
                    Bang => Ok(Bool(!is_truthy(&rhs))),
                    Minus => match rhs {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(RuntimeError {
                            token: op.clone(),
                            error: "Operand must be a number.".to_owned(),
                        }),
                    },
                    _ => unreachable!(),
                }
            }
            Expr::Logical(e1, op, e2) => {
                let left = self.evaluate(e1)?;
                match op.token_type {
                    Or if is_truthy(&left) => Ok(left),
                    And if !is_truthy(&left) => Ok(left),
                    _ => self.evaluate(e2),
                }
            }
            Expr::Binary(e1, op, e2) => {
                let l = self.evaluate(e1)?;
                let r = self.evaluate(e2)?;
                match op.token_type {
                    EqualEqual => Ok(Value::Bool(l.eq(&r))),
                    BangEqual => Ok(Value::Bool(!l.eq(&r))),
                    Greater => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l > r)),
                        _ => err_numeric_operand(op),
                    },
                    GreaterEqual => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l >= r)),
                        _ => err_numeric_operand(op),
                    },
                    Less => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l < r)),
                        _ => err_numeric_operand(op),
                    },
                    LessEqual => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Bool(l <= r)),
                        _ => err_numeric_operand(op),
                    },
                    Minus => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l - r)),
                        _ => err_numeric_operand(op),
                    },
                    Slash => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l / r)),
                        _ => err_numeric_operand(op),
                    },
                    Star => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l * r)),
                        _ => err_numeric_operand(op),
                    },
                    Plus => match (l, r) {
                        (Value::Number(l), Value::Number(r)) => Ok(Value::Number(l + r)),
                        (Value::String(ref l), Value::String(ref r)) => {
                            Ok(Value::String(format!("{}{}", l, r)))
                        }
                        _ => err_numstr_operand(op),
                    },
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn execute_block(&mut self, stmts: &Vec<Stmt>, env: Environment) -> Result<(), RuntimeError> {
        let new_env = Rc::new(RefCell::new(env));
        let old_env = std::mem::replace(&mut self.env, new_env);

        for stmt in stmts {
            if let Err(e) = self.execute(stmt) {
                self.env = old_env;
                return Err(e);
            }
        }
        self.env = old_env;
        Ok(())
    }

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::If(expr, if_part, else_part) => {
                if is_truthy(&self.evaluate(expr)?) {
                    self.execute(if_part)?;
                } else if let Some(else_part) = else_part {
                    self.execute(else_part)?;
                }
            }
            Stmt::Expression(e) => {
                self.evaluate(e)?;
            }

            Stmt::While(condition, body) => {
                while is_truthy(&self.evaluate(condition)?) {
                    self.execute(body)?;
                }
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

            Stmt::Block(stmts) => {
                self.execute_block(stmts, Environment::encloser(&self.env))?;
            }
            Stmt::Function(id, _, _) => {
                let fun = Rc::new(loxcallables::Function::new(stmt));
                self.env.borrow_mut().define(&id.lexeme, Value::Callable(fun))
            }
            Stmt::Null => (),
        };
        Ok(())
    }

    pub fn interpret(&mut self, stmts: &Vec<Stmt>) -> Result<(), RuntimeError> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }
}
