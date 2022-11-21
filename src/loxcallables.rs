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

pub struct Function {
    id: Token,
    params: Vec<Token>,
    body: Vec<Stmt>
}

impl Function {
    pub fn new(declaration: &Stmt) -> Function {
        match declaration {
            Stmt::Function(id, params, body) => {
                Function {
                    id: id.clone(), 
                    params: params.clone(), 
                    body: body.clone()
                }
            },
            _ => unreachable!()
        }
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
        let mut env = Environment::encloser(&interpreter.globals);
        for (i, param) in self.params.iter().enumerate() {
            env.define(&param.lexeme, args[i].clone())
        }

        let result = interpreter.execute_block(&self.body, env);

        match result {
            Ok(()) => Ok(Value::Nil),
            Err(RuntimeException::Return(value)) => Ok(value),
            Err(err) => Err(err),
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