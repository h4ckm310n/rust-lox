use std::{cell::RefCell, collections::HashMap, rc::{Rc, Weak}};

use crate::{class::Class, interpreter::{ErrType, Value}, token::Token};


pub struct Instance {
    class: Weak<RefCell<Class>>,
    fields: HashMap<String, Rc<Value>>,
    weak_self: Option<Weak<RefCell<Self>>>
}

impl Instance {
    pub fn new(class: Weak<RefCell<Class>>) -> Self {
        Self {
            class: class,
            fields: HashMap::new(),
            weak_self: None
        }
    }

    pub fn set_weak_self(&mut self, weak_self: Weak<RefCell<Self>>) {
        self.weak_self = Some(weak_self);
    }

    pub fn unset_weak_self(&mut self) {
        self.weak_self = None;
    }

    pub fn get(&self, name: &Token) -> Result<Rc<Value>, ErrType> {
        if let Some(field) = self.fields.get(&name.text) {
            return Ok(field.to_owned());
        }
        if let Some(method) = self.class.upgrade().unwrap().borrow().find_method(name.text.clone()) {
            let bind = method.borrow().bind(self.weak_self.as_ref().unwrap().upgrade().unwrap());
            return Ok(Rc::new(Value::Function(Rc::new(RefCell::new(bind)))));
        }
        Err(ErrType::Err(name.clone(), format!("Undefined property '{}'.", name.text)))
    }

    pub fn set(&mut self, name: &Token, value: Rc<Value>) {
        self.fields.insert(name.text.clone(), value);
    }
}

impl ToString for Instance {
    fn to_string(&self) -> String {
        format!("{} instance", self.class.upgrade().unwrap().borrow().name)
    }
}

impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool {
        self.class.upgrade().unwrap() == other.class.upgrade().unwrap() && self.fields == other.fields
    }
}