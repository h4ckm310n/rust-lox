use crate::{chunk::{Chunk, OpCode}, value::print_value};

pub fn disassemble_chunk(chunk: Chunk, name: String) {
    println!("== {name} ==");
    let mut offset = 0;
    let len = chunk.codes.len();
    while offset < len {
        offset = disassemble_instruction(&chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{offset}");
    print!("{}", chunk.lines[offset]);
    let instruction: Result<OpCode, _> = chunk.codes[offset].try_into();
    match instruction.unwrap() {
        OpCode::Constant => {
            constant_instruction("OP_CONSTANT", chunk, offset)
        },
        OpCode::True => {
            simple_instruction("OP_TRUE", offset)
        },
        OpCode::False => {
            simple_instruction("OP_FALSE", offset)
        },
        OpCode::Nil => {
            simple_instruction("OP_NIL", offset)
        },
        OpCode::Equal => {
            simple_instruction("OP_EQUAL", offset)
        },
        OpCode::Greater => {
            simple_instruction("OP_GREATER", offset)
        },
        OpCode::Less => {
            simple_instruction("OP_LESS", offset)
        },
        OpCode::Add => {
            simple_instruction("OP_ADD", offset)
        },
        OpCode::Subtract => {
            simple_instruction("OP_SUBTRACT", offset)
        },
        OpCode::Multiply => {
            simple_instruction("OP_MULTIPLY", offset)
        },
        OpCode::Divide => {
            simple_instruction("OP_DIVIDE", offset)
        },
        OpCode::Not => {
            simple_instruction("OP_NOT", offset)
        }
        OpCode::Negate => {
            simple_instruction("OP_NEGATE", offset)
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

fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> usize {
    let constant = chunk.codes[offset+1] as usize;
    print!("{name} {constant}");
    print_value(chunk.constants.values[constant].clone());
    println!();
    offset + 2
}