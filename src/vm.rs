use crate::{chunk::{Chunk, OpCode}, debug::disassemble_instruction, value::{Value, print_value}};

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: u8,
    stack: Vec<Value>
}

impl<'a> VM<'a> {
    pub fn init() {

    }

    pub fn free(&mut self) {

    }

    fn interpret(&mut self, chunk: &'a Chunk) -> Result<(), InterpretError> {
        self.chunk = chunk;
        self.ip = chunk.codes[0];
        self.run()
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            disassemble_instruction(self.chunk, self.ip as usize);
            let instruction: Result<OpCode, _> = self.read_byte().try_into();
            match instruction.unwrap() {
                OpCode::Constant => {
                    let constant = self.read_constant();
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
    
        Ok(())
    }

    fn read_byte(&mut self) -> u8 {
        self.ip += 1;
        self.ip
    }

    fn read_constant(&mut self) -> Value {
        self.chunk.constants.values[self.read_byte() as usize]
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