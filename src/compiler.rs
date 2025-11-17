use std::{cell::RefCell, rc::Rc};

use crate::{chunk::{Chunk, OpCode}, object::Obj, scanner::{Scanner, Token, TokenType}, value::Value};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub struct Compiler {
    current_chunk: Option<Rc<RefCell<Chunk>>>,
    parser: Parser
}

impl Compiler {
    pub fn init(file_path: String, source: String) -> Self {
        let scanner = Scanner::init(file_path.clone(), source.clone());
        Self {
            current_chunk: None,
            parser: Parser{
                scanner: scanner,
                current: None,
                previous: None,
                had_error: false
            }
        }
    }

    pub fn compile(&mut self, chunk: Rc<RefCell<Chunk>>) -> bool {
        self.current_chunk = Some(chunk);
        self.parser.advance();
        while !self.parser.is_match(TokenType::Eof) {
            self.declaration();
        }
        //self.expression();
        //self.parser.consume(TokenType::Eof, "Expect end of expression.");
        self.end();
        !self.parser.had_error
    }

    fn declaration(&mut self) {
        if self.parser.is_match(TokenType::Var) {
            self.var_declaration();
        } else {
            self.statement();
        }
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");
        if self.parser.is_match(TokenType::Equal) {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }
        self.parser.consume(TokenType::Semicolon, "Expect ';' after variable declaration.");
        self.define_variable(global);
    }

