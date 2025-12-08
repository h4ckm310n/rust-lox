use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{chunk::OpCode, compiler::{Compiler, FunctionType}, debug::disassemble_instruction, object::{Closure, Function, NativeFn, Obj, Upvalue}, value::{Value, print_value}};

pub struct VM {
    frames: Vec<Rc<RefCell<CallFrame>>>,
    stack: Rc<RefCell<Vec<Rc<Value>>>>,
    globals: HashMap<String, Rc<Value>>,
    open_upvalues: Option<Rc<RefCell<Upvalue>>>
}

impl VM {
    pub fn init() -> Self {
        Self {
            frames: Vec::new(),
            stack: Rc::new(RefCell::new(Vec::new())),
            globals: HashMap::new(),
            open_upvalues: None
        }
    }

    pub fn free(&mut self) {

    }

    pub fn interpret(&mut self, path: String, source: String) -> Result<(), InterpretError> {
        self.define_native("clock".to_string(), NativeFn::new("clock".to_string()));
        let compiler = Rc::new(RefCell::new(
            Compiler::new(path.clone(), source.clone(), FunctionType::Script, None)
        ));
        {
        compiler.borrow_mut().set_weak_self(Rc::downgrade(&compiler));
        compiler.borrow_mut().init();
        }
        let function = compiler.borrow().compile();
        if let Some(function) = function {
            let closure = Rc::new(RefCell::new(Closure::new(function)));
            self.push_stack(Rc::new(Value::Obj(Rc::new(RefCell::new(Obj::Closure(closure.clone()))))));
            self.call(closure, 0);
            self.run()
        }
        else {
            return Err(InterpretError::Compile)
        }
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        let mut frame = self.frames.last().unwrap().clone();
        loop {
            disassemble_instruction(frame.borrow().closure.borrow().function.borrow().chunk.clone(), frame.borrow().ip);
            let instruction: Result<OpCode, _> = frame.borrow_mut().read_byte().try_into();
            match instruction.unwrap() {
                OpCode::Constant => {
                    let constant = Rc::new(frame.borrow_mut().read_constant());
                    self.push_stack(constant);
                }
                OpCode::True => {
                    self.push_stack(Rc::new(Value::Boolean(true)));
                }
                OpCode::False => {
                    self.push_stack(Rc::new(Value::Boolean(false)));
                }
                OpCode::Pop => {
                    self.pop_stack();
                }
                OpCode::GetLocal => {
                    let slot = frame.borrow_mut().read_byte();
                    let offset = frame.borrow().slots_offset;
                    let value = frame.borrow().slots.borrow()[offset + slot as usize].clone();
                    self.push_stack(value);
                }
                OpCode::SetLocal => {
                    let slot = frame.borrow_mut().read_byte();
                    let offset = frame.borrow().slots_offset;
                    frame.borrow_mut().slots.borrow_mut()[offset + slot as usize] = self.peek(0).clone();
                }
                OpCode::GetGlobal => {
                    let name = frame.borrow_mut().read_string();
                    if let Some(value) = self.globals.get(&name) {
                        self.push_stack(value.clone());
                    } else {
                        self.runtime_error(&format!("Undefined variable {name}."));
                        return Err(InterpretError::Runtime);
                    }
                }
                OpCode::DefineGlobal => {
                    let name = frame.borrow_mut().read_string();
                    let value = self.peek(0).clone();
                    self.globals.insert(name, value);
                    self.pop_stack();
                }
                OpCode::SetGlobal => {
                    let name = frame.borrow_mut().read_string();
                    if !self.globals.contains_key(&name) {
                        self.runtime_error(&format!("Undefined variable {name}."));
                        return Err(InterpretError::Runtime);
                    }
                    let value = self.peek(0).clone();
                    self.globals.insert(name, value);
                }
                OpCode::GetUpvalue => {
                    let slot = frame.borrow_mut().read_byte() as usize;
                    let closure = &frame.borrow().closure;
                    let location = closure.borrow().upvalues[slot].borrow().location.clone();
                    self.push_stack(location);
                }
                OpCode::SetUpvalue => {
                    let slot = frame.borrow_mut().read_byte() as usize;
                    frame.borrow().closure.borrow_mut().upvalues[slot].borrow_mut().location = self.peek(0);
                }
                OpCode::Equal => {
                    let b = self.pop_stack().unwrap();
                    let a = self.pop_stack().unwrap();
                    self.push_stack(Rc::new(Value::Boolean(a == b)));
                }
                OpCode::Nil => {
                    self.push_stack(Rc::new(Value::Nil));
                }
                OpCode::Add => {
                    if self.peek(0).is_string() && self.peek(1).is_string() {
                        self.concatenate();
                    } else if self.peek(0).is_number() && self.peek(1).is_number() {
                        let b = self.pop_stack().unwrap().as_number().unwrap();
                        let a = self.pop_stack().unwrap().as_number().unwrap();
                        self.push_stack(Rc::new(Value::Number(a + b)))
                    } else {
                        self.runtime_error("Operands must be two numbers or two strings.");
                        return Err(InterpretError::Runtime);
                    }
                }
                op @ (OpCode::Greater | OpCode::Less | OpCode::Subtract | OpCode::Multiply | OpCode::Divide) => {
                    self.binary_op(op)?;
                }
                OpCode::Not => {
                    let value = self.pop_stack().unwrap();
                    self.push_stack(Rc::new(Value::Boolean(is_falsey(value))));
                }
                OpCode::Negate => {
                    let rc_stack = self.stack.clone();
                    let stack = rc_stack.borrow();
                    let value = stack.last().unwrap().clone();
                    if let Some(number) = value.as_number() {
                        self.pop_stack();
                        self.push_stack(Rc::new(Value::Number(-number)));
                    } else {
                        self.runtime_error("Operand must be a number.");
                        return Err(InterpretError::Runtime);
                    }
                }
                OpCode::Print => {
                    let value = self.pop_stack().unwrap();
                    print_value(value);
                    println!();
                }
                OpCode::Jump => {
                    let offset = frame.borrow_mut().read_short();
                    frame.borrow_mut().ip += offset as usize;
                }
                OpCode::JumpIfFalse => {
                    let offset = frame.borrow_mut().read_short();
                    if is_falsey(self.peek(0)) {
                        frame.borrow_mut().ip += offset as usize;
                    }
                }
                OpCode::Loop => {
                    let offset = frame.borrow_mut().read_short();
                    frame.borrow_mut().ip -= offset as usize;
                }
                OpCode::Call => {
                    let arg_count = frame.borrow_mut().read_byte();
                    let value = self.peek(arg_count as usize).clone();
                    if !self.call_value(value, arg_count as usize) {
                        return Err(InterpretError::Runtime);
                    }
                    frame = self.frames.last().unwrap().clone();
                }
                OpCode::Closure => {
                    let function = frame.borrow_mut().read_constant().as_function().unwrap();
                    let closure = Rc::new(RefCell::new(Closure::new(function.clone())));
                    self.push_stack(Rc::new(Value::Obj(Rc::new(RefCell::new(Obj::Closure(closure.clone()))))));
                    for i in 0..function.borrow().upvalue_count {
                        let is_local = frame.borrow_mut().read_byte();
                        let index = frame.borrow_mut().read_byte() as usize;
                        if is_local == 1 {
                            let offset = frame.borrow().slots_offset;
                            let slots = &frame.borrow().slots;
                            let local = &slots.borrow()[offset + index];
                            closure.borrow_mut().upvalues.push(self.capture_upvalue(local.clone()));
                        } else {
                            closure.borrow_mut().upvalues.push(frame.borrow().closure.borrow().upvalues[i].clone());
                        }
                    }
                }
                OpCode::CloseUpvalue => {
                    let last_index = self.stack.borrow().len() - 1;
                    self.close_upvalues( last_index);
                    self.pop_stack();
                }
                OpCode::Return => {
                    let result = self.pop_stack().unwrap();
                    let last_index = frame.borrow().slots_offset;
                    self.close_upvalues(last_index);
                    let last_frame = self.frames.pop().unwrap();
                    if self.frames.len() == 0 {
                        self.pop_stack();
                        return Ok(());
                    }
                    for _ in { 0..self.stack.borrow().len()-last_frame.borrow().slots_offset } {
                        self.pop_stack();
                    }
                    self.push_stack(result);
                    frame = self.frames.last().unwrap().clone();
                }
            };
        }
    }

