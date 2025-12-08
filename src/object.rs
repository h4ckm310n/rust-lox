use crate::{chunk::Chunk, value::Value};
use std::{rc::Rc, cell::RefCell, fmt};
use std::time::{SystemTime, UNIX_EPOCH};

pub enum Obj {
    Closure(Rc<RefCell<Closure>>),
    Function(Rc<RefCell<Function>>),
    NativeFn(Rc<NativeFn>),
    String(String),
    Upvalue(Rc<RefCell<Upvalue>>)
}

pub struct Function {
    pub arity: usize,
    pub upvalue_count: usize,
    pub chunk: Rc<RefCell<Chunk>>,
    pub name: String
}

impl Function {
    pub fn new() -> Self {
        Self {
            arity: 0,
            upvalue_count: 0,
            chunk: Rc::new(RefCell::new(Chunk::new())),
            name: String::new()
        }
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.name.is_empty() {
            write!(f, "<script>")
        } else {
            write!(f, "<fn {}>", self.name)
        }
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.name == other.name
    }
}

pub struct NativeFn {
    pub name: String
}

impl NativeFn {
    pub fn new(name: String) -> Self {
        Self {
            name: name
        }
    }

    pub fn call(&self, _arg_count: usize, _values: Vec<Rc<Value>>) -> Rc<Value> {
        match &self.name as &str {
            "clock" => {
                let time = SystemTime::now().duration_since(UNIX_EPOCH).expect("").as_secs_f64();
                Rc::new(Value::Number(time))
            }
            _ => Rc::new(Value::Nil)
        }
    }
}

pub struct Closure {
    pub function: Rc<RefCell<Function>>,
    pub upvalues: Vec<Rc<RefCell<Upvalue>>>
}

impl Closure {
    pub fn new(function: Rc<RefCell<Function>>) -> Self {
        Self {
            function: function,
            upvalues: Vec::new()
        }
    }
}

pub struct Upvalue {
    pub location: Rc<Value>,
    pub closed: Value,
    pub next: Option<Rc<RefCell<Upvalue>>>
}

impl Upvalue {
    pub fn new(slot: Rc<Value>) -> Self {
        Self {
            location: slot,
            closed: Value::Nil,
            next: None
        }
    }
}