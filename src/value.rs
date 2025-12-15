use std::{cell::RefCell, fmt, rc::Rc};

use crate::object::{BoundMethod, Class, Closure, Function, Instance, NativeFn, Obj};

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    Obj(Rc<Obj>)
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

    pub fn as_obj(&self) -> Option<Rc<Obj>> {
        if let Self::Obj(obj) = &self {
            Some(obj.clone())
        } else {
            None
        }
    }

    pub fn as_class(&self) -> Option<Rc<RefCell<Class>>> {
        if let Some(obj) = self.as_obj() &&
           let Obj::Class(class) = &*obj {
            Some(class.clone())
        } else {
            None
        }
    }

    pub fn as_instance(&self) -> Option<Rc<RefCell<Instance>>> {
        if let Some(obj) = self.as_obj() &&
           let Obj::Instance(instance) = &*obj {
            Some(instance.clone())
        } else {
            None
        }
    }

    pub fn as_bound_method(&self) -> Option<Rc<RefCell<BoundMethod>>> {
        if let Some(obj) = self.as_obj() &&
           let Obj::BoundMethod(bound_method) = &*obj {
            Some(bound_method.clone())
        } else {
            None
        }
    }

    pub fn as_closure(&self) -> Option<Rc<RefCell<Closure>>> {
        if let Some(obj) = self.as_obj() &&
           let Obj::Closure(closure) = &*obj {
            Some(closure.clone())
        } else {
            None
        }
    }

    pub fn as_function(&self) -> Option<Rc<RefCell<Function>>> {
        if let Some(obj) = self.as_obj() &&
           let Obj::Function(function) = &*obj {
            Some(function.clone())
        } else {
            None
        }
    }

    pub fn as_native(&self) -> Option<Rc<NativeFn>> {
        if let Some(obj) = self.as_obj() &&
           let Obj::NativeFn(native_fn) = &*obj {
            Some(native_fn.clone())
        } else {
            None
        }
    }

    pub fn as_string(&self) -> Option<String> {
        if let Some(obj) = self.as_obj() &&
           let Obj::String(string) = &*obj {
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

    pub fn is_class(&self) -> bool {
        self.as_class().is_some()
    }

    pub fn is_instance(&self) -> bool {
        self.as_instance().is_some()
    }

    pub fn is_bound_method(&self) -> bool {
        self.as_bound_method().is_some()
    }

    pub fn is_closure(&self) -> bool {
        self.as_closure().is_some()
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
            Self::Obj(obj) => match obj.as_ref() {
                Obj::Class(class) => write!(f, "{}", class.borrow().name),
                Obj::Instance(instance) => write!(f, "{} instance", instance.borrow().class.borrow().name),
                Obj::BoundMethod(bound_method) => write!(f, "{}", bound_method.borrow().method.borrow().function.borrow()),
                Obj::Closure(closure) => write!(f, "{}", closure.borrow().function.borrow()),
                Obj::Function(function) => write!(f, "{}", function.borrow()),
                Obj::NativeFn(_) => write!(f, "<native fn>"),
                Obj::String(string) => write!(f, "{string}"),
                Obj::Upvalue(_) => write!(f, "upvalue")
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
                match (l0.as_ref(), r0.as_ref()) {
                    (Obj::Class(l0), Obj::Class(r0)) => *l0.borrow() == *r0.borrow(),
                    (Obj::Closure(l0), Obj::Closure(r0)) => *l0.borrow().function == *r0.borrow().function,
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