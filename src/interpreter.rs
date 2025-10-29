use crate::environment::Environment;
use crate::token::{Literal, Token, TokenType};
use crate::visit::*;
use crate::ast::{expr::*, stmt::*};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Interpreter {
    environment: Rc<RefCell<Environment>>
}

impl Visitor for Interpreter {
    type R = Value;

    fn visit_literal_expr(&mut self, literal_expr: &LiteralExpr) -> Result<Option<Self::R>, (Token, String)> {
        Ok(Some(Value::literal_to_value(&literal_expr.content)))
    }

    fn visit_unary_expr(&mut self, unary_expr: &UnaryExpr) -> Result<Option<Self::R>, (Token, String)> {
        let value = if let Some(value) = self.visit_expr(&*unary_expr.expr)? {
            value
        } else {
            return Err((unary_expr.op.clone(), "".to_string()));
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

    fn visit_binary_expr(&mut self, binary_expr: &BinaryExpr) -> Result<Option<Self::R>, (Token, String)> {
        let _lhs = self.visit_expr(&*binary_expr.lhs)?;
        let _rhs = self.visit_expr(&*binary_expr.rhs)?;
        if _lhs.is_none() || _rhs.is_none() {
            return Err((binary_expr.op.clone(), "".to_string()));
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
                    return Err((binary_expr.op.clone(), "Operands must be two numbers or two strings.".to_string()));
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
    
    fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) -> Result<Option<Self::R>, (Token, String)> {
        let value = {
            if let Some(value) = self.visit_expr(&assign_expr.value)? {
                value
            } else {
                Value::Nil
            }
        };
        self.environment.borrow_mut().assign(&assign_expr.name, value.to_owned())?;
        Ok(Some(value))
    }

    fn visit_logical_expr(&mut self, logical_expr: &LogicalExpr) -> Result<Option<Self::R>, (Token, String)> {
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

    fn visit_identifier(&mut self, identifier: &Identifier) -> Result<Option<Self::R>, (Token, String)> {
        Ok(Some(self.environment.borrow().get(&identifier.name)?))
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> Result<Option<Self::R>, (Token, String)> {
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

    fn visit_block(&mut self, block: &Block) -> Result<Option<Self::R>, (Token, String)> {
        let new_environment = Rc::new(RefCell::new(Environment::new(Some(self.environment.clone()))));
        self.execute_block(&block.stmts, new_environment)?;
        Ok(None)
    }

    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) -> Result<Option<Self::R>, (Token, String)> {
        if let Some(condition) = self.visit_expr(&if_stmt.condition)? &&
           self.is_truthy(condition) {
            self.visit_stmt(&if_stmt.then_stmt)?;
        } else if let Some(else_stmt) = &if_stmt.else_stmt {
            self.visit_stmt(&**else_stmt)?;
        }
        Ok(None)
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) -> Result<Option<Self::R>, (Token, String)> {
        while let Some(condition) = self.visit_expr(&while_stmt.condition)? && self.is_truthy(condition) {
            self.visit_stmt(&*while_stmt.stmt)?;
        }
        Ok(None)
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self { 
            environment: Rc::new(RefCell::new(Environment::new(None)))
        }
    }

    pub fn interpret(&mut self, expr: &Expr) {
        if self.visit_expr(expr).is_ok() {

        } else {

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

    fn check_number_operand(&self, operator: &Token, operand: &Value) -> Result<(), (Token, String)> {
        if let Value::Number(_) = operand {
            return Ok(());
        }
        Err((operator.clone(), "Operand must be a number.".to_string()))
    }

    fn check_number_operands(&self, operator: &Token, left: &Value, right: &Value) -> Result<(), (Token, String)> {
        if self.check_number_operand(operator, left).is_ok() && 
           self.check_number_operand(operator, right).is_ok() {
            return Ok(());
        }
        Err((operator.clone(), "Operands must be numbers.".to_string()))
    }

    fn execute_block(&mut self, stmts: &Vec<Stmt>, environment: Rc<RefCell<Environment>>) -> Result<(), (Token, String)> {
        let previous = self.environment.clone();
        self.environment = environment;
        for stmt in stmts {
            if let Err((token, message)) = self.visit_stmt(stmt) {
                self.environment = previous;
                return Err((token, message));
            }
        }
        self.environment = previous;
        Ok(())
    }
}

#[derive(Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    String(String),
    Number(f64),
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
}