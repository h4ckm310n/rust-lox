use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{class::Class, interpreter::{ErrType, Value}, token::Token};


#[derive(PartialEq)]
pub struct Instance {
    class: Rc<RefCell<Class>>,
    fields: HashMap<String, Value>,
    rc_self: Option<Rc<RefCell<Self>>>
}

impl Instance {
    pub fn new(class: Rc<RefCell<Class>>) -> Self {
        Self {
            class: class,
            fields: HashMap::new(),
            rc_self: None
        }
    }

    pub fn set_rc_self(&mut self, rc_self: Rc<RefCell<Self>>) {
        self.rc_self = Some(rc_self);
    }

    pub fn unset_rc_self(&mut self) {
        self.rc_self = None;
    }

    pub fn get(&self, name: &Token) -> Result<Value, ErrType> {
        if let Some(field) = self.fields.get(&name.text) {
            return Ok(field.to_owned());
        }
        if let Some(method) = self.class.borrow().find_method(name.text.clone()) {
            let bind = method.borrow().bind(self.rc_self.as_ref().unwrap().clone());
            return Ok(Value::Function(Rc::new(RefCell::new(bind))));
        }
        Err(ErrType::Err(name.clone(), format!("Undefined property '{}'.", name.text)))
    }

    pub fn set(&mut self, name: &Token, value: Value) {
        self.fields.insert(name.text.clone(), value);
    }
}

impl ToString for Instance {
    fn to_string(&self) -> String {
        format!("{} instance", self.class.borrow().name)
    }
}