use std::cell::RefCell;
use std::rc::Rc;

use crate::env::Environment;
use crate::expr::Value;
use crate::interpreter::Interpreter;
use crate::loxerr::RuntimeException;
use crate::stmt::Stmt;
use crate::token::Token;

pub trait LoxCallable: std::fmt::Display {
    fn arity(&self) -> usize;
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, RuntimeException>;
}

#[derive(Clone)]
pub struct Native {
    arity: usize,
    body: fn(&[Value]) -> Result<Value, RuntimeException>,
}

impl Native{
    pub fn new(arity: usize, body: fn(&[Value])->Result<Value, RuntimeException>) -> Self {
        Native { arity, body: body}
    }
}

#[derive(Clone)]
pub struct Function {
    id: Token,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
    is_init: bool,
}

impl Function {
    pub fn new(declaration: &Stmt, closure: &Rc<RefCell<Environment>>, is_init:bool ) -> Function {
        match declaration {
            Stmt::Function(id, params, body) => {
                Function {
                    id: id.clone(), 
                    params: params.clone(), 
                    body: body.clone(),
                    closure: Rc::clone(closure),
                    is_init
                }
            },
            _ => unreachable!()
        }
    }

    pub fn bind(&self, instance: Value) -> Function {
        let mut env = Environment::encloser(&self.closure);
        env.define(&"this", instance);
        
        let mut fun = self.clone();
        fun.closure = Rc::new(RefCell::new(env));
        fun
    }
}


impl LoxCallable for Native {
    fn call(&self, _interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, RuntimeException> {
        (self.body)(args)
    }

    fn arity(&self) -> usize {
        self.arity
    }
}

impl LoxCallable for Function {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, RuntimeException> {
        let mut env = Environment::encloser(&self.closure);
        for (i, param) in self.params.iter().enumerate() {
            env.define(&param.lexeme, args[i].clone())
        }

        let result = interpreter.execute_block(&self.body, env);

        let dummy_token = Token{
            token_type: crate::token::TokenType::This, 
            lexeme: "this".to_owned(),
            line: 0
        };

        match result {
            Ok(()) => (),
            Err(RuntimeException::Return(value)) =>  {
                if self.is_init {
                    return self.closure.borrow().get_at(0, &dummy_token)
                } else {
                    return Ok(value)
                }
            },

            Err(err) => return Err(err),
        }
        
        if self.is_init {
            self.closure.borrow().get_at(0, &dummy_token)
        } else {
            Ok(Value::Nil)
        }
        
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}

impl std::fmt::Display for Native {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")    
    }
}

impl std::fmt::Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn {}>", self.id.lexeme)    
    }
}