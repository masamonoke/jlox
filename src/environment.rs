use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::value::Value;

pub struct Environment {
    globals: RefCell<HashMap<String, Option<Value>>>,
    enclosing: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            globals: RefCell::new(HashMap::new()),
            enclosing: None,
        }
    }

    pub fn from(enclosing: Rc<Environment>) -> Environment {
        let mut outer = Environment::new();
        outer.enclosing = Some(enclosing);
        outer
    }

    pub fn define(&self, name: String, value: Option<Value>) {
        self.globals.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        let values = self.globals.borrow();
        if let Some(value) = values.get(name) {
            return value.clone();
        }

        if let Some(ref enclosing) = self.enclosing {
            return enclosing.get(name);
        }

        None
    }

    pub fn contains(&self, name: &str) -> bool {
        self.globals.borrow().contains_key(name)
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
