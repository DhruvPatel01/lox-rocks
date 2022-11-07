use std::collections::HashMap;

use crate::token::Token;
use crate::expr::Value;
use crate::loxerr::RuntimeError;

pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Environment{values: HashMap::new()}
    }

    pub fn define(&mut self, name: &str, val: Value) {
        self.values.insert(name.to_owned(), val);
    }

    pub fn assign(&mut self, t: &Token, val: Value) ->Result<(), RuntimeError> {
        if self.values.contains_key(&t.lexeme) {
            self.values.insert(t.lexeme.clone(), val);
            Ok(())
        } else {
            Err(RuntimeError { 
                token: t.clone(), 
                error: format!("Undefined variable '{}'.", t.lexeme)})
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeError> {
        if let Some(v) = self.values.get(&name.lexeme) {
            Ok(v.clone())
        } else {
            Err(RuntimeError{
                token:name.clone(), 
                error: format!("Undefined variable '{}'.", name.lexeme)
            })
        }
    }
}