use crate::token::{Literal, TokenType};
use crate::visit::*;
use crate::ast::{expr::*, stmt::*};

pub struct Interpreter {
    value_stack: Vec<Option<Literal>>
}

impl Visitor for Interpreter {
    fn visit_literal_expr(&mut self, literal_expr: &LiteralExpr) {
        self.value_stack.push(Some(literal_expr.content.clone()));
    }

    fn visit_unary_expr(&mut self, unary_expr: &UnaryExpr) {
        self.visit_expr(&*unary_expr.expr);
        let value = if let Some(value) = self.value_stack.pop().unwrap() {
            value
        } else {
            return;
        };
        match unary_expr.op.token_type {
            TokenType::Minus => {
                if let Literal::Number(value) = value {
                    self.value_stack.push(Some(Literal::Number("-".to_owned()+&value)));
                    return;
                }
            },
            TokenType::Bang => {
                self.value_stack.push(Some(Literal::Bool(!self.is_truthy(value))));
                return;
            },
            _ => {}
        };
        self.value_stack.push(None);
    }

    fn visit_binary_expr(&mut self, binary_expr: &BinaryExpr) {
        self.visit_expr(&*binary_expr.lhs);
        let lhs = self.value_stack.pop().unwrap();
        self.visit_expr(&*binary_expr.rhs);
        let rhs = self.value_stack.pop().unwrap();
        if lhs.is_none() || rhs.is_none() {
            return;
        }

        let (left, right) = {
            if let (Literal::Number(lhs), Literal::Number(rhs)) = (lhs.as_ref().unwrap(), rhs.as_ref().unwrap()) {
                (lhs.clone(), rhs.clone())
            } else if let (Literal::String(lhs), Literal::String(rhs)) = (lhs.as_ref().unwrap(), rhs.as_ref().unwrap()) {
                (lhs.clone(), rhs.clone())
            } else {
                return;
            }
        };

        let (lhs_number, rhs_number) = (left.parse::<f64>(), right.parse::<f64>());
        
        match binary_expr.op.token_type {
            TokenType::Minus => {
                let result = (lhs_number.unwrap() - rhs_number.unwrap()).to_string();
                self.value_stack.push(Some(Literal::Number(result)));
                return;
            },
            TokenType::Slash => {
                let result = (lhs_number.unwrap() / rhs_number.unwrap()).to_string();
                self.value_stack.push(Some(Literal::Number(result)));
                return;
            },
            TokenType::Star => {
                let result = (lhs_number.unwrap() * rhs_number.unwrap()).to_string();
                self.value_stack.push(Some(Literal::Number(result)));
                return;
            },
            TokenType::Plus => {
                if let (Ok(lhs_number), Ok(rhs_number)) = (lhs_number, rhs_number) {
                    let result = (lhs_number + rhs_number).to_string();
                    self.value_stack.push(Some(Literal::Number(result)));
                } else {
                    self.value_stack.push(Some(Literal::String(left+&right)));
                }
                return;
            },

            TokenType::Greater => {
                let result = lhs_number.unwrap() > rhs_number.unwrap();
                self.value_stack.push(Some(Literal::Bool(result)));
                return;
            },
            TokenType::GreaterEqual => {
                let result = lhs_number.unwrap() >= rhs_number.unwrap();
                self.value_stack.push(Some(Literal::Bool(result)));
                return;
            },
            TokenType::Less => {
                let result = lhs_number.unwrap() < rhs_number.unwrap();
                self.value_stack.push(Some(Literal::Bool(result)));
                return;
            },
            TokenType::LessEqual => {
                let result = lhs_number.unwrap() <= rhs_number.unwrap();
                self.value_stack.push(Some(Literal::Bool(result)));
                return;
            },

            TokenType::BangEqual => {
                self.value_stack.push(Some(Literal::Bool(!self.is_equal(lhs.unwrap(), rhs.unwrap()))));
                return;
            },
            TokenType::EqualEqual => {
                self.value_stack.push(Some(Literal::Bool(self.is_equal(lhs.unwrap(), rhs.unwrap()))));
                return;
            }

            _ => {}
        }

        self.value_stack.push(None);
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self { 
            value_stack: Vec::new()
        }
    }

    fn is_truthy(&self, value: Literal) -> bool {
        match value {
            Literal::Bool(value) => value,
            Literal::Nil => false,
            _ => true
        }
    }

    fn is_equal(&self, left: Literal, right: Literal) -> bool {
        if let (Literal::Nil, Literal::Nil) = (&left, &right) {
            return true;
        }
        if let Literal::Nil = &left {
            return false;
        }
        left == right
    }
}