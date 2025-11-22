use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{chunk::{Chunk, OpCode}, compiler::Compiler, debug::disassemble_instruction, object::Obj, value::{Value, print_value}};

pub struct VM {
    ip: usize,
    chunk: Rc<RefCell<Chunk>>,
    stack: Vec<Value>,
    globals: HashMap<String, Value>
}

impl VM {
    pub fn init() -> Self {
        Self {
            ip: 0,
            chunk: Rc::new(RefCell::new(Chunk::new())),
            stack: Vec::new(),
            globals: HashMap::new()
        }
    }

    pub fn free(&mut self) {

    }

    pub fn interpret(&mut self, path: String, source: String) -> Result<(), InterpretError> {
        let mut compiler = Compiler::init(path.clone(), source.clone());
        if !compiler.compile(self.chunk.clone()) {
            return Err(InterpretError::Compile);
        }
        self.ip = 0;
        self.run()
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        loop {
            disassemble_instruction(self.chunk.clone(), self.ip);
            let instruction: Result<OpCode, _> = self.read_byte().try_into();
            match instruction.unwrap() {
                OpCode::Constant => {
                    let constant = self.read_constant();
                    self.stack.push(constant);
                }
                OpCode::True => {
                    self.stack.push(Value::Boolean(true));
                }
                OpCode::False => {
                    self.stack.push(Value::Boolean(false));
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte();
                    self.stack.push(self.stack[slot as usize].clone());
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte();
                    self.stack[slot as usize] = self.peek(0).clone();
                }
                OpCode::GetGlobal => {
                    let name = self.read_string();
                    if let Some(value) = self.globals.get(&name) {
                        self.stack.push(value.clone());
                    } else {
                        self.runtime_error(&format!("Undefined variable {name}."));
                        return Err(InterpretError::Runtime);
                    }
                }
                OpCode::DefineGlobal => {
                    let name = self.read_string();
                    let value = self.peek(0).clone();
                    self.globals.insert(name, value);
                    self.stack.pop();
                }
                OpCode::SetGlobal => {
                    let name = self.read_string();
                    if !self.globals.contains_key(&name) {
                        self.runtime_error(&format!("Undefined variable {name}."));
                        return Err(InterpretError::Runtime);
                    }
                    let value = self.peek(0).clone();
                    self.globals.insert(name, value);
                }
                OpCode::Equal => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::Nil => {
                    self.stack.push(Value::Nil);
                }
                OpCode::Add => {
                    if self.peek(0).is_string() && self.peek(1).is_string() {
                        self.concatenate();
                    } else if self.peek(0).is_number() && self.peek(1).is_number() {
                        let b = self.stack.pop().unwrap().as_number().unwrap();
                        let a = self.stack.pop().unwrap().as_number().unwrap();
                        self.stack.push(Value::Number(a + b))
                    } else {
                        self.runtime_error("Operands must be two numbers or two strings.");
                        return Err(InterpretError::Runtime);
                    }
                }
                op @ (OpCode::Greater | OpCode::Less | OpCode::Subtract | OpCode::Multiply | OpCode::Divide) => {
                    self.binary_op(op)?;
                }
                OpCode::Not => {
                    let value = self.stack.pop().unwrap();
                    self.stack.push(Value::Boolean(is_falsey(&value)));
                }
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
                OpCode::Print => {
                    let value = self.stack.pop().unwrap();
                    print_value(value);
                    println!();
                }
                OpCode::Jump => {
                    let offset = self.read_short();
                    self.ip += offset as usize;
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short();
                    if is_falsey(self.peek(0)) {
                        self.ip += offset as usize;
                    }
                }
                OpCode::Loop => {
                    let offset = self.read_short();
                    self.ip -= offset as usize;
                }
                OpCode::Return => {
                    //self.stack.pop();
                    return Ok(());
                }
            };
        }
    }

    fn read_byte(&mut self) -> u8 {
        self.ip += 1;
        self.chunk.borrow().codes[self.ip - 1]
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.chunk.borrow().constants.values[byte as usize].clone()
    }

    fn read_short(&mut self) -> u16 {
        self.ip += 2;
        ((((self.chunk.borrow().codes[self.ip - 2]) as u32) << 8) | (self.chunk.borrow().codes[self.ip-1]) as u32) as u16
    }

    fn read_string(&mut self) -> String {
        self.read_constant().to_string()
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

    fn concatenate(&mut self) {
        let b = self.stack.pop().unwrap().as_string().unwrap();
        let a = self.stack.pop().unwrap().as_string().unwrap();
        self.stack.push(Value::Obj(Rc::new(RefCell::new(Obj::String(a + &b)))));
    }

    fn peek(&mut self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - 1 - distance]
    }

    fn runtime_error(&mut self, message: &str) {
        println!("{message}");
        let line = self.chunk.borrow().lines[self.chunk.borrow().lines.len()-2];
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