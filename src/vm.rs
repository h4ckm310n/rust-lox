use crate::{chunk::{Chunk, OpCode}, compiler::Compiler, debug::disassemble_instruction, value::Value};

pub struct VM {
    ip: u8,
    stack: Vec<Value>
}

impl VM {
    pub fn init() -> Self {
        Self { ip: 0, stack: Vec::new() }
    }

    pub fn free(&mut self) {

    }

    pub fn interpret(&mut self, path: String, source: String) -> Result<(), InterpretError> {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::init(path.clone(), source.clone());
        if !compiler.compile(&mut chunk) {
            return Err(InterpretError::Compile);
        }
        self.ip = chunk.codes[0];
        self.run(&chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> Result<(), InterpretError> {
        loop {
            disassemble_instruction(chunk, self.ip as usize);
            let instruction: Result<OpCode, _> = self.read_byte().try_into();
            match instruction.unwrap() {
                OpCode::Constant => {
                    let constant = self.read_constant(chunk);
                    self.stack.push(constant);
                },
                op @ (OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide) => {
                    self.binary_op(op)
                },
                OpCode::Negate => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(-value);
                }
                OpCode::Return => {
                    self.stack.pop();
                    return Ok(());
                },
            };
        }
    }

    fn read_byte(&mut self) -> u8 {
        self.ip += 1;
        self.ip
    }

    fn read_constant(&mut self, chunk: &Chunk) -> Value {
        chunk.constants.values[self.read_byte() as usize]
    }

    fn binary_op(&mut self, op: OpCode) {
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        match op {
            OpCode::Add => self.stack.push(a + b),
            OpCode::Subtract => self.stack.push(a - b),
            OpCode::Multiply => self.stack.push(a * b),
            OpCode::Divide => self.stack.push(a / b),
            _ => ()
        };
    }
}

pub enum InterpretError {
    Compile,
    Runtime
}