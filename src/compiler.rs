use std::{cell::RefCell, rc::{Rc, Weak}, sync::{Mutex, OnceLock}};

use crate::{chunk::{Chunk, OpCode}, object::{Function, Obj}, scanner::{Scanner, Token, TokenType}, value::Value};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub struct Compiler {
    weak_self: Option<Weak<RefCell<Self>>>,
    enclosing: Option<Weak<RefCell<Compiler>>>,
    current_class: RefCell<Option<Rc<RefCell<ClassCompiler>>>>,
    function: Rc<RefCell<Function>>,
    function_type: FunctionType,
    locals: RefCell<[Option<Local>; u8::MAX as usize + 1]>,
    local_count: RefCell<usize>,
    upvalues: RefCell<[Option<Upvalue>; u8::MAX as usize + 1]>,
    scope_depth: RefCell<usize>
}

impl Compiler {
    pub fn new(file_path: String, source: String, function_type: FunctionType, enclosing: Option<Weak<RefCell<Compiler>>>) -> Self {
        if enclosing.is_none() {
            Parser::instance().lock().unwrap().init(file_path, source);
        }
        let current_class = if let Some(enclosing) = &enclosing {
            enclosing.upgrade().unwrap().borrow().current_class.clone()
        } else {
            RefCell::new(None)
        };
        Self {
            weak_self: None,
            enclosing: enclosing,
            current_class: current_class,
            function: Rc::new(RefCell::new(Function::new())),
            function_type: function_type,
            locals: RefCell::new([const { None }; u8::MAX as usize + 1]),
            local_count: RefCell::new(0),
            upvalues: RefCell::new([const { None }; u8::MAX as usize + 1]),
            scope_depth: RefCell::new(0)
        }
    }

    pub fn init(&mut self) {
        if self.function_type != FunctionType::Script {
            let parser = Parser::instance().lock().unwrap();
            let previous = parser.previous.clone().unwrap();
            let chars = &parser.scanner.source[previous.start..previous.start+previous.length];
            self.function.borrow_mut().name = String::from_iter(chars);
        }
        self.locals.borrow_mut()[0] = Some(Local {
            name: if self.function_type == FunctionType::Function {
                Token { token_type: TokenType::Nil, start: 0, length: 0, line: 0, text: None }
            } else {
                Token { token_type: TokenType::Nil, start: 0, length: 0, line: 0, text: Some("this".to_string()) }
            },
            depth: 0,
            is_captured: false
        });
        *self.local_count.borrow_mut() += 1;
    }

    pub fn set_weak_self(&mut self, weak_self: Weak<RefCell<Self>>) {
        self.weak_self = Some(weak_self);
    }

    pub fn compile(&self) -> Option<Rc<RefCell<Function>>> {
        Parser::instance().lock().unwrap().advance();
        loop {
            let is_match = {
                Parser::instance().lock().unwrap().is_match(TokenType::Eof)
            };
            if is_match {
                break;
            }
            self.declaration();
        }
        let function = self.end();
        if Parser::instance().lock().unwrap().had_error {
            None
        } else {
            Some(function)
        }
    }

    fn declaration(&self) {
        if { Parser::instance().lock().unwrap().is_match(TokenType::Class) } {
            self.class_declaration();
        } else if { Parser::instance().lock().unwrap().is_match(TokenType::Fun) } {
            self.fun_declaration();
        } else if { Parser::instance().lock().unwrap().is_match(TokenType::Var) } {
            self.var_declaration();
        } else {
            self.statement();
        }
        if { Parser::instance().lock().unwrap().panic_mode } {
            Parser::instance().lock().unwrap().synchronize();
        }
    }

