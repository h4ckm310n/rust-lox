use crate::interpreter::{Interpreter, Value};

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Value;
    fn arity(&self) -> usize;
}