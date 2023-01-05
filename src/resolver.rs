use std::collections::HashMap;
use std::rc::Rc;

use crate::expr::Expr;
use crate::interpreter::Interpreter;
use crate::loxerr;
use crate::stmt::Stmt;
use crate::token::Token;

#[derive(PartialEq)]
enum FunctionType {
    NONE,
    FUNCTION,
    INITIALIZER,
    METHOD,
}

#[derive(PartialEq)]
enum ClassType {
    NONE, 
    CLASS
}

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>,
    pub has_error: bool,
    current_function: FunctionType,
    current_class: ClassType,
}

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &'a mut Interpreter) -> Resolver<'a> {
        Resolver {
            interpreter,
            scopes: Vec::new(),
            has_error: false,
            current_function: FunctionType::NONE,
            current_class: ClassType::NONE,
        }
    }

    pub fn resolve(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Null => (),
            Stmt::Block(stmts) => {
                self.begin_scope();
                self.resolve(stmts);
                self.end_scope();
            }
            Stmt::Class(name, methods) => {
                let enclosing_class = std::mem::replace(&mut self.current_class, ClassType::CLASS);
                self.declare(name);
                self.define(name);

                self.begin_scope();
                self.scopes.last_mut().unwrap().insert("this".to_owned(), true);

                for stmt in methods {
                    if let Stmt::Function(name, params , body) = stmt {
                        let  declaration = if name.lexeme == "init" {
                            FunctionType::INITIALIZER
                        } else {
                            FunctionType::METHOD
                        };

                        self.resolve_function(params, body, declaration);
                    }
                    
                }

                self.end_scope();
                self.current_class = enclosing_class;
            }
            Stmt::Expression(expr) => self.resolve_expr(expr),
            Stmt::Function(token, params, body) => {
                self.declare(token);
                self.define(token);
                self.resolve_function(params, body, FunctionType::FUNCTION);
            }
            Stmt::If(condition, then_branch, else_branch) => {
                self.resolve_expr(condition);
                self.resolve_stmt(then_branch);
                if let Some(else_branch) = else_branch {
                    self.resolve_stmt(else_branch);
                }
            }
            Stmt::Print(expr) => self.resolve_expr(expr),
            Stmt::Return(name, ret_expr) => {
                if self.current_function == FunctionType::NONE {
                    loxerr::parse_error(
                        name,
                        "Can't return from top-level code."
                    );
                    self.has_error = true;
                    ()
                } else if let Some(value) = ret_expr {
                    if self.current_function == FunctionType::INITIALIZER {
                        loxerr::parse_error(
                            name,
                            "Can't return a value from an initializer."
                        );
                        self.has_error = true;
                    }
                    self.resolve_expr(value)
                } else {
                    ()
                }
            }
            Stmt::Var(token, init) => {
                self.declare(token);
                if let Some(init) = init {
                    self.resolve_expr(init);
                }
                self.define(token);
            }
            Stmt::While(condition, body) => {
                self.resolve_expr(condition);
                self.resolve_stmt(body);
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Rc<Expr>) {
        match &**expr {
            Expr::Assign(token, right) => {
                self.resolve_expr(right);
                self.resolve_local(expr, token)
            }

            Expr::Binary(left, _op, right) => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }

            Expr::Call(callee, _, body) => {
                self.resolve_expr(&callee);
                for expr in body {
                    self.resolve_expr(expr);
                }
            }
            Expr::Get(object, _) => self.resolve_expr(object),
            Expr::Grouping(expr) => self.resolve_expr(expr),
            Expr::Literal(..) => (),
            Expr::Logical(left, _op, right) => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            }
            Expr::Set(object, _, value) => {
              self.resolve_expr(object);
              self.resolve_expr(value);  
            }
            Expr::This(token) => {
                if self.current_class == ClassType::NONE {
                    loxerr::parse_error(
                        token,
                        "Can't use 'this' outside of a class."
                    );
                    self.has_error = true;
                } else {
                    self.resolve_local(expr, token);
                }
                

            } 
            Expr::Unary(_, expr) => self.resolve_expr(expr),
            Expr::Variable(token) => {
                if let Some(scope) = self.scopes.last() {
                    if scope.get(&token.lexeme) == Some(&false) {
                        loxerr::parse_error(
                            token,
                            "Can't read local variable in its own initializer.",
                        );
                        self.has_error = true;
                    } else {
                        self.resolve_local(expr, token);
                    }
                }
            }
        }
    }

    fn resolve_local(&mut self, expr: &Rc<Expr>, name: &Token) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
                self.interpreter.resolve(expr, i);
                return;
            }
        }
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body: &Vec<Stmt>, ftype: FunctionType) {
        let enclosing_function = std::mem::replace(&mut self.current_function, ftype);
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve(body);
        self.end_scope();

        self.current_function = enclosing_function;
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                loxerr::parse_error(
                    name,
                    "Already a variable with this name in this scope.",
                );
                self.has_error = true;
            }
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }
}
