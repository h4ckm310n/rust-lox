use crate::value::{Value, ValueArray};
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum OpCode {
    Constant,
    Nil,
    True,
    False,
    Pop,
    GetLocal,
    SetLocal,
    GetGlobal,
    DefineGlobal,
    SetGlobal,
    GetUpvalue,
    SetUpvalue,
    GetProperty,
    SetProperty,
    GetSuper,
    Equal,
    Greater,
    Less,
    Add,
    Subtract,
    Multiply,
    Divide,
    Not,
    Negate,
    Print,
    Jump,
    JumpIfFalse,
    Loop,
    Call,
    Invoke,
    SuperInvoke,
    Closure,
    CloseUpvalue,
    Return,
    Class,
    Inherit,
    Method
}

pub struct Chunk {
    pub codes: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: ValueArray
}

impl Chunk {
    pub fn new() -> Self {
        Self { 
            codes: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new()
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.codes.push(byte);
        self.lines.push(line);
    }

    pub fn free(&mut self) {
        self.codes.clear();
        self.lines.clear();
        self.constants.free();
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        let i = self.find_exist_string(&value);
        if i != -1 {
            return i as usize;
        }
        self.constants.write(value);
        self.constants.count() - 1
    }

    pub fn find_exist_string(&self, value: &Value) -> isize {
        if !value.is_string() {
            return -1;
        }
        for i in 0..self.constants.values.len() {
            if let Some(value_) = self.constants.values[i].as_string() && value_ == value.as_string().unwrap() {
                return i as isize;
            }
        }
        -1
    }
}