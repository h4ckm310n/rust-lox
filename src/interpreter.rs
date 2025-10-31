use crate::environment::Environment;
use crate::function::Function;
use crate::callable::Callable;
use crate::token::{Literal, Token, TokenType};
use crate::visit::*;
use crate::ast::{expr::*, stmt::*};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<Expr, usize>,
}

impl Visitor for Interpreter {
    type R = Value;
    type E = ErrType;

    fn visit_literal_expr(&mut self, literal_expr: &LiteralExpr) -> Result<Option<Self::R>, Self::E> {
        Ok(Some(Value::literal_to_value(&literal_expr.content)))
    }

    fn visit_unary_expr(&mut self, unary_expr: &UnaryExpr) -> Result<Option<Self::R>, Self::E> {
        let value = if let Some(value) = self.visit_expr(&*unary_expr.expr)? {
            value
        } else {
            return Err(ErrType::Err(unary_expr.op.clone(), "".to_string()));
        };

        match unary_expr.op.token_type {
            TokenType::Minus => {
                self.check_number_operand(&unary_expr.op, &value)?;
                return Ok(Some(Value::Number(-value.to_number().unwrap())));
            },
            TokenType::Bang => {
                return Ok(Some(Value::Bool(!self.is_truthy(value))));
            },
            _ => {}
        };
        Ok(None)
    }

