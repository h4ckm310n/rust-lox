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
    Closure,
    CloseUpvalue,
    Return
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
        self.constants.write(value);
        self.constants.count() - 1
    }
}