    fn binary_op(&mut self, op: OpCode) -> Result<(), InterpretError> {
        if !self.peek(0).is_number() || !self.peek(1).is_number() {
            self.runtime_error("Operands must be numbers.");
            return Err(InterpretError::Runtime);
        }
        let b = self.pop_stack().unwrap().as_number().unwrap();
        let a = self.pop_stack().unwrap().as_number().unwrap();
        match op {
            OpCode::Greater => self.push_stack(Rc::new(Value::Boolean(a > b))),
            OpCode::Less => self.push_stack(Rc::new(Value::Boolean(a < b))),
            OpCode::Add => self.push_stack(Rc::new(Value::Number(a + b))),
            OpCode::Subtract => self.push_stack(Rc::new(Value::Number(a - b))),
            OpCode::Multiply => self.push_stack(Rc::new(Value::Number(a * b))),
            OpCode::Divide => self.push_stack(Rc::new(Value::Number(a / b))),
            _ => ()
        };
        Ok(())
    }

    fn concatenate(&mut self) {
        let b = self.pop_stack().unwrap().as_string().unwrap();
        let a = self.pop_stack().unwrap().as_string().unwrap();
        self.push_stack(Rc::new(Value::Obj(Rc::new(RefCell::new(Obj::String(a + &b))))));
    }

