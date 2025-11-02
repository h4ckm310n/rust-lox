use std::cell::RefCell;
use crate::error::ErrorReporter;
use crate::token::{Literal, Token, TokenType};
use crate::ast::expr::*;
use crate::ast::stmt::*;

pub struct Parser {
    file_path: String,
    tokens: Vec<Token>,
    current: RefCell<usize>,
    pub had_error: RefCell<bool>,
    errors: RefCell<Vec<(usize, usize, String)>>
}

impl Parser {
    pub fn new(file_path: String, tokens: Vec<Token>) -> Self {
        Self { 
            file_path: file_path,
            tokens: tokens, 
            current: RefCell::new(0),
            had_error: RefCell::new(false),
            errors: RefCell::new(Vec::new())
        }
    }

    pub fn parse(&self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while !self.is_end() {
            if let Ok(stmt) =  self.parse_decl() && let Some(stmt) = stmt {
                stmts.push(stmt);
            }
        }
        stmts
    }
}

// expr
impl Parser {
    fn parse_expr(&self) -> Result<Expr, (Token, String)> {
        self.parse_assignment()
    }

    fn parse_assignment(&self) -> Result<Expr, (Token, String)> {
        let expr = self.parse_logic_or()?;
        if self.is_match(vec![TokenType::Equal]) {
            let equals = self.previous();
            let value = self.parse_assignment()?;
            if let Expr::Identifier(identifier) = expr {
                let name = identifier.name;
                return Ok(Expr::Assign(
                    AssignExpr { 
                        name: name, 
                        value: Box::new(value) 
                    }
                ));
            }
            else if let Expr::Get(expr_get) = expr {
                return Ok(Expr::Set(
                    SetExpr {
                        object: expr_get.object,
                        name: expr_get.name,
                        value: Box::new(value)
                    }
                ));
            }
            return Err(self.handle_error(equals, "Invalid assignment target.".to_string()));
        }
        Ok(expr)
    }

    fn parse_logic_or(&self) -> Result<Expr, (Token, String)> {
        let mut expr = self.parse_logic_and()?;
        while self.is_match(vec![TokenType::Or]) {
            let operator = self.previous();
            let right = self.parse_logic_and()?;
            expr = Expr::Logical(
                LogicalExpr { 
                    operator: operator.clone(), 
                    lhs: Box::new(expr), 
                    rhs: Box::new(right) 
                }
            )
        }
        Ok(expr)
    }

    fn parse_logic_and(&self) -> Result<Expr, (Token, String)> {
        let mut expr = self.parse_equality()?;
        while self.is_match(vec![TokenType::And]) {
            let operator = self.previous();
            let right = self.parse_equality()?;
            expr = Expr::Logical(
                LogicalExpr { 
                    operator: operator.clone(), 
                    lhs: Box::new(expr), 
                    rhs: Box::new(right) 
                }
            )
        }
        Ok(expr)
    }

    fn parse_equality(&self) -> Result<Expr, (Token, String)> {
        let mut expr = self.parse_comparison()?;
        while self.is_match(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.parse_comparison()?;
            expr = Expr::Binary(
                BinaryExpr { 
                    op: operator.clone(),
                    lhs: Box::new(expr),
                    rhs: Box::new(right)
                }
            );
        }
        Ok(expr)
    }

    fn parse_comparison(&self) -> Result<Expr, (Token, String)> {
        let mut expr = self.parse_term()?;
        while self.is_match(vec![TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual]) {
            let operator = self.previous();
            let right = self.parse_term()?;
            expr = Expr::Binary(
                BinaryExpr { 
                    op: operator.clone(),
                    lhs: Box::new(expr),
                    rhs: Box::new(right) 
                }
            );
        }
        Ok(expr)
    }

