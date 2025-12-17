use std::rc::Rc;
use crate::interpreter::{Interpreter, Value};

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Rc<Value>>) -> Rc<Value>;
    fn arity(&self) -> usize;
}