    fn peek(&mut self, distance: usize) -> Rc<Value> {
        let rc_stack = self.stack.clone();
        let stack = rc_stack.borrow();
        stack[stack.len() - 1 - distance].clone()
    }

    fn call_value(&mut self, callee: Rc<Value>, arg_count: usize) -> bool {
        if callee.is_obj() {
            if callee.is_closure() {
                return self.call(callee.as_closure().unwrap(), arg_count);
            } else if callee.is_native() {
                let native = callee.as_native().unwrap();
                let result = native.call(arg_count, self.stack.borrow()[self.stack.borrow().len()-arg_count+1..].to_vec());
                self.push_stack(result);
                return true;
            }
        }
        self.runtime_error("Can only call functions and classes.");
        false
    }

    fn call(&mut self, closure: Rc<RefCell<Closure>>, arg_count: usize) -> bool {
        let function = &closure.borrow().function;
        if arg_count != function.borrow().arity {
            self.runtime_error(&format!("Expected {} arguments but got {arg_count}.", function.borrow().arity));
            return false;
        }
        let frame = CallFrame::new(self.stack.clone(), closure.clone(), self.stack.borrow().len() - arg_count - 1);
        self.frames.push(Rc::new(RefCell::new(frame)));
        true
    }

    fn capture_upvalue(&mut self, local: Rc<Value>) -> Rc<RefCell<Upvalue>> {
        let mut prev_upvalue = None;
        let mut upvalue = self.open_upvalues.clone();
        while let Some(upvalue_) = upvalue.clone() && self.compare_values_slot(&upvalue_.borrow().location, &local) == 1 {
            prev_upvalue = Some(upvalue_.clone());
            upvalue = upvalue_.borrow().next.clone();
        }
        if let Some(upvalue_) = upvalue.clone() && upvalue_.borrow().location == local {
            return upvalue_.clone();
        }
        let created_upvalue = Rc::new(RefCell::new(Upvalue::new(local)));
        created_upvalue.borrow_mut().next = upvalue;
        if let Some(prev_upvalue_) = prev_upvalue {
            prev_upvalue_.borrow_mut().next = Some(created_upvalue.clone());
        } else {
            self.open_upvalues = Some(created_upvalue.clone());
        }
        created_upvalue
    }