    fn statement(&mut self) {
        if self.parser.is_match(TokenType::Print) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&mut self) {
        self.expression();
        self.parser.consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print.into());
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.parser.consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&mut self) {
        let token = self.parser.previous.clone().unwrap();
        if token.token_type == TokenType::Number {
            let chars = &self.parser.scanner.source[token.start..token.start+token.lenght];
            let value = String::from_iter(chars).parse::<f64>();
            if let Ok(value) = value {
                self.emit_constant(Value::Number(value));
            }
        }
    }

    fn grouping(&mut self) {
        self.expression();
        self.parser.consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_type = self.parser.previous.clone().unwrap().token_type;
        self.parse_precedence(Precedence::Unary);
        match operator_type {
            TokenType::Bang => {
                self.emit_byte(OpCode::Not.into());
            },
            TokenType::Minus => {
                self.emit_byte(OpCode::Negate.into());
            },
            _ => {}
        }
    }

    fn binary(&mut self) {
        let operator_type = self.parser.previous.clone().unwrap().token_type;
        let (_, _, precedence) = self.get_rule(&operator_type);
        self.parse_precedence((precedence as u8 + 1).try_into().unwrap());
        match operator_type {
            TokenType::BangEqual => self.emit_bytes(OpCode::Equal.into(), OpCode::Not.into()),
            TokenType::EqualEqual => self.emit_byte(OpCode::Equal.into()),
            TokenType::Greater => self.emit_byte(OpCode::Greater.into()),
            TokenType::GreaterEqual => self.emit_bytes(OpCode::Less.into(), OpCode::Not.into()),
            TokenType::Less => self.emit_byte(OpCode::Less.into()),
            TokenType::LessEqual => self.emit_bytes(OpCode::Greater.into(), OpCode::Not.into()),
            TokenType::Plus => self.emit_byte(OpCode::Add.into()),
            TokenType::Minus => self.emit_byte(OpCode::Subtract.into()),
            TokenType::Star => self.emit_byte(OpCode::Multiply.into()),
            TokenType::Slash => self.emit_byte(OpCode::Divide.into()),
            _ => ()
        };
    }

    fn literal(&mut self) {
        match self.parser.previous.clone().unwrap().token_type {
            TokenType::True => self.emit_byte(OpCode::True.into()),
            TokenType::False => self.emit_byte(OpCode::False.into()),
            TokenType::Nil => self.emit_byte(OpCode::Nil.into()),
            _ => ()
        }
    }

    fn string(&mut self) {
        let previous = self.parser.previous.as_ref().unwrap();
        let chars = &self.parser.scanner.source[previous.start+1..previous.start+previous.lenght-1];
        let value = String::from_iter(chars);
        let obj = Rc::new(RefCell::new(Obj::String(value)));
        self.emit_constant(Value::Obj(obj));
    }

    fn variable(&mut self, can_assign: bool) {
        let previous = self.parser.previous.as_ref().unwrap().clone();
        self.name_variable(previous, can_assign);
    }

    fn name_variable(&mut self, name: Token, can_assign: bool) {
        let arg = self.identifier_constant(name);
        if can_assign && self.parser.is_match(TokenType::Equal) {
            self.expression();
            self.emit_bytes(OpCode::SetGlobal.into(), arg);

        } else {
            self.emit_bytes(OpCode::GetGlobal.into(), arg);
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.parser.advance();
        let (prefix, _, _) = self.get_rule(&self.parser.previous.clone().unwrap().token_type);
        let can_assign = precedence.clone() as u8 <= Precedence::Assignment as u8;
        match prefix {
            Some(prefix) => self.parse_by_name(prefix.to_owned(), can_assign),
            None => self.parser.error_at(self.parser.previous.clone().unwrap(), "Expect expression.")
        };        

        while let (_, _, precedence_) = self.get_rule(&self.parser.current.clone().unwrap().token_type) && (precedence.clone() as u8) <= (precedence_ as u8) {
            self.parser.advance();
            let (_, infix, _) = self.get_rule(&self.parser.previous.clone().unwrap().token_type);
            if let Some(infix) = infix {
                self.parse_by_name(infix, can_assign);
            }
        }
        if can_assign && self.parser.is_match(TokenType::Equal) {
            self.parser.error_at_current("Invalid assignment target.");
        }
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.parser.consume(TokenType::Identifier, error_message);
        let previous = self.parser.previous.as_ref().unwrap().clone();
        self.identifier_constant(previous)
    }

    fn identifier_constant(&mut self, name: Token) -> u8 {
        let chars = &self.parser.scanner.source[name.start..name.start+name.lenght];
        let value = String::from_iter(chars);
        self.make_constant(Value::Obj(Rc::new(RefCell::new(Obj::String(value)))))
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(OpCode::DefineGlobal.into(), global);
    }

    fn get_rule(&mut self, token_type: &TokenType) -> (Option<String>, Option<String>, Precedence) {
        match token_type {
            TokenType::LeftParen => (Some("grouping".to_string()), None, Precedence::None),
            TokenType::RightParen => (None, None, Precedence::None),
            TokenType::LeftBrace => (None, None, Precedence::None),
            TokenType::RightBrace => (None, None, Precedence::None),
            TokenType::Comma => (None, None, Precedence::None),
            TokenType::Dot => (None, None, Precedence::None),
            TokenType::Minus => (Some("unary".to_string()), Some("binary".to_string()), Precedence::Term),
            TokenType::Plus => (None, Some("binary".to_string()), Precedence::Term),
            TokenType::Semicolon => (None, None, Precedence::None),
            TokenType::Slash => (None, Some("binary".to_string()), Precedence::Factor),
            TokenType::Star => (None, Some("binary".to_string()), Precedence::Factor),
            TokenType::Bang => (Some("binary".to_string()), None, Precedence::None),
            TokenType::BangEqual => (None, Some("binary".to_string()), Precedence::Equality),
            TokenType::Equal => (None, None, Precedence::None),
            TokenType::EqualEqual => (None, Some("binary".to_string()), Precedence::Equality),
            TokenType::Greater => (None, Some("binary".to_string()), Precedence::Comparison),
            TokenType::GreaterEqual => (None, Some("binary".to_string()), Precedence::Comparison),
            TokenType::Less => (None, Some("binary".to_string()), Precedence::Comparison),
            TokenType::LessEqual => (None, Some("binary".to_string()), Precedence::Comparison),
            TokenType::Identifier => (Some("variable".to_string()), None, Precedence::None),
            TokenType::String => (Some("string".to_string()), None, Precedence::None),
            TokenType::Number => (Some("number".to_string()), None, Precedence::None),
            TokenType::And => (None, None, Precedence::None),
            TokenType::Or => (None, None, Precedence::None),
            TokenType::True => (Some("literal".to_string()), None, Precedence::None),
            TokenType::False => (Some("literal".to_string()), None, Precedence::None),
            TokenType::If => (None, None, Precedence::None),
            TokenType::Else => (None, None, Precedence::None),
            TokenType::For => (None, None, Precedence::None),
            TokenType::While => (None, None, Precedence::None),
            TokenType::Print => (None, None, Precedence::None),
            TokenType::Return => (None, None, Precedence::None),
            TokenType::Super => (None, None, Precedence::None),
            TokenType::This => (None, None, Precedence::None),
            TokenType::Var => (None, None, Precedence::None),
            TokenType::Class => (None, None, Precedence::None),
            TokenType::Fun => (None, None, Precedence::None),
            TokenType::Nil => (Some("literal".to_string()), None, Precedence::None),
            TokenType::Eof => (None, None, Precedence::None),
        }
    }

    fn parse_by_name(&mut self, name: String, can_assign: bool) {
        match &name as &str {
            "grouping" => self.grouping(),
            "unary" => self.unary(),
            "binary" => self.binary(),
            "number" => self.number(),
            "literal" => self.literal(),
            "string" => self.string(),
            "variable" => self.variable(can_assign),
            _ => ()
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        if let Some(chunk) = &self.current_chunk {
            chunk.borrow_mut().write(byte, self.parser.previous.as_ref().unwrap().line);
        }
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant.into(), constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk.clone().unwrap().borrow_mut().add_constant(value);
        constant as u8
    }

    fn end(&mut self) {
        self.emit_byte(OpCode::Return.into());
    }
}

struct Parser {
    scanner: Scanner,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool
}

impl Parser {
    fn advance(&mut self) {
        //self.previous = self.current.clone();
        loop {
            let current = self.scanner.scan_token();
            match current {
                Ok(current) => {
                    self.current = Some(current);
                    break;
                },
                Err((message, line)) => {
                    self.error_at_line(line, message);
                },
            };
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.as_ref().unwrap().token_type == token_type {
            self.advance();
            return;
        }
        self.error_at_current(message);
    }

    fn is_match(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        true
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.current.as_ref().unwrap().token_type == token_type
    }

    fn error_at_current(&mut self, message: &str) {
        let previous = self.previous.clone();
        self.error_at(previous.unwrap(), message);
    }

    fn error_at(&mut self, token: Token, message: &str) {
        print!("[line {}] Error", token.line);
        if token.token_type == TokenType::Eof {
            print!(" at end");
        } else {
            print!("at {} {}", token.lenght, token.start);
        }
        println!(": {message}");
        self.had_error = true;
    }

    fn error_at_line(&mut self, line: usize, message: String) {
        print!("[line {}] Error", line);
        println!(": {message}");
        self.had_error = true;
    }

    fn synchronize(&mut self) {
        while let Some(current) = &self.current && current.token_type != TokenType::Eof {
            if self.previous.as_ref().unwrap().token_type == TokenType::Semicolon {
                return;
            }
            match current.token_type {
                TokenType::Class | TokenType::Fun | TokenType::Var |
                TokenType::For | TokenType::If | TokenType::While | 
                TokenType::Print | TokenType::Return => {
                    return;
                }
                _ => ()
            }
            self.advance();
        }
    }
}

#[derive(Clone, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Unary,
    Call,
    Primary
}
