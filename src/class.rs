use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::callable::Callable;
use crate::function::Function;
use crate::instance::Instance;
use crate::interpreter::*;


#[derive(PartialEq)]
pub struct Class {
    pub name: String,
    methods: HashMap<String, Rc<RefCell<Function>>>,
    superclass: Option<Rc<RefCell<Class>>>,
    rc_self: Option<Rc<RefCell<Self>>>
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Rc<RefCell<Function>>>, superclass: Option<Rc<RefCell<Class>>>) -> Self {
        Self {
           name: name,
           methods: methods,
           superclass: superclass,
           rc_self: None
        }
    }

    pub fn set_rc_self(&mut self, rc_self: Rc<RefCell<Self>>) {
        self.rc_self = Some(rc_self);
    }

    pub fn unset_rc_self(&mut self) {
        self.rc_self = None;
    }

    pub fn find_method(&self, name: String) -> Option<Rc<RefCell<Function>>> {
        if let Some(method) = self.methods.get(&name) {
            Some(method.clone())
        } else if let Some(superclass) = &self.superclass {
            superclass.borrow().find_method(name)
        } else {
            None
        }
    }
}

impl Callable for Class {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Value {
        let instance = Rc::new(RefCell::new(Instance::new(self.rc_self.as_ref().unwrap().clone())));
        instance.borrow_mut().set_rc_self(instance.clone());
        let initializer = self.find_method("init".to_string());
        if let Some(initializer) = initializer {
            initializer.borrow().bind(instance.clone()).call(interpreter, arguments);
        }
        Value::Instance(instance)
    }

    fn arity(&self) -> usize {
        let initializer = self.find_method("init".to_string());
        if let Some(initializer) = initializer {
            return initializer.borrow().arity();
        }
        0
    }
}

impl ToString for Class {
    fn to_string(&self) -> String {
        self.name.clone()
    }
}

