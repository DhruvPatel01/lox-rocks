use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::token::Token;
use crate::expr::Value;
use crate::loxerr::RuntimeException;

pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment{values: HashMap::new(), enclosing: None}
    }

    pub fn encloser(encloser: &Rc<RefCell<Environment>>) -> Self {
        Environment { values: HashMap::new(), enclosing: Some(Rc::clone(encloser)) }
    }

    pub fn define(&mut self, name: &str, val: Value) {
        self.values.insert(name.to_owned(), val);
    }

    pub fn assign(&mut self, t: &Token, val: Value) ->Result<(), RuntimeException> {
        if self.values.contains_key(&t.lexeme) {
            self.values.insert(t.lexeme.clone(), val);
            Ok(())
        } else if let Some(enclosed) = &self.enclosing {
            enclosed.borrow_mut().assign(t, val)
        } else {
            Err(RuntimeException::RuntimeError { 
                token: t.clone(), 
                error: format!("Undefined variable '{}'.", t.lexeme)})
        }
    }

    pub fn assign_at(&mut self, dist:usize, t: &Token, val: Value) {
        if dist == 0 {
            self.values.insert(t.lexeme.clone(), val);
        } else {
            let env = self.ancestor(dist);
            env.borrow_mut().values.insert(t.lexeme.clone(), val);
        }
        
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeException> {
        if let Some(v) = self.values.get(&name.lexeme) {
            Ok(v.clone())
        } else if let Some(enclosed) = &self.enclosing {
            enclosed.borrow().get(name)
        } else {
            Err(RuntimeException::RuntimeError{
                token:name.clone(), 
                error: format!("Undefined variable '{}'.", name.lexeme)
            })
        }
    }

    fn ancestor(&self, idx: usize) -> Rc<RefCell<Environment>> {
        let mut env = self.enclosing.clone().unwrap();

        for _ in 1..idx {
            env = Rc::clone(&env).borrow().enclosing.clone().unwrap();
        }
        env
    }

    pub fn get_at(&self, dist: usize, name: &Token) -> Result<Value, RuntimeException> {
        let val = if dist == 0 {
            self.values.get(&name.lexeme).unwrap().clone()
        } else {
            let env = self.ancestor(dist);
            Rc::clone(&env).borrow().values.get(&name.lexeme).unwrap().clone()
        };
        
        Ok(val)
    }

}