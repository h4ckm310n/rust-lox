use std::{rc::Rc, time::{SystemTime, UNIX_EPOCH}};
use crate::{callable::Callable, interpreter::{Value, Interpreter}, native::NativeFunction};

pub struct Clock {}
impl NativeFunction for Clock {
    fn get_name(&self) -> String {
        "clock".to_string()
    }
}

impl Callable for Clock {
    fn call(&self, _interpreter: &mut Interpreter, _arguments: Vec<Rc<Value>>) -> Rc<Value> {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).expect("").as_secs_f64();
        Rc::new(Value::Number(time))
    }

    fn arity(&self) -> usize {
        0
    }
}