    fn visit_binary_expr(&mut self, binary_expr: &BinaryExpr) -> Result<Option<Self::R>, Self::E> {
        let _lhs = self.visit_expr(&*binary_expr.lhs)?;
        let _rhs = self.visit_expr(&*binary_expr.rhs)?;
        if _lhs.is_none() || _rhs.is_none() {
            return Err(ErrType::Err(binary_expr.op.clone(), "".to_string()));
        }
        let (lhs, rhs) = (_lhs.as_ref().unwrap(), _rhs.as_ref().unwrap());
        
        match binary_expr.op.token_type {
            TokenType::Minus => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() - rhs.to_number().unwrap();
                return Ok(Some(Value::Number(result)));
            },
            TokenType::Slash => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() / rhs.to_number().unwrap();
                return Ok(Some(Value::Number(result)));
            },
            TokenType::Star => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() * rhs.to_number().unwrap();
                return Ok(Some(Value::Number(result)));
            },
            TokenType::Plus => {
                if let (Some(lhs), Some(rhs)) = (lhs.to_number(), rhs.to_number()) {
                    let result = lhs + rhs;
                    return Ok(Some(Value::Number(result)));
                } else if let (Some(lhs), Some(rhs)) = (lhs.to_string(), rhs.to_string()) {
                    let result = lhs + &rhs;
                    return Ok(Some(Value::String(result)));
                } else {
                    return Err(ErrType::Err(binary_expr.op.clone(), "Operands must be two numbers or two strings.".to_string()));
                }
            },

            TokenType::Greater => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() > rhs.to_number().unwrap();
                return Ok(Some(Value::Bool(result)));
            },
            TokenType::GreaterEqual => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() >= rhs.to_number().unwrap();
                return Ok(Some(Value::Bool(result)));
            },
            TokenType::Less => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() < rhs.to_number().unwrap();
                return Ok(Some(Value::Bool(result)));
            },
            TokenType::LessEqual => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() <= rhs.to_number().unwrap();
                return Ok(Some(Value::Bool(result)));
            },

            TokenType::BangEqual => {
                return Ok(Some(Value::Bool(!self.is_equal(lhs, rhs))));
            },
            TokenType::EqualEqual => {
                return Ok(Some(Value::Bool(self.is_equal(lhs, rhs))));
            }

            _ => {}
        }

        Ok(None)
    }
    
    fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) -> Result<Option<Self::R>, Self::E> {
        let value = {
            if let Some(value) = self.visit_expr(&assign_expr.value)? {
                value
            } else {
                Value::Nil
            }
        };
        if let Some(distance) = self.locals.get(&Expr::Assign(assign_expr.to_owned())) {
            self.environment.borrow_mut().assign_at(*distance, &assign_expr.name, value.to_owned());
        }
        else if let Err((token, message)) = self.environment.borrow_mut().assign(&assign_expr.name, value.to_owned()) {
            return Err(ErrType::Err(token, message));
        };
        Ok(Some(value))
    }

    fn visit_logical_expr(&mut self, logical_expr: &LogicalExpr) -> Result<Option<Self::R>, Self::E> {
        let left = self.visit_expr(&logical_expr.lhs)?;
        if let Some(left) = left {
            if logical_expr.operator.token_type == TokenType::Or {
                if self.is_truthy(left.to_owned()) {
                    return Ok(Some(left));
                }
            } else {
                if !self.is_truthy(left.to_owned()) {
                    return Ok(Some(left));
                }
            }
        }
        self.visit_expr(&logical_expr.rhs)
    }

    fn visit_call_expr(&mut self, call_expr: &CallExpr) -> Result<Option<Self::R>, Self::E> {
        let callee = self.visit_expr(&call_expr.name)?;
        let function = if callee.is_some() && let Value::Function(function) = callee.unwrap() {
            function
        } else {
            return Err(ErrType::Err(call_expr.paren.clone(), "Can only call functions and classes.".to_string()));
        };
        if call_expr.args.len() != function.borrow().arity() {
            return Err(ErrType::Err(call_expr.paren.clone(), format!("Expected {} arguments but got {}.", call_expr.args.len(), function.borrow().arity())));
        }
        let mut arguments = Vec::new();
        for argument in &call_expr.args {
            arguments.push(self.visit_expr(argument)?.unwrap());
        }
        Ok(Some(function.borrow().call(self, arguments)))
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> Result<Option<Self::R>, Self::E> {
        Ok(self.look_up_variable(&identifier.name, Expr::Identifier(identifier.to_owned()))?)
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> Result<Option<Self::R>, Self::E> {
        let value = {
            if let Some(initializer) = &var_decl.initializer &&
               let Some(value) = self.visit_expr(initializer)?
            {
                value
            } else {
                Value::Nil
            }
        };
        self.environment.borrow_mut().define(var_decl.name.text.clone(), value);
        Ok(None)
    }

    fn visit_fun_decl(&mut self, fun_decl: &FunDecl) -> Result<Option<Self::R>, Self::E> {
        let function = Function::new(fun_decl.clone(), self.environment.clone());
        self.environment.borrow_mut().define(
            fun_decl.name.text.clone(), 
            Value::Function(Rc::new(RefCell::new(function))));
        Ok(None)
    }

    fn visit_block(&mut self, block: &Block) -> Result<Option<Self::R>, Self::E> {
        let new_environment = Rc::new(RefCell::new(Environment::new(Some(self.environment.clone()))));
        self.execute_block(&block.stmts, new_environment)?;
        Ok(None)
    }

    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) -> Result<Option<Self::R>, Self::E> {
        if let Some(condition) = self.visit_expr(&if_stmt.condition)? &&
           self.is_truthy(condition) {
            self.visit_stmt(&if_stmt.then_stmt)?;
        } else if let Some(else_stmt) = &if_stmt.else_stmt {
            self.visit_stmt(&**else_stmt)?;
        }
        Ok(None)
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) -> Result<Option<Self::R>, Self::E> {
        while let Some(condition) = self.visit_expr(&while_stmt.condition)? && self.is_truthy(condition) {
            self.visit_stmt(&*while_stmt.stmt)?;
        }
        Ok(None)
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) -> Result<Option<Self::R>, Self::E> {
        let value = if let Some(value) = &return_stmt.value {
            self.visit_expr(value)?.unwrap()
        } else {
            Value::Nil
        };
        Err(ErrType::Return(value))
    }

    fn visit_print_stmt(&mut self, print_stmt: &PrintStmt) -> Result<Option<Self::R>, Self::E> {
        let value = self.visit_expr(&print_stmt.expr)?;
        if let Some(value) = value {
            println!("{}", value.stringify());
        } else {
            println!("nil");
        }
        Ok(None)
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let environment = Rc::new(RefCell::new(Environment::new(None)));
        Self { 
            globals: environment.clone(),
            environment: environment,
            locals: HashMap::new()
        }
    }

    pub fn interpret(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            if let Err(err) = self.visit_stmt(stmt) {

            }
        }
    }

    fn is_truthy(&self, value: Value) -> bool {
        match value {
            Value::Bool(value) => value,
            Value::Nil => false,
            _ => true
        }
    }

    fn is_equal(&self, left: &Value, right: &Value) -> bool {
        if let (Value::Nil, Value::Nil) = (left, right) {
            return true;
        }
        if let Value::Nil = left {
            return false;
        }
        *left == *right
    }

    fn check_number_operand(&self, operator: &Token, operand: &Value) -> Result<(), ErrType> {
        if let Value::Number(_) = operand {
            return Ok(());
        }
        Err(ErrType::Err(operator.clone(), "Operand must be a number.".to_string()))
    }

    fn check_number_operands(&self, operator: &Token, left: &Value, right: &Value) -> Result<(), ErrType> {
        if self.check_number_operand(operator, left).is_ok() && 
           self.check_number_operand(operator, right).is_ok() {
            return Ok(());
        }
        Err(ErrType::Err(operator.clone(), "Operands must be numbers.".to_string()))
    }

    pub fn execute_block(&mut self, stmts: &Vec<Stmt>, environment: Rc<RefCell<Environment>>) -> Result<(), ErrType> {
        let previous = self.environment.clone();
        self.environment = environment;
        for stmt in stmts {
            if let Err(ErrType::Err(token, message)) = self.visit_stmt(stmt) {
                self.environment = previous;
                return Err(ErrType::Err(token, message));
            }
        }
        self.environment = previous;
        Ok(())
    }

    pub fn resolve_depth(&mut self, expr: Expr, depth: usize) {
        self.locals.insert(expr, depth);
    }

    fn look_up_variable(&self, name: &Token, expr: Expr) -> Result<Option<Value>, ErrType> {
        let distance = self.locals.get(&expr);
        if let Some(distance) = distance {
            Ok(Some(self.environment.borrow().get_at(*distance, name.text.clone())))
        } else {
            let variable = self.globals.borrow().get(name);
            if let Err((token, message)) = variable {
                Err(ErrType::Err(token, message))
            } else if let Ok(variable) = variable {
                Ok(Some(variable))
            } else {
                // unreachable
                Ok(None)
            }
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    String(String),
    Number(f64),
    Function(Rc<RefCell<Function>>),
    Nil
}

impl Value {
    fn literal_to_value(literal: &Literal) -> Self {
        match literal {
            Literal::Bool(value) => Self::Bool(value.clone()),
            Literal::String(value) => Self::String(value.clone()),
            Literal::Number(value) => Self::Number(value.parse().unwrap()),
            Literal::Nil => Self::Nil,
        }
    }

    fn to_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(value.clone())
        } else {
            None
        }
    }

    fn to_string(&self) -> Option<String> {
        if let Self::String(value) = self {
            Some(value.clone())
        } else {
            None
        }
    }

    fn to_number(&self) -> Option<f64> {
        if let Self::Number(value) = self {
            Some(value.clone())
        } else {
            None
        }
    }

    fn stringify(&self) -> String {
        match &self {
            Self::Bool(value) => value.to_string(),
            Self::String(value) => value.clone(),
            Self::Number(value) => value.to_string(),
            Self::Function(fun) => fun.borrow().to_string(),
            Self::Nil => "nil".to_string()
        }
    }
}

pub enum ErrType {
    Err(Token, String),
    Return(Value)
}