use std::{collections::HashMap, fmt::Display};
use std::rc::Rc;
use std::cell::RefCell;

use crate::{class::LoxClass, expr::Value, loxerr::RuntimeException, token::Token};

#[derive(Clone)]
pub struct LoxInstance {
    pub class: LoxClass,
    fields: Rc<RefCell<HashMap<String, Value>>>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> LoxInstance {
        LoxInstance {
            class,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&self, name: &Token) -> Result<Value, RuntimeException> {
        if self.fields.borrow().contains_key(&name.lexeme) {
            Ok(self.fields.borrow().get(&name.lexeme).unwrap().clone())
        } else if let Some(method) = self.class.find_method(&name.lexeme) {
            let val = Value::Instance(Rc::new(RefCell::new(self.clone())));
            let fun = method.bind(val);
            Ok(Value::Callable(Rc::new(fun)))
        } else {
            Err(RuntimeException::RuntimeError {
                token: name.clone(),
                error: format!("Undefined property '{}'.", name.lexeme),
            })
        }
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.borrow_mut().insert(name.lexeme.clone(), value);
    }
}

impl Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