    fn class_declaration(&self) {
        Parser::instance().lock().unwrap().consume(TokenType::Identifier, "Expect class name.");
        let class_name = { Parser::instance().lock().unwrap().previous.as_ref().unwrap().clone() };
        let previous = { Parser::instance().lock().unwrap().previous.as_ref().unwrap().clone() };
        let name_constant = self.identifier_constant(previous);
        self.declare_variable();
        self.emit_bytes(OpCode::Class.into(), name_constant);
        self.define_variable(name_constant);
        let class_compiler = Rc::new(RefCell::new(ClassCompiler::new(self.current_class.borrow().clone())));
        *self.current_class.borrow_mut() = Some(class_compiler.clone());

        if { Parser::instance().lock().unwrap().is_match(TokenType::Less) } {
            Parser::instance().lock().unwrap().consume(TokenType::Identifier, "Expect superclass name.");
            self.variable(false);
            let previous = { Parser::instance().lock().unwrap().previous.as_ref().unwrap().clone() };
            if self.identifiers_equal(&class_name, &previous) {
                Parser::instance().lock().unwrap().error("A class can't inherit from itself.");
            }
            self.begin_scope();
            self.add_local(Token { token_type: TokenType::Nil, start: 0, length: 0, line: 0, text: Some("super".to_string()) });
            self.define_variable(0);
            self.named_variable(class_name.clone(), false);
            self.emit_byte(OpCode::Inherit.into());
            class_compiler.borrow_mut().has_super_class = true;
        }

        self.named_variable(class_name, false);
        Parser::instance().lock().unwrap().consume(TokenType::LeftBrace, "Expect '{' before class body.");
        loop {
            {
                let parser = Parser::instance().lock().unwrap();
                if parser.check(TokenType::RightBrace) || parser.check(TokenType::Eof) {
                    break;
                }
            }
            self.method();
        }
        Parser::instance().lock().unwrap().consume(TokenType::RightBrace, "Expect '}' after class body.");
        self.emit_byte(OpCode::Pop.into());
        if class_compiler.borrow().has_super_class {
            self.end_scope();
        }
        *self.current_class.borrow_mut() = class_compiler.borrow().enclosing.clone();
    }

    fn fun_declaration(&self) {
        let global = self.parse_variable("Expect function name.");
        self.mark_initialized();
        self.function(FunctionType::Function);
        self.define_variable(global);
    }

    fn var_declaration(&self) {
        let global = self.parse_variable("Expect variable name.");
        if { Parser::instance().lock().unwrap().is_match(TokenType::Equal) } {
            self.expression();
        } else {
            self.emit_byte(OpCode::Nil.into());
        }
        Parser::instance().lock().unwrap().consume(TokenType::Semicolon, "Expect ';' after variable declaration.");
        self.define_variable(global);
    }

    fn statement(&self) {
        if { Parser::instance().lock().unwrap().is_match(TokenType::Print) } {
            self.print_statement();
        } else if { Parser::instance().lock().unwrap().is_match(TokenType::Return) } {
            self.return_statement();
        } else if { Parser::instance().lock().unwrap().is_match(TokenType::If) } {
            self.if_statement();
        } else if { Parser::instance().lock().unwrap().is_match(TokenType::While) } {
            self.while_statement();
        } else if { Parser::instance().lock().unwrap().is_match(TokenType::For) } {
            self.for_statement();
        } else if { Parser::instance().lock().unwrap().is_match(TokenType::LeftBrace) } {
            self.begin_scope();
            self.block();
            self.end_scope();
        } else {
            self.expression_statement();
        }
    }

    fn print_statement(&self) {
        self.expression();
        Parser::instance().lock().unwrap().consume(TokenType::Semicolon, "Expect ';' after value.");
        self.emit_byte(OpCode::Print.into());
    }

    fn return_statement(&self) {
        if self.function_type == FunctionType::Script {
            Parser::instance().lock().unwrap().error("Can't return from top-level code.");
        }
        if { Parser::instance().lock().unwrap().is_match(TokenType::Semicolon) } {
            self.emit_return();
        } else {
            if self.function_type == FunctionType::Initializer {
                Parser::instance().lock().unwrap().error("Can't return a value from an initializer.");
            }
            self.expression();
            Parser::instance().lock().unwrap().consume(TokenType::Semicolon, "Expect ';' after return value.");
            self.emit_byte(OpCode::Return.into());
        }
    }

    fn block(&self) {
        loop {
            let check = {
                let parser = Parser::instance().lock().unwrap();
                !parser.check(TokenType::RightBrace) && !parser.check(TokenType::Eof)
            };
            if !check {
                break;
            }

            self.declaration();
        }
        Parser::instance().lock().unwrap().consume(TokenType::RightBrace, "Expect '}' after block.");
    }

