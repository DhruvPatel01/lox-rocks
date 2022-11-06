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
        self.values.insert(name.to_owned(), val.clone());
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