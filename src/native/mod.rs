use crate::callable::Callable;
use crate::interpreter::{Interpreter, Value};
use std::rc::Rc;
mod clock;

#[derive(PartialEq)]
pub struct NativeFunction {
    pub name: String
}

impl NativeFunction {
    pub fn new(name: String) -> Self {
        Self { name: name }
    }
}

impl Callable for NativeFunction {
    fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<Rc<Value>>) -> Rc<Value> {
        match &self.name as &str {
            "clock" => clock::clock(),
            _ => Rc::new(Value::Nil)
        }
    }

    fn arity(&self) -> usize {
        match &self.name as &str {
            "clock" => 0,
            _ => 0
        }
    }
}