    fn function(&self, function_type: FunctionType) {
        let file_path = {
            Parser::instance().lock().unwrap().scanner.file_path.clone()
        };
        let source = {
            String::from_iter(Parser::instance().lock().unwrap().scanner.source.clone())
        };
        let compiler = Rc::new(RefCell::new(
            Compiler::new(file_path, source, function_type, Some(self.weak_self.as_ref().unwrap().clone()))
        ));
        compiler.borrow_mut().set_weak_self(Rc::downgrade(&compiler));
        compiler.borrow_mut().init();
        compiler.borrow().begin_scope();
        Parser::instance().lock().unwrap().consume(TokenType::LeftParen, "Expect '(' after function name.");
        if { !Parser::instance().lock().unwrap().check(TokenType::RightParen) } {
            loop {
                compiler.borrow().function.borrow_mut().arity += 1;
                if compiler.borrow().function.borrow().arity > u8::MAX as usize {
                    Parser::instance().lock().unwrap().error_at_current("Can't have more than 255 parameters.");
                }
                let constant = compiler.borrow().parse_variable("Expect parameter name.");
                compiler.borrow().define_variable(constant);
                if { !Parser::instance().lock().unwrap().is_match(TokenType::Comma) } {
                    break;
                }
            }
        }
        {
            let mut parser = Parser::instance().lock().unwrap();
            parser.consume(TokenType::RightParen, "Expect ')' after parameters.");
            parser.consume(TokenType::LeftBrace, "Expect '{' before function body.");
        }
        compiler.borrow().block();
        let function = compiler.borrow().end();
        let value = self.make_constant(Value::Obj(Rc::new(Obj::Function(function.clone()))));
        self.emit_bytes(OpCode::Closure.into(), value);

        let upvalues = &compiler.borrow().upvalues;
        for i in 0..function.borrow().upvalue_count {
            let upvalues_ = upvalues.borrow();
            let upvalue = upvalues_[i].as_ref().unwrap();
            self.emit_byte(upvalue.is_local as u8);
            self.emit_byte(upvalue.index as u8);
        }
    }

