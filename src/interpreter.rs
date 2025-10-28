use crate::token::{Literal, Token, TokenType};
use crate::visit::*;
use crate::ast::{expr::*, stmt::*};

pub struct Interpreter {
    
}

impl Visitor for Interpreter {
    type R = Literal;

    fn visit_literal_expr(&mut self, literal_expr: &LiteralExpr) -> Result<Option<Self::R>, (Token, String)> {
        Ok(Some(literal_expr.content.clone()))
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
                return Ok(Some(Literal::Number((-value.to_number().unwrap()).to_string())));
            },
            TokenType::Bang => {
                return Ok(Some(Literal::Bool(!self.is_truthy(value))));
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
                let result = (lhs.to_number().unwrap() - rhs.to_number().unwrap()).to_string();
                return Ok(Some(Literal::Number(result)));
            },
            TokenType::Slash => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = (lhs.to_number().unwrap() / rhs.to_number().unwrap()).to_string();
                return Ok(Some(Literal::Number(result)));
            },
            TokenType::Star => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = (lhs.to_number().unwrap() * rhs.to_number().unwrap()).to_string();
                return Ok(Some(Literal::Number(result)));
            },
            TokenType::Plus => {
                if let (Some(lhs), Some(rhs)) = (lhs.to_number(), rhs.to_number()) {
                    let result = (lhs + rhs).to_string();
                    return Ok(Some(Literal::Number(result)));
                } else if let (Some(lhs), Some(rhs)) = (lhs.to_string(), rhs.to_string()) {
                    let result = lhs + &rhs;
                    return Ok(Some(Literal::String(result)));
                } else {
                    return Err((binary_expr.op.clone(), "Operands must be two numbers or two strings.".to_string()));
                }
            },

            TokenType::Greater => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() > rhs.to_number().unwrap();
                return Ok(Some(Literal::Bool(result)));
            },
            TokenType::GreaterEqual => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() >= rhs.to_number().unwrap();
                return Ok(Some(Literal::Bool(result)));
            },
            TokenType::Less => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() < rhs.to_number().unwrap();
                return Ok(Some(Literal::Bool(result)));
            },
            TokenType::LessEqual => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.to_number().unwrap() <= rhs.to_number().unwrap();
                return Ok(Some(Literal::Bool(result)));
            },

            TokenType::BangEqual => {
                return Ok(Some(Literal::Bool(!self.is_equal(lhs, rhs))));
            },
            TokenType::EqualEqual => {
                return Ok(Some(Literal::Bool(self.is_equal(lhs, rhs))));
            }

            _ => {}
        }

        Ok(None)
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self { 
            
        }
    }

    pub fn interpret(&mut self, expr: &Expr) {
        if self.visit_expr(expr).is_ok() {

        } else {
            
        }
    }

    fn is_truthy(&self, value: Literal) -> bool {
        match value {
            Literal::Bool(value) => value,
            Literal::Nil => false,
            _ => true
        }
    }

    fn is_equal(&self, left: &Literal, right: &Literal) -> bool {
        if let (Literal::Nil, Literal::Nil) = (left, right) {
            return true;
        }
        if let Literal::Nil = left {
            return false;
        }
        *left == *right
    }

    fn check_number_operand(&self, operator: &Token, operand: &Literal) -> Result<(), (Token, String)> {
        if let Literal::Number(number) = operand && number.parse::<f64>().is_ok() {
            return Ok(());
        }
        Err((operator.clone(), "Operand must be a number.".to_string()))
    }

    fn check_number_operands(&self, operator: &Token, left: &Literal, right: &Literal) -> Result<(), (Token, String)> {
        if self.check_number_operand(operator, left).is_ok() && 
           self.check_number_operand(operator, right).is_ok() {
            return Ok(());
        }
        Err((operator.clone(), "Operands must be numbers.".to_string()))
    }
}