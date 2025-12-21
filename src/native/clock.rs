use std::{rc::Rc, time::{SystemTime, UNIX_EPOCH}};
use crate::interpreter::Value;

pub fn clock() -> Rc<Value> {
    let time = SystemTime::now().duration_since(UNIX_EPOCH).expect("").as_secs_f64();
    Rc::new(Value::Number(time))
}