use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::class;
use crate::env::Environment;
use crate::expr::{Expr, Value};
use crate::loxcallables::LoxCallable;
use crate::loxcallables::{self, Native};
use crate::loxerr::RuntimeException;
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

fn gen_err(token: &Token, msg: &str) -> RuntimeException {
    RuntimeException::RuntimeError {
        token: token.clone(),
        error: msg.to_owned(),
    }
}

fn err_numeric_operand(token: &Token) -> Result<Value, RuntimeException> {
    Err(gen_err(token, "Operands must be numbers."))
}

fn err_numstr_operand(token: &Token) -> Result<Value, RuntimeException> {
    Err(gen_err(
        token,
        "Operands must be two numbers or two strings.",
    ))
}

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    locals: HashMap<*const Expr, usize>,
    env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let global = Rc::new(RefCell::new(Environment::new()));
        (*global).borrow_mut().define(
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
            locals: HashMap::new(),
        }
    }

    pub fn resolve(&mut self, expr: &Rc<Expr>, idx: usize) {
        let expr_ref: *const Expr = &**expr;
        // println!("Inserting express {:?} @ {:?} @@ {}", expr, expr_ref, idx);
        self.locals.insert(expr_ref, idx);
    }

    fn lookup_variable(&self, name: &Token, expr: &Rc<Expr>) -> Result<Value, RuntimeException> {
        let key: *const Expr = &**expr;

        if let Some(dist) = self.locals.get(&key) {
            // println!("Resolving {:?} @ {:?} @@ {}", expr, key, dist);
            self.env.borrow().get_at(*dist, name)
        } else {
            // println!("Resolving {:?} @ {:?} @@ GLOBAL", expr, key);
            self.globals.borrow().get(name)
        }
    }

    fn evaluate(&mut self, expr: &Rc<Expr>) -> Result<Value, RuntimeException> {
        match &**expr {
            Expr::Literal(val) => Ok(val.clone()),
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Variable(token) => self.lookup_variable(token, expr),
            Expr::Assign(token, right_expr) => {
                let val = self.evaluate(right_expr)?;
                let key: *const Expr = &**expr;
                if let Some(dist) = self.locals.get(&key) {
                    (*self.env)
                        .borrow_mut()
                        .assign_at(*dist, &token, val.clone());
                } else {
                    (*self.globals).borrow_mut().assign(token, val.clone())?;
                }

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
                            Err(gen_err(
                                paren,
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    callee.arity(),
                                    args_evaluated.len()
                                ),
                            ))
                        } else {
                            callee.call(self, &args_evaluated)
                        }
                    }
                    Value::Class(callee) => {
                        if args_evaluated.len() != callee.arity() {
                            Err(gen_err(
                                paren,
                                &format!(
                                    "Expected {} arguments but got {}.",
                                    callee.arity(),
                                    args_evaluated.len()
                                ),
                            ))
                        } else {
                            callee.call(self, &args_evaluated)
                        }
                    }
                    _ => Err(gen_err(paren, "Can only call functions and classes.")),
                }
            }

            Expr::Get(object, field) => {
                let object = self.evaluate(object)?;
                if let Value::Instance(ref instance) = object {
                    Ok(instance.borrow().get(field)?)
                } else {
                    Err(RuntimeException::RuntimeError {
                        token: field.clone(),
                        error: "Only instances have properties.".to_owned(),
                    })
                }
            }

            Expr::Unary(op, expr) => {
                let rhs = self.evaluate(expr)?;
                match op.token_type {
                    Bang => Ok(Bool(!is_truthy(&rhs))),
                    Minus => match rhs {
                        Value::Number(n) => Ok(Value::Number(-n)),
                        _ => Err(gen_err(op, "Operand must be a number.")),
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

            Expr::Set(object, name, value) => {
                let object = self.evaluate(object)?;
                if let Value::Instance(instance) = object {
                    let value = self.evaluate(value)?;
                    (*instance).borrow_mut().set(name, value.clone());
                    Ok(value)
                } else {
                    Err(RuntimeException::RuntimeError {
                        token: name.clone(),
                        error: "Only instances have fields.".to_owned(),
                    })
                }
            }

            Expr::Super(keyword, identifier) => {
                let expr_ref: *const Expr = &**expr;
                let dist = *self.locals.get(&expr_ref).unwrap();
                let superclass = self.env.borrow_mut().get_at(dist, keyword)?;

                let dummy_token = Token {
                    token_type: This,
                    lexeme: "this".to_owned(),
                    line: 0,
                };
                let this = self.env.borrow().get_at(dist - 1, &dummy_token)?;

                if let Value::Class(superclass) = superclass {
                    let method = superclass.find_method(&identifier.lexeme);
                    if let None = method {
                        return Err(RuntimeException::RuntimeError {
                            token: identifier.clone(),
                            error: format!("Undefined property '{}'.", identifier.lexeme),
                        });
                    }
                    let method = method.unwrap().bind(this);
                    return Ok(Value::Callable(Rc::new(method)));
                } else {
                    unreachable!()
                }
            }

            Expr::This(token) => self.lookup_variable(token, expr),

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

    pub fn execute_block(
        &mut self,
        stmts: &Vec<Stmt>,
        env: Environment,
    ) -> Result<(), RuntimeException> {
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

    fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeException> {
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
                (*self.env).borrow_mut().define(&token.lexeme, value);
            }

            Stmt::Block(stmts) => {
                self.execute_block(stmts, Environment::encloser(&self.env))?;
            }

            Stmt::Class(name, superclass, methods) => {
                let mut superclass_t = None;
                if let Some(superclass) = superclass {
                    if let Expr::Variable(token) = &**superclass {
                        let superclass = self.evaluate(superclass)?;
                        if let Value::Class(tmp) = &superclass {
                            superclass_t = Some(tmp.clone())
                        } else {
                            return Err(RuntimeException::RuntimeError {
                                token: token.clone(),
                                error: "Superclass must be a class.".to_owned(),
                            });
                        }
                    } else {
                        unreachable!()
                    }
                }

                (*self.env).borrow_mut().define(&name.lexeme, Value::Nil);

                let mut old_env = None;
                if let Some(superclass) = &superclass_t {
                    let new_env = Rc::new(RefCell::new(Environment::encloser(&self.env)));
                    let superclass = Value::Class(Rc::clone(superclass));
                    (*new_env).borrow_mut().define("super", superclass);
                    old_env = Some(std::mem::replace(&mut self.env, new_env));
                }

                let mut methods_hm = HashMap::new();
                for method in methods {
                    if let Stmt::Function(token, _, _) = method {
                        let fun =
                            loxcallables::Function::new(method, &self.env, token.lexeme == "init");
                        methods_hm.insert(token.lexeme.clone(), fun);
                    }
                }
                let methods = Rc::new(methods_hm);
                let klass = Rc::new(class::LoxClass::new(
                    name.lexeme.clone(),
                    superclass_t,
                    &methods,
                ));

                if let Some(_) = superclass {
                    if let Some(old_env) = old_env {
                        self.env = old_env;
                    }
                }
                (*self.env).borrow_mut().assign(name, Value::Class(klass))?;
            }

            Stmt::Function(id, _, _) => {
                let fun = loxcallables::Function::new(stmt, &self.env, false);
                let fun = Rc::new(fun);
                (*self.env)
                    .borrow_mut()
                    .define(&id.lexeme, Value::Callable(fun))
            }
            Stmt::Null => (),
            Stmt::Return(_token, expr) => {
                let return_value = if let Some(expr) = expr {
                    self.evaluate(expr)?
                } else {
                    Value::Nil
                };
                return Err(RuntimeException::Return(return_value));
            }
        };
        Ok(())
    }

    pub fn interpret(&mut self, stmts: &Vec<Stmt>) -> Result<(), RuntimeException> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }
}
