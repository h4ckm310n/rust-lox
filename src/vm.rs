use crate::{chunk::{Chunk, OpCode}, compiler::Compiler, debug::disassemble_instruction, value::Value};

pub struct VM {
    ip: u8,
    chunk: Chunk,
    stack: Vec<Value>
}

impl VM {
    pub fn init() -> Self {
        Self {
            ip: 0,
            chunk: Chunk::new(),
            stack: Vec::new()
        }
    }

    pub fn free(&mut self) {

    }

    pub fn interpret(&mut self, path: String, source: String) -> Result<(), InterpretError> {
        let mut compiler = Compiler::init(path.clone(), source.clone());
        if !compiler.compile(&mut self.chunk) {
            return Err(InterpretError::Compile);
        }
        self.ip = self.chunk.codes[0];
        self.run()
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            disassemble_instruction(&self.chunk, self.ip as usize);
            let instruction: Result<OpCode, _> = self.read_byte().try_into();
            match instruction.unwrap() {
                OpCode::Constant => {
                    let constant = self.read_constant().clone();
                    self.stack.push(constant);
                },
                OpCode::True => {
                    self.stack.push(Value::Boolean(true));
                }
                OpCode::False => {
                    self.stack.push(Value::Boolean(false));
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::Nil => {
                    self.stack.push(Value::Nil);
                }
                op @ (OpCode::Greater | OpCode::Less | OpCode::Add | OpCode::Subtract | OpCode::Multiply | OpCode::Divide) => {
                    self.binary_op(op)?;
                },
                OpCode::Not => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(is_falsey(&value)));
                },
                OpCode::Negate => {
                    let value = self.stack.last().unwrap();
                    if let Some(number) = value.as_number() {
                        self.stack.pop();
                        self.stack.push(Value::Number(-number));
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return Err(InterpretError::Runtime);
                    }
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

    fn read_constant(&mut self) -> &Value {
        let byte = self.read_byte();
        &self.chunk.constants.values[byte as usize]
    }

    fn binary_op(&mut self, op: OpCode) -> Result<(), InterpretError> {
        if !self.peek(0).is_number() || !self.peek(1).is_number() {
            self.runtime_error("Operands must be numbers.");
            return Err(InterpretError::Runtime);
        }
        let b = self.stack.pop().unwrap().as_number().unwrap();
        let a = self.stack.pop().unwrap().as_number().unwrap();
        match op {
            OpCode::Greater => self.stack.push(Value::Boolean(a > b)),
            OpCode::Less => self.stack.push(Value::Boolean(a < b)),
            OpCode::Add => self.stack.push(Value::Number(a + b)),
            OpCode::Subtract => self.stack.push(Value::Number(a - b)),
            OpCode::Multiply => self.stack.push(Value::Number(a * b)),
            OpCode::Divide => self.stack.push(Value::Number(a / b)),
            _ => ()
        };
        Ok(())
    }

    fn peek(&mut self, distance: i8) -> &Value {
        &self.stack[(-1-distance) as usize]
    }

    fn runtime_error(&mut self, message: &str) {
        println!("{message}");
        let line = self.chunk.lines[self.chunk.lines.len()-2];
        println!("[line {line}] in script");
        self.reset_stack();
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }
}

pub enum InterpretError {
    Compile,
    Runtime
}

fn is_falsey(value: &Value) -> bool {
    value.is_nil() || (value.is_bool() && !value.as_bool().unwrap())
}