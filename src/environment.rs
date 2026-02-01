use std::collections::{HashMap};

use crate::value::Value;

pub struct Environment {
    globals: HashMap<String, Option<Value>>
}

impl Environment {
    pub fn new() -> Environment{
        Environment { globals: HashMap::new() }
    }

    pub fn define(&mut self, name: String, value: Option<Value>) {
        self.globals.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.globals.get(name).and_then(|opt| opt.as_ref())
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}