    fn parse_term(&self) -> Result<Expr, (Token, String)> {
        let mut expr = self.parse_factor()?;
        while self.is_match(vec![TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.parse_factor()?;
            expr = Expr::Binary(
                BinaryExpr { 
                    op: operator.clone(), 
                    lhs: Box::new(expr), 
                    rhs: Box::new(right) 
                }
            )
        }
        Ok(expr)
    }

    fn parse_factor(&self) -> Result<Expr, (Token, String)> {
        let mut expr = self.parse_unary()?;
        while self.is_match(vec![TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.parse_unary()?;
            expr = Expr::Binary(
                BinaryExpr { 
                    op: operator.clone(), 
                    lhs: Box::new(expr), 
                    rhs: Box::new(right) 
                }
            )
        }
        Ok(expr)
    }

    fn parse_unary(&self) -> Result<Expr, (Token, String)> {
        if self.is_match(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.parse_unary()?;
            return Ok(Expr::Unary(
                UnaryExpr {
                    op: operator.clone(), 
                    expr: Box::new(right) 
                }
            ));
        }
        self.parse_call()
    }

    fn parse_call(&self) -> Result<Expr, (Token, String)> {
        let mut expr = self.parse_primary()?;
        loop {
            if self.is_match(vec![TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.is_match(vec![TokenType::Dot]) {
                let name = self.consume(TokenType::Identifier, "Expect property name after '.'.".to_string())?;
                expr = Expr::Get(
                    GetExpr { 
                        object: Box::new(expr), 
                        name: name.clone() 
                    }
                )
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn parse_primary(&self) -> Result<Expr, (Token, String)> {
        if self.is_match(vec![TokenType::False]) {
            return Ok(Expr::Literal(
                LiteralExpr { 
                    content: Literal::Bool(false)
                }
            ));
        }
        if self.is_match(vec![TokenType::True]) {
            return Ok(Expr::Literal(
                LiteralExpr { 
                    content: Literal::Bool(true)
                }
            ));
        }
        if self.is_match(vec![TokenType::Nil]) {
            return Ok(Expr::Literal(
                LiteralExpr { 
                    content: Literal::Nil 
                }
            ))
        }
        if self.is_match(vec![TokenType::String, TokenType::Number]) {
            return Ok(Expr::Literal(
                LiteralExpr { 
                    content: self.previous().literal.clone().unwrap(),
                }
            ));
        }
        if self.is_match(vec![TokenType::Super]) {
            let keyword = self.previous();
            self.consume(TokenType::Dot, "Expect '.' after 'super'.".to_string())?;
            let method = self.consume(TokenType::Identifier, "Expect superclass method name.".to_string())?;
            return Ok(Expr::Super(
                Super { keyword: keyword.clone(), method: method.clone() }
            ));
        }
        if self.is_match(vec![TokenType::This]) {
            return Ok(Expr::This(
                This { keyword: self.previous().clone() }
            ));
        }
        if self.is_match(vec![TokenType::Identifier]) {
            return Ok(Expr::Identifier(
                Identifier { name: self.previous().clone() }
            ))
        }
        if self.is_match(vec![TokenType::LeftParen]) {
            let expr = self.parse_expr()?;
            self.consume(TokenType::RightParen, "Expect ')' afer expression.".to_string())?;
            return Ok(Expr::Grouping(
                GroupingExpr { expr: Box::new(expr) }
            ));
        }
        Err(self.handle_error(self.peek(), "Expect expression.".to_string()))
    } 
}

// stmt
impl Parser {
    fn parse_stmt(&self) -> Result<Stmt, (Token, String)> {
        if self.is_match(vec![TokenType::Print]) {
            return self.parse_print_stmt();
        }
        if self.is_match(vec![TokenType::LeftBrace]) {
            return self.parse_block();
        }
        if self.is_match(vec![TokenType::If]) {
            return self.parse_if_stmt();
        }
        if self.is_match(vec![TokenType::While]) {
            return self.parse_while_stmt();
        }
        if self.is_match(vec![TokenType::For]) {
            return self.parse_for_stmt();
        }
        if self.is_match(vec![TokenType::Return]) {
            return self.parse_return_stmt();
        }
        self.parse_expr_stmt()
    }

    fn parse_print_stmt(&self) -> Result<Stmt, (Token, String)> {
        let expr = self.parse_expr()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.".to_string())?;
        Ok(Stmt::Print(
            PrintStmt {
                expr: expr
            }
        ))
    }

    fn parse_expr_stmt(&self) -> Result<Stmt, (Token, String)> {
        let expr = self.parse_expr()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression.".to_string())?;
        Ok(Stmt::Expr(
            ExprStmt { 
                expr: expr 
            }
        ))
    }

    fn parse_if_stmt(&self) -> Result<Stmt, (Token, String)> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.".to_string())?;
        let condition = self.parse_expr()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.".to_string())?;
        let then_stmt = self.parse_stmt()?;
        let mut else_stmt = None;
        if self.is_match(vec![TokenType::Else]) {
            else_stmt = Some(Box::new(self.parse_stmt()?));
        }
        Ok(Stmt::If(
            IfStmt { 
                condition: condition, 
                then_stmt: Box::new(then_stmt), 
                else_stmt: else_stmt
            }
        ))
    }

    fn parse_while_stmt(&self) -> Result<Stmt, (Token, String)> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.".to_string())?;
        let condition = self.parse_expr()?;
        self.consume(TokenType::RightParen, "Expect ')' after condition.".to_string())?;
        let stmt = self.parse_stmt()?;
        Ok(Stmt::While(
            WhileStmt { 
                condition: condition, 
                stmt: Box::new(stmt)
            }
        ))
    }

    fn parse_for_stmt(&self) -> Result<Stmt, (Token, String)> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.".to_string())?;
        let mut init = None;
        if self.is_match(vec![TokenType::Var]) {
            init = Some(self.parse_var_decl()?);
        }
        else if !self.is_match(vec![TokenType::Semicolon]) {
            init = Some(self.parse_expr_stmt()?);
        }
        let mut condition = None;
        if !self.check(TokenType::Semicolon) {
            condition = Some(self.parse_expr()?);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after loop condition.".to_string())?;
        let mut update = None;
        if !self.check(TokenType::RightParen) {
            update = Some(self.parse_expr()?);
        }
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.".to_string())?;
        let mut stmt = self.parse_stmt()?;

        // desugaring for
        if let Some(update) = update {
            // add update expr after for body
            stmt = Stmt::Block(
                Block { 
                    stmts: vec![
                        stmt, 
                        Stmt::Expr(
                            ExprStmt { expr: update }
                        )
                    ]
                }
            );
        }
        if condition.is_none() {
            // change empty condition into true
            condition = Some(Expr::Literal(LiteralExpr { content: Literal::Bool(true) }));
        }
        // convert for into while
        stmt = Stmt::While(
            WhileStmt { 
                condition: condition.unwrap(), 
                stmt: Box::new(stmt) 
            }
        );
        // add init stmt before while
        if let Some(init) = init {
            stmt = Stmt::Block(
                Block { stmts: vec![init, stmt] }
            )
        }
        Ok(stmt)
    }

    fn parse_return_stmt(&self) -> Result<Stmt, (Token, String)> {
        let keyword = self.previous();
        let mut value = None;
        if !self.check(TokenType::Semicolon) {
            value = Some(self.parse_expr()?);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after return value.".to_string())?;
        Ok(Stmt::Return(
            ReturnStmt { 
                keyword: keyword.clone(),
                value: value 
            }
        ))
    }

    fn parse_block(&self) -> Result<Stmt, (Token, String)> {
        let mut stmts = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_end() {
            if let Some(stmt) = self.parse_decl()? {
                stmts.push(stmt);
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.".to_string())?;
        Ok(Stmt::Block(
            Block { stmts: stmts }
        ))
    }
}

// decl
impl Parser {
    fn parse_decl(&self) -> Result<Option<Stmt>, (Token, String)> {
        if self.is_match(vec![TokenType::Var]) {
            if let Ok(var_decl) = self.parse_var_decl() {
                return Ok(Some(var_decl));
            }
        }
        else if self.is_match(vec![TokenType::Fun]) {
            if let Ok(fun_decl) = self.parse_fun_decl("function".to_string()) {
                return Ok(Some(fun_decl));
            }
        }
        else if self.is_match(vec![TokenType::Class]) {
            if let Ok(class_decl) = self.parse_class_decl() {
                return Ok(Some(class_decl));
            }
        }
        else if let Ok(stmt) = self.parse_stmt() {
            return Ok(Some(stmt));
        }
        self.synchronize();
        Ok(None)
    }

    fn parse_var_decl(&self) -> Result<Stmt, (Token, String)> {
        let identifier = self.consume(TokenType::Identifier, "Expect variable name.".to_string())?;
        let mut initializer = None;
        if self.is_match(vec![TokenType::Equal]) {
            initializer = Some(self.parse_expr()?);
        }
        self.consume(TokenType::Semicolon, "Expect ';' after variable declaration.".to_string())?;
        Ok(Stmt::VarDecl(
            VarDecl {
                name: identifier.clone(),
                initializer: initializer
            }
        ))
    }

    fn parse_fun_decl(&self, kind: String) -> Result<Stmt, (Token, String)> {
        let identifier = self.consume(TokenType::Identifier, "Expect ".to_owned()+&kind+" name.")?;
        self.consume(TokenType::LeftParen, "Expect '(' after ".to_owned()+&kind+" name.")?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(self.handle_error(self.peek(), "Can't have more than 255 parameters.".to_string()));
                }

                params.push(self.consume(TokenType::Identifier, "Expect parameter name.".to_string())?.clone());

                if !self.is_match(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters.".to_string())?;
        self.consume(TokenType::LeftBrace, "Expect '{' before ".to_owned()+&kind+" name.")?;
        let body = self.parse_block()?;
        let mut body_stmts = Vec::new();
        if let Stmt::Block(block) = body {
            body_stmts = block.stmts;
        }
        Ok(Stmt::FunDecl(
            FunDecl { 
                name: identifier.clone(), 
                params: params, 
                body: body_stmts
            }
        ))
    }

    fn parse_class_decl(&self) -> Result<Stmt, (Token, String)> {
        let identifier = self.consume(TokenType::Identifier, "Expect class name.".to_string())?;
        let superclass = if self.is_match(vec![TokenType::Less]) {
            // extends
            self.consume(TokenType::Identifier, "Expect super class name.".to_string())?;
            Some(Identifier { name: self.previous().clone() })
        } else {
            None
        };
        self.consume(TokenType::LeftBrace, "Expect '{' before class body.".to_string())?;
        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_end() {
            if let Stmt::FunDecl(fun_decl) = self.parse_fun_decl("method".to_string())? {
                methods.push(fun_decl);
            }
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body.".to_string())?;
        Ok(Stmt::ClassDecl(
            ClassDecl { 
                name: identifier.clone(),
                superclass: superclass,
                methods: methods 
            }
        ))
    }
}

impl Parser {
    fn is_match(&self, token_types: Vec<TokenType>) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.next_token();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn next_token(&self) -> &Token {
        if !self.is_end() {
            *self.current.borrow_mut() += 1;
        }
        self.previous()
    }

    fn previous(&self) -> &Token {
        self.tokens.get(*self.current.borrow()-1).unwrap()
    }

    fn peek(&self) -> &Token {
        self.tokens.get(*self.current.borrow()).unwrap()
    }

    fn is_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    fn consume(&self, token_type: TokenType, message: String) -> Result<&Token, (Token, String)> {
        if self.check(token_type) {
            return Ok(self.next_token());
        }
        Err(self.handle_error(self.peek(), message.clone()))
    }

    fn handle_error(&self, token: &Token, message: String) -> (Token, String) {
        self.error(token.start, token.end, message.clone());
        (token.clone(), message)
    }

    fn synchronize(&self) {
        self.next_token();
        while !self.is_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class |
                TokenType::Fun |
                TokenType::Var |
                TokenType::For |
                TokenType::If |
                TokenType::While |
                TokenType::Print |
                TokenType::Return => {
                    return;
                }
                _ => {}
            }
            self.next_token();
        }
    }

    fn finish_call(&self, callee: Expr) -> Result<Expr, (Token, String)> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(self.handle_error(self.peek(), "Can't have more than 255 arguments.".to_string()));
                }
                arguments.push(self.parse_expr()?);
                if !self.is_match(vec![TokenType::Comma]) {
                    break;
                }
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments.".to_string())?;
        Ok(Expr::Call(
            CallExpr { 
                name: Box::new(callee), 
                args: arguments,
                paren: paren.clone() 
            }
        ))
    }
}

impl ErrorReporter for Parser {
    fn error(&self, start: usize, end: usize, error_content: String) {
        *self.had_error.borrow_mut() = true;
        println!("Parser error: {} {} {} {}", self.file_path, start, end, error_content);
        self.errors.borrow_mut().push((start, end, error_content));
    }
}