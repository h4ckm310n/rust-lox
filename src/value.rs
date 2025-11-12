use std::fmt;

#[derive(Clone)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64)
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
}

pub fn print_value(value: Value) {
    print!("{value}");
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Nil => write!(f, "nil"),
            _ => write!(f, "{self}")
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