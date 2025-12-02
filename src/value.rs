use std::{cell::RefCell, fmt, rc::Rc};

use crate::object::{Function, NativeFn, Obj};

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    Obj(Rc<RefCell<Obj>>)
}

impl Value {
    pub fn as_number(&self) -> Option<f64> {
        if let Self::Number(number) = &self {
            Some(*number)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Boolean(boolean) = &self {
            Some(*boolean)
        } else {
            None
        }
    }

    pub fn as_obj(&self) -> Option<Rc<RefCell<Obj>>> {
        if let Self::Obj(obj) = &self {
            Some(obj.clone())
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<Rc<RefCell<Function>>> {
        if let Some(obj) = self.as_obj() &&
           let Obj::Function(function) = &*obj.borrow() {
            Some(function.clone())
        } else {
            None
        }
    }

    pub fn as_native(&self) -> Option<Rc<NativeFn>> {
        if let Some(obj) = self.as_obj() &&
           let Obj::NativeFn(native_fn) = &*obj.borrow() {
            Some(native_fn.clone())
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<String> {
        if let Some(obj) = self.as_obj() &&
           let Obj::String(string) = &*obj.borrow() {
            Some(string.clone())
        } else {
            None
        }
    }

    pub fn is_number(&self) -> bool {
        self.as_number().is_some()
    }

    pub fn is_bool(&self) -> bool {
        self.as_bool().is_some()
    }

    pub fn is_nil(&self) -> bool {
        if let Self::Nil = &self {
            true
        } else {
            false
        }
    }

    pub fn is_obj(&self) -> bool {
        self.as_obj().is_some()
    }

    pub fn is_function(&self) -> bool {
        self.as_function().is_some()
    }

    pub fn is_native(&self) -> bool {
        self.as_native().is_some()
    }

    pub fn is_string(&self) -> bool {
        self.as_string().is_some()
    }
}

pub fn print_value(value: Rc<Value>) {
    print!("{value}");
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Nil => write!(f, "nil"),
            Self::Boolean(boolean) => write!(f, "{boolean}"),
            Self::Number(number) => write!(f, "{number}"),
            Self::Obj(obj) => match &*obj.borrow() {
                Obj::Function(function) => write!(f, "{}", function.borrow()),
                Obj::NativeFn(_) => write!(f, "<native fn>"),
                Obj::String(string) => write!(f, "{string}"),
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.is_nil() {
            return true;
        }
        match (self, other) {
            (Self::Boolean(l0), Self::Boolean(r0)) => l0 == r0,
            (Self::Number(l0), Self::Number(r0)) => l0 == r0,
            (Self::Obj(l0), Self::Obj(r0)) => {
                match (&*l0.borrow(), &*r0.borrow()) {
                    (Obj::Function(l0), Obj::Function(r0)) => *l0 == *r0,
                    (Obj::String(l0), Obj::String(r0)) => *l0 == *r0,
                    _ => false
                }
            },
            _ => false,
        }
    }
}

pub struct ValueArray {
    pub values: Vec<Value>
}

impl ValueArray {
    pub fn new() -> Self {
        Self {
            values: Vec::new()
        }
    }

    pub fn write(&mut self, value: Value) {
        self.values.push(value);
    }

    pub fn free(&mut self) {
        self.values.clear();
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }
}