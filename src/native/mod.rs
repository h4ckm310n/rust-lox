use crate::callable::Callable;
use crate::environment::Environment;
use crate::interpreter::Value;
use crate::native::array::{ArrayPop, ArrayPush};
use crate::native::clock::Clock;
use crate::native::len::Len;
use std::cell::RefCell;
use std::rc::Rc;
mod clock;
mod array;
mod len;

pub trait NativeFunction: Callable {
    fn get_name(&self) -> String;
}

impl PartialEq for dyn NativeFunction {
    fn eq(&self, other: &Self) -> bool {
        self.get_name() == other.get_name()
    }
}

pub fn init_native_functions(environment: Rc<RefCell<Environment>>) {
    let mut environment = environment.borrow_mut();
    environment.define("clock".to_string(), Rc::new(Value::NativeFunction(Rc::new(RefCell::new(Clock{})))));
    environment.define("push_array".to_string(), Rc::new(Value::NativeFunction(Rc::new(RefCell::new(ArrayPush{})))));
    environment.define("pop_array".to_string(), Rc::new(Value::NativeFunction(Rc::new(RefCell::new(ArrayPop{})))));
    environment.define("len".to_string(), Rc::new(Value::NativeFunction(Rc::new(RefCell::new(Len{})))));
}