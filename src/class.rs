use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::instance::LoxInstance;
use crate::loxcallables::{LoxCallable, Function};
use crate::interpreter::Interpreter;
use crate::loxerr::RuntimeException;
use crate::expr::Value;

#[derive(Clone)]
pub struct LoxClass {
    pub name: String,
    superclass: Option<Rc<LoxClass>>,
    methods: Rc<HashMap<String, Function>>, //Rc to derive Clone.
}

impl LoxClass {
    pub fn new(name: String, superclass: Option<Rc<LoxClass>>, methods: &Rc<HashMap<String, Function>>) -> Self {
        LoxClass {name, superclass, methods: Rc::clone(methods)}
    }

    pub fn find_method(&self, name: &str) -> Option<&Function> {
        if self.methods.contains_key(name) {
            return self.methods.get(name);
        }
        
        if let Some(superclass) = &self.superclass {
            return superclass.find_method(name);
        }

        None
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl LoxCallable for LoxClass {
    fn call(&self, interpreter: &mut Interpreter, args: &[Value]) -> Result<Value, RuntimeException> {
        let instance = LoxInstance::new(self.clone());
        let instance = Value::Instance(Rc::new(RefCell::new(instance)));

        if let Some(init) = self.find_method("init") {
            init.bind(instance.clone()).call(interpreter, args)?;
        }
        Ok(instance)
    }

    fn arity(&self) -> usize {
        if let Some(init) = self.find_method("init") {
            init.arity()
        } else {
            0
        }
    }
}