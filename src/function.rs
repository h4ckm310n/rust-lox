use std::{cell::RefCell, rc::Rc};
use crate::{ast::stmt::*, callable::Callable, environment::Environment, instance::Instance, interpreter::*};

#[derive(PartialEq)]
pub struct Function {
    decl: FunDecl,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool
}

impl Function {
    pub fn new(decl: FunDecl, closure: Rc<RefCell<Environment>>, is_initializer: bool) -> Self {
        Self {
            decl: decl,
            closure: closure,
            is_initializer: is_initializer
        }
    }

    pub fn bind(&self, instance: Rc<RefCell<Instance>>) -> Self {
        let environment = Rc::new(RefCell::new(Environment::new(Some(self.closure.clone()))));
        environment.borrow_mut().define("this".to_string(), Value::Instance(instance));
        Self {
            decl: self.decl.clone(),
            closure: environment.clone(),
            is_initializer: self.is_initializer
        }
    }
}

impl Callable for Function {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Value {
        let environment = Rc::new(RefCell::new(Environment::new(Some(self.closure.clone()))));
        for i in 0..self.decl.params.len() {
            environment.borrow_mut().define(
                self.decl.params.get(i).unwrap().text.clone(), 
                arguments.get(i).unwrap().to_owned());
        }
        let result = interpreter.execute_block(&self.decl.body, environment);
        if let Err(ErrType::Return(value)) = result {            
            if self.is_initializer {
                return self.closure.borrow().get_at(0, "this".to_string());
            }
            return value;
        }
        if self.is_initializer {
            return self.closure.borrow().get_at(0, "this".to_string());
        }
        Value::Nil
    }

    fn arity(&self) -> usize {
        self.decl.params.len()
    }
}

impl ToString for Function {
    fn to_string(&self) -> String {
        format!("<fn {}>", self.decl.name.text)
    }
}