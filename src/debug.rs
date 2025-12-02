use std::{cell::RefCell, rc::Rc};

use crate::{chunk::{Chunk, OpCode}, value::print_value};

pub fn disassemble_chunk(chunk: Rc<RefCell<Chunk>>, name: String) {
    println!("== {name} ==");
    let mut offset = 0;
    let len = chunk.borrow().codes.len();
    while offset < len {
        offset = disassemble_instruction(chunk.clone(), offset);
    }
}

pub fn disassemble_instruction(chunk: Rc<RefCell<Chunk>>, offset: usize) -> usize {
    print!("{offset} ");
    print!("{} ", chunk.borrow().lines[offset]);
    let instruction: Result<OpCode, _> = chunk.borrow().codes[offset].try_into();
    match instruction.unwrap() {
        OpCode::Constant => {
            constant_instruction("OP_CONSTANT", chunk, offset)
        }
        OpCode::True => {
            simple_instruction("OP_TRUE", offset)
        }
        OpCode::False => {
            simple_instruction("OP_FALSE", offset)
        }
        OpCode::Nil => {
            simple_instruction("OP_NIL", offset)
        }
        OpCode::Pop => {
            simple_instruction("OP_POP", offset)
        }
        OpCode::GetLocal => {
            byte_instruction("OP_GET_LOCAL", chunk, offset)
        }
        OpCode::SetLocal => {
            byte_instruction("OP_SET_LOCAL", chunk, offset)
        }
        OpCode::GetGlobal => {
            constant_instruction("OP_GET_GLOBAL", chunk, offset)
        }
        OpCode::DefineGlobal => {
            constant_instruction("OP_DEFINE_GLOBAL", chunk, offset)
        }
        OpCode::SetGlobal => {
            constant_instruction("OP_SET_GLOBAL", chunk, offset)
        }
        OpCode::Equal => {
            simple_instruction("OP_EQUAL", offset)
        }
        OpCode::Greater => {
            simple_instruction("OP_GREATER", offset)
        }
        OpCode::Less => {
            simple_instruction("OP_LESS", offset)
        }
        OpCode::Add => {
            simple_instruction("OP_ADD", offset)
        }
        OpCode::Subtract => {
            simple_instruction("OP_SUBTRACT", offset)
        }
        OpCode::Multiply => {
            simple_instruction("OP_MULTIPLY", offset)
        }
        OpCode::Divide => {
            simple_instruction("OP_DIVIDE", offset)
        }
        OpCode::Not => {
            simple_instruction("OP_NOT", offset)
        }
        OpCode::Negate => {
            simple_instruction("OP_NEGATE", offset)
        }
        OpCode::Print => {
            simple_instruction("OP_PRINT", offset)
        }
        OpCode::Jump => {
            jump_instruction("OP_JUMP", 1, chunk, offset)
        }
        OpCode::JumpIfFalse => {
            jump_instruction("OP_JUMP_IF_FALSE", 1, chunk, offset)
        }
        OpCode::Loop => {
            jump_instruction("OP_LOOP", -1, chunk, offset)
        }
        OpCode::Call => {
            byte_instruction("OP_CALL", chunk, offset)
        }
        OpCode::Return => {
            simple_instruction("OP_RETURN", offset)
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{name}");
    offset + 1
}

fn constant_instruction(name: &str, chunk: Rc<RefCell<Chunk>>, offset: usize) -> usize {
    let constant = chunk.borrow().codes[offset+1] as usize;
    print!("{name} {constant} ");
    print_value(Rc::new(chunk.borrow().constants.values[constant].clone()));
    println!();
    offset + 2
}

fn byte_instruction(name: &str, chunk: Rc<RefCell<Chunk>>, offset: usize) -> usize {
    let slot = chunk.borrow().codes[offset+1];
    println!("{name} {slot}");
    offset + 2
}

fn jump_instruction(name: &str, sign: i8, chunk: Rc<RefCell<Chunk>>, offset: usize) -> usize {
    let mut jump = ((chunk.borrow().codes[offset+1] as u32) << 8) as i16;
    jump |= chunk.borrow().codes[offset+2] as i16;
    println!("{name} {offset} -> {}", (offset as i16 + 3 + sign as i16 * jump) as u16);
    offset + 3
}