    fn close_upvalues(&mut self, last_index: usize) {
        let last = &self.stack.borrow()[last_index];
        while let Some(upvalue) = &self.open_upvalues.clone() && { self.compare_values_slot(&upvalue.borrow().location, last) >= 0 } {
            println!("close upvalue {}", upvalue.borrow().location);
            upvalue.borrow_mut().closed = { (*upvalue.borrow().location).clone() };
            upvalue.borrow_mut().location = { Rc::new(upvalue.borrow().closed.clone()) };
            self.open_upvalues = upvalue.borrow().next.clone();
        }
    }

    fn runtime_error(&mut self, message: &str) {
        println!("{message}");
        for i in (0..self.frames.len()).rev() {
            let frame = &self.frames[i];
            let closure = &frame.borrow().closure;
            let function = &closure.borrow().function;
            let line = function.borrow().chunk.borrow().lines[frame.borrow().ip - 1];
            print!("[line {line}] in ");
            if function.borrow().name.is_empty() {
                println!("script");
            } else {
                println!("{}", function.borrow().name);
            }
        }
        self.reset_stack();
    }

    fn define_native(&mut self, name: String, function: NativeFn) {
        let value = Rc::new(Value::Obj(Rc::new(RefCell::new(Obj::NativeFn(Rc::new(function))))));
        self.push_stack(Rc::new(Value::Obj(Rc::new(RefCell::new(Obj::String(name.clone()))))));
        self.push_stack(value.clone());
        self.globals.insert(name, value);
        self.pop_stack();
        self.pop_stack();
    }

    fn compare_values_slot(&self, a: &Rc<Value>, b: &Rc<Value>) -> i8 {
        let stack = self.stack.borrow();
        let pos_a = stack.iter().position(|x| Rc::ptr_eq(x, a)).unwrap();
        let pos_b = stack.iter().position(|x| Rc::ptr_eq(x, b)).unwrap();
        if pos_a == pos_b {
            0
        } else if pos_a > pos_b {
            1
        } else {
            -1
        }
    }

    fn push_stack(&self, value: Rc<Value>) {
        //println!("push: {value}");
        self.stack.borrow_mut().push(value);
    }

    fn pop_stack(&self) -> Option<Rc<Value>> {
        let value = self.stack.borrow_mut().pop();
        //println!("pop: {}", value.clone().unwrap());
        value
    }

    fn reset_stack(&mut self) {
        self.stack.borrow_mut().clear();
    }
}

pub enum InterpretError {
    Compile,
    Runtime
}

fn is_falsey(value: Rc<Value>) -> bool {
    value.is_nil() || (value.is_bool() && !value.as_bool().unwrap())
}

pub struct CallFrame {
    closure: Rc<RefCell<Closure>>,
    ip: usize,
    slots: Rc<RefCell<Vec<Rc<Value>>>>,
    slots_offset: usize
}

impl CallFrame {
    fn new(slots: Rc<RefCell<Vec<Rc<Value>>>>, closure: Rc<RefCell<Closure>>, slots_offset: usize) -> Self {
        Self {
            closure: closure,
            ip: 0,
            slots: slots,
            slots_offset: slots_offset
        }
    }

    fn read_byte(&mut self) -> u8 {
        self.ip += 1;
        self.closure.borrow().function.borrow().chunk.borrow().codes[self.ip - 1]
    }

    fn read_constant(&mut self) -> Value {
        let byte = self.read_byte();
        self.closure.borrow().function.borrow().chunk.borrow().constants.values[byte as usize].clone()
    }

    fn read_short(&mut self) -> u16 {
        self.ip += 2;
        ((((self.closure.borrow().function.borrow().chunk.borrow().codes[self.ip - 2]) as u32) << 8) | (self.closure.borrow().function.borrow().chunk.borrow().codes[self.ip-1]) as u32) as u16
    }

    fn read_string(&mut self) -> String {
        self.read_constant().to_string()
    }
}