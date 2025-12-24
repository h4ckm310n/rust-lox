use std::rc::Rc;
use crate::{callable::Callable, interpreter::{Interpreter, Value}, native::NativeFunction};

pub struct Len {}

impl NativeFunction for Len {
    fn get_name(&self) -> String {
        "len".to_string()
    }
}

impl Callable for Len {
    fn call(&self, _interpreter: &mut Interpreter, arguments: Vec<Rc<Value>>) -> Rc<Value> {
        let value = &arguments[0];
        let length = match &**value {
            Value::String(string) => string.len(),
            Value::Array(array) => array.borrow().len(),
            _ => {
                println!("Error: Only strings and arrays have length.");
                return Rc::new(Value::Nil);
            }
        };
        Rc::new(Value::Number(length as f64))
    }

    fn arity(&self) -> usize {
        1
    }
}