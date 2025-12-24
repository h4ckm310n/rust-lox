use std::rc::Rc;
use crate::{callable::Callable, interpreter::{Interpreter, Value}, native::NativeFunction};

pub struct ArrayPush {}

impl NativeFunction for ArrayPush {
    fn get_name(&self) -> String {
        "push_array".to_string()
    }
}

impl Callable for ArrayPush {
    fn call(&self, _interpreter: &mut Interpreter, arguments: Vec<Rc<Value>>) -> Rc<Value> {
        let array = &arguments[0];
        if let Value::Array(array) = &**array {
            let value = &arguments[1];
            array.borrow_mut().push(value.clone());
        } else {
            println!("Only arrays can be pushed.");
        }
        Rc::new(Value::Nil)
    }

    fn arity(&self) -> usize {
        2
    }
}

pub struct ArrayPop {}

impl NativeFunction for ArrayPop {
    fn get_name(&self) -> String {
        "pop_array".to_string()
    }
}

impl Callable for ArrayPop {
    fn call(&self, _interpreter: &mut Interpreter, arguments: Vec<Rc<Value>>) -> Rc<Value> {
        let array = &arguments[0];
        if let Value::Array(array) = &**array {
            let value = array.borrow_mut().pop();
            if let Some(value) = value {
                return value;
            } else {
                println!("Failed to pop from array.");
            }
        } else {
            println!("Only arrays can be poped.");
        }
        Rc::new(Value::Nil)
    }

    fn arity(&self) -> usize {
        1
    }
}