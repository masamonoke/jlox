use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::value::Value;

#[derive(Debug)]
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

    pub fn define(&self, name: String, value: Option<Value>) -> bool {
        return self.globals.borrow_mut().insert(name, value).is_some();
    }

    pub fn update(&self, name: String, value: Option<Value>) -> bool {
        if self.globals.borrow().contains_key(&name) {
            self.globals.borrow_mut().insert(name, value);
            return true;
        }

        let mut scope = self.enclosing.as_ref();

        while let Some(current) = scope {
            if current
                .globals
                .borrow_mut()
                .insert(name.clone(), value.clone())
                .is_some()
            {
                return true;
            }
            scope = current.enclosing.as_ref();
        }
        false
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
            || self.enclosing.as_ref().is_some_and(|e| e.contains(name))
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
