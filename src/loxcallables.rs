use crate::expr::Value;
use crate::interpreter::Interpreter;
use crate::loxerr::RuntimeError;

#[derive(Clone)]
pub struct LoxCallable {
    pub arity: usize,
    body: fn(&[Value]) -> Result<Value, RuntimeError>,
}

impl LoxCallable {
    pub fn new(arity: usize, body: fn(&[Value])->Result<Value, RuntimeError>) -> Self {
        LoxCallable { arity, body: body}
    }

    pub fn call(&self, interpreter: &Interpreter, args: &[Value]) -> Result<Value, RuntimeError> {
        (self.body)(args)
    }
}

impl std::fmt::Display for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<native fn>")    
    }
}