    fn method(&self) {
        Parser::instance().lock().unwrap().consume(TokenType::Identifier, "Expect method name.");
        let previous = { Parser::instance().lock().unwrap().previous.as_ref().unwrap().clone() };
        let constant = self.identifier_constant(previous);
        let function_type = {
            let parser = Parser::instance().lock().unwrap();
            let previous = parser.previous.as_ref().unwrap();
            if String::from_iter(&parser.scanner.source[previous.start..previous.start+previous.length]) == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            }
        };
        self.function(function_type);
        self.emit_bytes(OpCode::Method.into(), constant);
    }

    fn expression_statement(&self) {
        self.expression();
        Parser::instance().lock().unwrap().consume(TokenType::Semicolon, "Expect ';' after expression.");
        self.emit_byte(OpCode::Pop.into());
    }

    fn if_statement(&self) {
        Parser::instance().lock().unwrap().consume(TokenType::LeftParen, "Expect '(' after 'if'.");
        self.expression();
        Parser::instance().lock().unwrap().consume(TokenType::RightParen, "Expect ')' after condition.");
        let then_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        self.emit_byte(OpCode::Pop.into());
        self.statement();
        let else_jump = self.emit_jump(OpCode::Jump.into());
        self.patch_jump(then_jump);
        self.emit_byte(OpCode::Pop.into());
        
        if { Parser::instance().lock().unwrap().is_match(TokenType::Else) } {
            self.statement();
        }
        self.patch_jump(else_jump);
    }

    fn while_statement(&self) {
        let loop_start = self.function.borrow().chunk.borrow().codes.len();
        Parser::instance().lock().unwrap().consume(TokenType::LeftParen, "Expect '(' after 'while'.");
        self.expression();
        Parser::instance().lock().unwrap().consume(TokenType::RightParen, "Expect ')' after condition.");

        let exit_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        self.emit_byte(OpCode::Pop.into());
        self.statement();
        self.emit_loop(loop_start);

        self.patch_jump(exit_jump);
        self.emit_byte(OpCode::Pop.into());
    }

    fn for_statement(&self) {
        self.begin_scope();
        Parser::instance().lock().unwrap().consume(TokenType::LeftParen, "Expect '(' after 'for'.");
        if { Parser::instance().lock().unwrap().is_match(TokenType::Semicolon) } {

        } else if { Parser::instance().lock().unwrap().is_match(TokenType::Var) } {
            self.var_declaration();
        } else {
            self.expression_statement();
        }

        let mut loop_start = self.function.borrow().chunk.borrow().codes.len();
        let mut exit_jump = -1;
        if { !Parser::instance().lock().unwrap().is_match(TokenType::Semicolon) } {
            self.expression();
            Parser::instance().lock().unwrap().consume(TokenType::Semicolon, "Expect ';' after loop condition.");
            exit_jump = self.emit_jump(OpCode::JumpIfFalse.into()) as i32;
            self.emit_byte(OpCode::Pop.into());
        }

        if { !Parser::instance().lock().unwrap().is_match(TokenType::RightParen) } {
            let body_jump = self.emit_jump(OpCode::Jump.into());
            let increment_start = self.function.borrow().chunk.borrow().codes.len();
            self.expression();
            self.emit_byte(OpCode::Pop.into());
            Parser::instance().lock().unwrap().consume(TokenType::RightParen, "Expect ')' after for clauses.");

            self.emit_loop(loop_start);
            loop_start = increment_start;
            self.patch_jump(body_jump);
        }

        self.statement();
        self.emit_loop(loop_start);

        if exit_jump != -1 {
            self.patch_jump(exit_jump as usize);
            self.emit_byte(OpCode::Pop.into());
        }

        self.end_scope();
    }

    fn expression(&self) {
        self.parse_precedence(Precedence::Assignment);
    }

    fn number(&self) {
        let token = { Parser::instance().lock().unwrap().previous.clone().unwrap() };
        if token.token_type == TokenType::Number {
            let value = {
                let chars = &Parser::instance().lock().unwrap().scanner.source[token.start..token.start+token.length];
                String::from_iter(chars).parse::<f64>()
            };
            if let Ok(value) = value {
                self.emit_constant(Value::Number(value));
            }
        }
    }

    fn grouping(&self) {
        self.expression();
        Parser::instance().lock().unwrap().consume(TokenType::RightParen, "Expect ')' after expression.");
    }

    fn unary(&self) {
        let operator_type = { Parser::instance().lock().unwrap().previous.clone().unwrap().token_type };
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

    fn binary(&self) {
        let operator_type = { Parser::instance().lock().unwrap().previous.clone().unwrap().token_type }.clone();
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

    fn call(&self) {
        let arg_count = self.argument_list();
        self.emit_bytes(OpCode::Call.into(), arg_count as u8);
    }

    fn dot(&self, can_assign: bool) {
        Parser::instance().lock().unwrap().consume(TokenType::Identifier, "Expect property name after '.'.");
        let previous = { Parser::instance().lock().unwrap().previous.as_ref().unwrap().clone() };
        let name = self.identifier_constant(previous);
        if can_assign && { Parser::instance().lock().unwrap().is_match(TokenType::Equal) } {
            self.expression();
            self.emit_bytes(OpCode::SetProperty.into(), name);
        } else if { Parser::instance().lock().unwrap().is_match(TokenType::LeftParen) } {
            let arg_count = self.argument_list();
            self.emit_bytes(OpCode::Invoke.into(), name);
            self.emit_byte(arg_count as u8);
        } else {
            self.emit_bytes(OpCode::GetProperty.into(), name);
        }
    }

    fn this(&self) {
        if self.current_class.borrow().is_none() {
            Parser::instance().lock().unwrap().error("Can't use 'this' outside of a class.");
            return;
        }
        self.variable(false);
    }

    fn super_(&self) {
        let previous = {
            let mut parser = Parser::instance().lock().unwrap();
            if self.current_class.borrow().is_none() {
                parser.error("Can't use 'super' outside of a class.");
            } else if !self.current_class.borrow().as_ref().unwrap().borrow().has_super_class {
                parser.error("Can't use 'super' in a class with no superclass.");
            }
            parser.consume(TokenType::Dot, "Expect '.' after 'super'.");
            parser.consume(TokenType::Identifier, "Expect superclass method name.");
            parser.previous.as_ref().unwrap().clone()
        };
        let name = self.identifier_constant(previous);
        self.named_variable(Token { token_type: TokenType::Nil, start: 0, length: 0, line: 0, text: Some("this".to_string()) }, false);
        if { Parser::instance().lock().unwrap().is_match(TokenType::LeftParen) } {
            let arg_count = self.argument_list();
            self.named_variable(Token { token_type: TokenType::Nil, start: 0, length: 0, line: 0, text: Some("super".to_string()) }, false);
            self.emit_bytes(OpCode::SuperInvoke.into(), name);
            self.emit_byte(arg_count as u8);
        } else {
            self.named_variable(Token { token_type: TokenType::Nil, start: 0, length: 0, line: 0, text: Some("super".to_string()) }, false);
            self.emit_bytes(OpCode::GetSuper.into(), name);
        }
    }

    fn literal(&self) {
        let token_type = { Parser::instance().lock().unwrap().previous.clone().unwrap().token_type };
        match token_type {
            TokenType::True => self.emit_byte(OpCode::True.into()),
            TokenType::False => self.emit_byte(OpCode::False.into()),
            TokenType::Nil => self.emit_byte(OpCode::Nil.into()),
            _ => ()
        }
    }

    fn string(&self) {
        let obj = {
            let parser = Parser::instance().lock().unwrap();
            let previous = parser.previous.as_ref().unwrap();
            let chars = &parser.scanner.source[previous.start+1..previous.start+previous.length-1];
            let value = String::from_iter(chars);
            Rc::new(Obj::String(value))
        };
        self.emit_constant(Value::Obj(obj));
    }

    fn variable(&self, can_assign: bool) {
        let previous = { Parser::instance().lock().unwrap().previous.as_ref().unwrap().clone() };
        self.named_variable(previous, can_assign);
    }

    fn named_variable(&self, name: Token, can_assign: bool) {
        let get_op;
        let set_op;
        let mut arg = self.resolve_local(&name);
        if arg != -1 {
            get_op = OpCode::GetLocal;
            set_op = OpCode::SetLocal;
        } else if let arg_ = self.resolve_upvalue(&name) && arg_ != -1 {
            arg = arg_;
            get_op = OpCode::GetUpvalue;
            set_op = OpCode::SetUpvalue;
        } else {
            arg = self.identifier_constant(name) as i8;
            get_op = OpCode::GetGlobal;
            set_op = OpCode::SetGlobal;
        }
        
        if can_assign && { Parser::instance().lock().unwrap().is_match(TokenType::Equal) } {
            self.expression();
            self.emit_bytes(set_op.into(), arg as u8);

        } else {
            self.emit_bytes(get_op.into(), arg as u8);
        }
    }

    fn parse_precedence(&self, precedence: Precedence) {
        Parser::instance().lock().unwrap().advance();
        let previous = { Parser::instance().lock().unwrap().previous.clone().unwrap() };
        let (prefix, _, _) = self.get_rule(&previous.token_type);
        let can_assign = precedence.clone() as u8 <= Precedence::Assignment as u8;
        match prefix {
            Some(prefix) => self.parse_by_name(prefix.to_owned(), can_assign),
            None => { Parser::instance().lock().unwrap().error("Expect expression.") }
        };        

        loop {
            {
                let (_, _, precedence_) = {
                    let token_type = Parser::instance().lock().unwrap().current.clone().unwrap().token_type;
                    self.get_rule(&token_type)
                };
                if (precedence.clone() as u8) <= (precedence_ as u8) {
                    Parser::instance().lock().unwrap().advance();
                } else {
                    break;
                }
            }
            let (_, infix, _) = {
                let previous = { Parser::instance().lock().unwrap().previous.clone().unwrap() };
                self.get_rule(&previous.token_type)
            };
            if let Some(infix) = infix {
                self.parse_by_name(infix, can_assign);
            }
        }
        if can_assign && { Parser::instance().lock().unwrap().is_match(TokenType::Equal) } {
            Parser::instance().lock().unwrap().error("Invalid assignment target.");
        }
    }

    fn parse_variable(&self, error_message: &str) -> u8 {
        Parser::instance().lock().unwrap().consume(TokenType::Identifier, error_message);
        self.declare_variable();
        if *self.scope_depth.borrow() > 0 {
            return 0;
        }
        let previous = { Parser::instance().lock().unwrap().previous.as_ref().unwrap().clone() };
        self.identifier_constant(previous)
    }

    fn mark_initialized(&self) {
        if *self.scope_depth.borrow() == 0 {
            return;
        }
        if let Some(local) = self.locals.borrow_mut()[*self.local_count.borrow()-1].as_mut() {
            local.depth = *self.scope_depth.borrow() as i8;
        }
    }

    fn identifier_constant(&self, name: Token) -> u8 {
        let value = {
            let chars = { &Parser::instance().lock().unwrap().scanner.source[name.start..name.start+name.length] };
            String::from_iter(chars)
        };
        self.make_constant(Value::Obj(Rc::new(Obj::String(value))))
    }

    fn identifiers_equal(&self, a: &Token, b: &Token) -> bool {
        let parser = Parser::instance().lock().unwrap();
        if let (Some(a_text), Some(b_text)) = (&a.text, &b.text) {
            a_text == b_text
        } else if let Some(b_text) = &b.text {
            String::from_iter(&parser.scanner.source[a.start..a.start+a.length]) == *b_text
        } else {
            parser.scanner.source[a.start..a.start+a.length] == parser.scanner.source[b.start..b.start+b.length]
        }
    }

    fn resolve_local(&self, name: &Token) -> i8 {
        let locals = self.locals.borrow();
        for i in (0..*self.local_count.borrow()).rev() {
            let local = locals[i].as_ref().unwrap();
            if self.identifiers_equal(name, &local.name) {
                if local.depth == -1 {
                    Parser::instance().lock().unwrap().error("Can't read local variable in its own initializer.");
                }
                return i as i8;
            }
        }
        -1
    }

    fn resolve_upvalue(&self, name: &Token) -> i8 {
        if self.enclosing.is_none() {
            return -1;
        }
        let local = self.enclosing.as_ref().unwrap().upgrade().unwrap().borrow().resolve_local(name);
        if local != -1 {
            self.enclosing.as_ref().unwrap().upgrade().unwrap().borrow().locals.borrow_mut()[local as usize].as_mut().unwrap().is_captured = true;
            return self.add_upvalue(local as usize, true);
        }
        let upvalue = self.enclosing.as_ref().unwrap().upgrade().unwrap().borrow().resolve_upvalue(name);
        if upvalue != -1 {
            return self.add_upvalue(upvalue as usize, false);
        }
        -1
    }

    fn add_upvalue(&self, index: usize, is_local: bool) -> i8 {
        let upvalue_count = self.function.borrow().upvalue_count;

        let mut upvalues = self.upvalues.borrow_mut();
        for i in 0..upvalue_count {
            let upvalue = &upvalues[i].as_ref().unwrap();
            if upvalue.index == index && upvalue.is_local == is_local {
                return i as i8;
            }
        }

        if upvalue_count == u8::MAX as usize + 1 {
            Parser::instance().lock().unwrap().error("Too many closure variables in function.");
            return 0;
        }

        upvalues[upvalue_count] = Some(Upvalue {
            index: index,
            is_local: is_local,
        });
        self.function.borrow_mut().upvalue_count += 1;
        upvalue_count as i8
    }

    fn declare_variable(&self) {
        if *self.scope_depth.borrow() == 0 {
            return;
        }
        let name = { Parser::instance().lock().unwrap().previous.as_ref().unwrap().clone() };
        {
            let locals = self.locals.borrow();
            for i in (0..*self.local_count.borrow()).rev() {
                let local = *&locals[i].as_ref().unwrap();
                if local.depth != -1 && local.depth < *self.scope_depth.borrow() as i8 {
                    break;
                }
                
                if self.identifiers_equal(&name, &local.name) {
                    Parser::instance().lock().unwrap().error("Already a variable with this name in this scope.");
                }
            }
        }
        self.add_local(name);
    }

    fn add_local(&self, name: Token) {
        if *self.local_count.borrow() == u8::MAX as usize + 1 {
            Parser::instance().lock().unwrap().error("Too many local variables in function.");
            return;
        }
        let local = Local{ name: name, depth: -1, is_captured: false };
        self.locals.borrow_mut()[*self.local_count.borrow()] = Some(local);
        *self.local_count.borrow_mut() += 1;
    }

    fn define_variable(&self, global: u8) {
        if *self.scope_depth.borrow() > 0 {
            self.mark_initialized();
            return;
        }
        self.emit_bytes(OpCode::DefineGlobal.into(), global);
    }

    fn argument_list(&self) -> usize {
        let mut arg_count = 0;
        if { !Parser::instance().lock().unwrap().check(TokenType::RightParen) } {
            loop {
                self.expression();
                if arg_count == u8::MAX as usize {
                    Parser::instance().lock().unwrap().error("Can't have more than 255 arguments.");
                }
                arg_count += 1;
                if { !Parser::instance().lock().unwrap().is_match(TokenType::Comma) } {
                    break;
                }
            }
        }
        Parser::instance().lock().unwrap().consume(TokenType::RightParen, "Expect ')' after arguments.");
        arg_count
    }

    fn and(&self) {
        let end_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        self.emit_byte(OpCode::Pop.into());
        self.parse_precedence(Precedence::And);
        self.patch_jump(end_jump);
    }

    fn or(&self) {
        let else_jump = self.emit_jump(OpCode::JumpIfFalse.into());
        let end_jump = self.emit_jump(OpCode::Jump.into());
        self.patch_jump(else_jump);
        self.emit_byte(OpCode::Pop.into());
        self.parse_precedence(Precedence::Or);
        self.patch_jump(end_jump);
    }

    fn get_rule(&self, token_type: &TokenType) -> (Option<String>, Option<String>, Precedence) {
        match token_type {
            TokenType::LeftParen => (Some("grouping".to_string()), Some("call".to_string()), Precedence::Call),
            TokenType::RightParen => (None, None, Precedence::None),
            TokenType::LeftBrace => (None, None, Precedence::None),
            TokenType::RightBrace => (None, None, Precedence::None),
            TokenType::Comma => (None, None, Precedence::None),
            TokenType::Dot => (None, Some("dot".to_string()), Precedence::Call),
            TokenType::Minus => (Some("unary".to_string()), Some("binary".to_string()), Precedence::Term),
            TokenType::Plus => (None, Some("binary".to_string()), Precedence::Term),
            TokenType::Semicolon => (None, None, Precedence::None),
            TokenType::Slash => (None, Some("binary".to_string()), Precedence::Factor),
            TokenType::Star => (None, Some("binary".to_string()), Precedence::Factor),
            TokenType::Bang => (Some("unary".to_string()), None, Precedence::None),
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
            TokenType::And => (None, Some("and".to_string()), Precedence::And),
            TokenType::Or => (None, Some("or".to_string()), Precedence::Or),
            TokenType::True => (Some("literal".to_string()), None, Precedence::None),
            TokenType::False => (Some("literal".to_string()), None, Precedence::None),
            TokenType::If => (None, None, Precedence::None),
            TokenType::Else => (None, None, Precedence::None),
            TokenType::For => (None, None, Precedence::None),
            TokenType::While => (None, None, Precedence::None),
            TokenType::Print => (None, None, Precedence::None),
            TokenType::Return => (None, None, Precedence::None),
            TokenType::Super => (Some("super".to_string()), None, Precedence::None),
            TokenType::This => (Some("this".to_string()), None, Precedence::None),
            TokenType::Var => (None, None, Precedence::None),
            TokenType::Class => (None, None, Precedence::None),
            TokenType::Fun => (None, None, Precedence::None),
            TokenType::Nil => (Some("literal".to_string()), None, Precedence::None),
            TokenType::Eof => (None, None, Precedence::None),
        }
    }

    fn parse_by_name(&self, name: String, can_assign: bool) {
        match &name as &str {
            "grouping" => self.grouping(),
            "unary" => self.unary(),
            "binary" => self.binary(),
            "call" => self.call(),
            "number" => self.number(),
            "literal" => self.literal(),
            "string" => self.string(),
            "variable" => self.variable(can_assign),
            "and" => self.and(),
            "or" => self.or(),
            "dot" => self.dot(can_assign),
            "this" => self.this(),
            "super" => self.super_(),
            _ => ()
        }
    }

    fn emit_byte(&self, byte: u8) {
        let line = { Parser::instance().lock().unwrap().previous.as_ref().unwrap().line };
        self.function.borrow().chunk.borrow_mut().write(byte, line);
    }

    fn emit_bytes(&self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn emit_jump(&self, instruction: u8) -> usize {
        self.emit_byte(instruction);
        self.emit_byte(0xff);
        self.emit_byte(0xff);
        self.function.borrow().chunk.borrow().codes.len() - 2
    }

    fn emit_loop(&self, loop_start: usize) {
        self.emit_byte(OpCode::Loop.into());
        let offset = self.function.borrow().chunk.borrow().codes.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            Parser::instance().lock().unwrap().error("Loop body too large.");
        }
        self.emit_byte(((offset >> 8) & 0xff) as u8);
        self.emit_byte((offset & 0xff) as u8);
    }

    fn emit_constant(&self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(OpCode::Constant.into(), constant);
    }

    fn emit_return(&self) {
        if self.function_type == FunctionType::Initializer {
            self.emit_bytes(OpCode::GetLocal.into(), 0);
        } else {
            self.emit_byte(OpCode::Nil.into());
        }
        self.emit_byte(OpCode::Return.into());
    }

    fn patch_jump(&self, offset: usize) {
        let jump = self.function.borrow().chunk.borrow().codes.len() - offset - 2;
        if jump > u16::MAX as usize {
            Parser::instance().lock().unwrap().error("Too much code to jump over.");
        }
        self.function.borrow().chunk.borrow_mut().codes[offset] = ((jump as u32 >> 8) & 0xff) as u8;
        self.function.borrow().chunk.borrow_mut().codes[offset+1] = jump as u8 & 0xff;
    }

    fn make_constant(&self, value: Value) -> u8 {
        let constant = self.function.borrow().chunk.borrow_mut().add_constant(value);
        if constant > u8::MAX as usize {
            Parser::instance().lock().unwrap().error("Too many constants in one chunk.");
            return 0;
        }
        constant as u8
    }

    fn begin_scope(&self) {
        *self.scope_depth.borrow_mut() += 1;
    }

    fn end_scope(&self) {
        *self.scope_depth.borrow_mut() -= 1;
        while *self.local_count.borrow() > 0 && self.locals.borrow()[*self.local_count.borrow()-1].as_ref().unwrap().depth > *self.scope_depth.borrow() as i8 {
            if self.locals.borrow()[*self.local_count.borrow()-1].as_ref().unwrap().is_captured {
                self.emit_byte(OpCode::CloseUpvalue.into());
            } else {
                self.emit_byte(OpCode::Pop.into());
            }
            *self.local_count.borrow_mut() -= 1;
        }
    }

    fn end(&self) -> Rc<RefCell<Function>> {
        self.emit_return();
        self.function.clone()
    }
}

struct ClassCompiler {
    enclosing: Option<Rc<RefCell<ClassCompiler>>>,
    has_super_class: bool
}

impl ClassCompiler {
    fn new(enclosing: Option<Rc<RefCell<ClassCompiler>>>) -> Self {
        Self {
            enclosing: enclosing,
            has_super_class: false
        }
    }
}

struct Parser {
    scanner: Scanner,
    current: Option<Token>,
    previous: Option<Token>,
    had_error: bool,
    panic_mode: bool
}

impl Parser {
    fn init(&mut self, file_path: String, source: String) {
        self.scanner = Scanner::init(file_path, source);
        self.current = None;
        self.previous = None;
        self.had_error = false;
        self.panic_mode = false;
    }

    fn instance() -> &'static Mutex<Parser> {
        static PARSER: OnceLock<Mutex<Parser>> = OnceLock::new();
        PARSER.get_or_init(|| Mutex::new({
            Parser {
                scanner: Scanner::init("".to_string(), "".to_string()), 
                current: None, 
                previous: None, 
                had_error: false, 
                panic_mode: false 
            }
        }))
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();
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
        let current = self.current.clone();
        self.error_at(current.unwrap(), message);
    }

    fn error(&mut self, message: &str) {
        let previous = self.previous.clone();
        self.error_at(previous.unwrap(), message);
    }

    fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        print!("[line {}] Error", token.line);
        if token.token_type == TokenType::Eof {
            print!(" at end");
        } else {
            print!(" at {} {}", token.length, token.start);
        }
        println!(": {message}");
        self.had_error = true;
    }

    fn error_at_line(&mut self, line: usize, message: String) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        print!("[line {}] Error", line);
        println!(": {message}");
        self.had_error = true;
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;
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

#[derive(Clone)]
struct Local {
    name: Token,
    depth: i8,
    is_captured: bool
}

struct Upvalue {
    index: usize,
    is_local: bool
}

#[derive(PartialEq)]
pub enum FunctionType {
    Function,
    Initializer,
    Method,
    Script
}
