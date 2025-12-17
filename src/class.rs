use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::callable::Callable;
use crate::function::Function;
use crate::instance::Instance;
use crate::interpreter::*;


pub struct Class {
    pub name: String,
    methods: HashMap<String, Rc<RefCell<Function>>>,
    superclass: Option<Rc<RefCell<Class>>>,
    weak_self: Option<Weak<RefCell<Self>>>
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Rc<RefCell<Function>>>, superclass: Option<Rc<RefCell<Class>>>) -> Self {
        Self {
           name: name,
           methods: methods,
           superclass: superclass,
           weak_self: None
        }
    }

    pub fn set_weak_self(&mut self, weak_self: Weak<RefCell<Self>>) {
        self.weak_self = Some(weak_self);
    }

    pub fn unset_weak_self(&mut self) {
        self.weak_self = None;
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
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Rc<Value>>) -> Rc<Value> {
        let instance = Rc::new(RefCell::new(Instance::new(self.weak_self.as_ref().unwrap().clone())));
        instance.borrow_mut().set_weak_self(Rc::downgrade(&instance));
        let initializer = self.find_method("init".to_string());
        if let Some(initializer) = initializer {
            initializer.borrow().bind(instance.clone()).call(interpreter, arguments);
        }
        Rc::new(Value::Instance(instance))
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

impl PartialEq for Class {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.methods == other.methods && self.superclass == other.superclass
    }
}
