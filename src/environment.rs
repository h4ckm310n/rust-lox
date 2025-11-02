use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::Value;
use crate::token::Token;

#[derive(PartialEq)]
pub struct Environment {
    pub enclosing: Option<Rc<RefCell<Environment>>>,
    values: HashMap<String, Value>
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
        Self {
            enclosing: enclosing,
            values: HashMap::new()
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> Result<Value, (Token, String)> {
        if let Some(value) = self.values.get(&name.text) {
            return Ok(value.to_owned());
        }
        
        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow().get(name);
        }
        Err((name.clone(), format!("Undefined variable '{}'.", name.text)))
    }

    pub fn assign(&mut self, name: &Token, value: Value) -> Result<(), (Token, String)> {
        if self.values.contains_key(&name.text) {
            self.values.insert(name.text.clone(), value);
            return Ok(());
        }

        if let Some(enclosing) = &self.enclosing {
            return enclosing.borrow_mut().assign(name, value);
        }
        Err((name.clone(), format!("Undefined variable '{}'.", name.text)))
    }

    pub fn assign_at(&mut self, distance: usize, name: &Token, value: Value) {
        if distance == 0 {
            self.values.insert(name.text.clone(), value);
        } else {
            self.ancestor(distance).borrow_mut().values.insert(name.text.clone(), value);
        }
    }

    pub fn get_at(&self, distance: usize, name: String) -> Value {
        if distance == 0 {
            self.values.get(&name).unwrap().to_owned()
        } else {
            self.ancestor(distance).borrow().values.get(&name).unwrap().to_owned()
        }
    }

    fn ancestor(&self, distance: usize) -> Rc<RefCell<Environment>> {
        let mut environment = self.enclosing.as_ref().unwrap().clone();
        for _ in 1..distance {
            let next = environment.borrow().enclosing.as_ref().unwrap().clone();
            environment = next;
        }
        environment
    }
}