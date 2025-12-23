use crate::array::Array;
use crate::class::Class;
use crate::environment::Environment;
use crate::function::Function;
use crate::callable::Callable;
use crate::instance::Instance;
use crate::native::NativeFunction;
use crate::token::{Literal, Token, TokenType};
use crate::visit::*;
use crate::ast::{expr::*, stmt::*};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<Expr, usize>,
}

impl Visitor for Interpreter {
    type R = Rc<Value>;
    type E = ErrType;

    fn visit_literal_expr(&mut self, literal_expr: &LiteralExpr) -> Result<Option<Self::R>, Self::E> {
        Ok(Some(Rc::new(Value::literal_to_value(&literal_expr.content))))
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
                return Ok(Some(Rc::new(Value::Number(-value.as_number().unwrap()))));
            },
            TokenType::Bang => {
                return Ok(Some(Rc::new(Value::Bool(!self.is_truthy(value)))));
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
                let result = lhs.as_number().unwrap() - rhs.as_number().unwrap();
                return Ok(Some(Rc::new(Value::Number(result))));
            },
            TokenType::Slash => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                if rhs.as_number().unwrap() == 0.0 {
                    return Err(ErrType::Err(binary_expr.op.clone(), "Cannot divide by 0.".to_string()));
                }
                let result = lhs.as_number().unwrap() / rhs.as_number().unwrap();
                return Ok(Some(Rc::new(Value::Number(result))));
            },
            TokenType::Star => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.as_number().unwrap() * rhs.as_number().unwrap();
                return Ok(Some(Rc::new(Value::Number(result))));
            },
            TokenType::Plus => {
                if let (Some(lhs), Some(rhs)) = (lhs.as_number(), rhs.as_number()) {
                    let result = lhs + rhs;
                    return Ok(Some(Rc::new(Value::Number(result))));
                } else if let (Some(lhs), Some(rhs)) = (lhs.as_string(), rhs.as_string()) {
                    let result = lhs + &rhs;
                    return Ok(Some(Rc::new(Value::String(result))));
                } else if let (Some(lhs), Some(rhs)) = (lhs.as_number(), rhs.as_string()) {
                    let result = lhs.to_string() + &rhs;
                    return Ok(Some(Rc::new(Value::String(result))));
                } else if let (Some(lhs), Some(rhs)) = (lhs.as_string(), rhs.as_number()) {
                    let result = lhs + &rhs.to_string();
                    return Ok(Some(Rc::new(Value::String(result))));
                } else {
                    return Err(ErrType::Err(binary_expr.op.clone(), "Operands must be two numbers or two strings.".to_string()));
                }
            },

            TokenType::Greater => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.as_number().unwrap() > rhs.as_number().unwrap();
                return Ok(Some(Rc::new(Value::Bool(result))));
            },
            TokenType::GreaterEqual => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.as_number().unwrap() >= rhs.as_number().unwrap();
                return Ok(Some(Rc::new(Value::Bool(result))));
            },
            TokenType::Less => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.as_number().unwrap() < rhs.as_number().unwrap();
                return Ok(Some(Rc::new(Value::Bool(result))));
            },
            TokenType::LessEqual => {
                self.check_number_operands(&binary_expr.op, lhs, rhs)?;
                let result = lhs.as_number().unwrap() <= rhs.as_number().unwrap();
                return Ok(Some(Rc::new(Value::Bool(result))));
            },

            TokenType::BangEqual => {
                return Ok(Some(Rc::new(Value::Bool(!self.is_equal(lhs, rhs)))));
            },
            TokenType::EqualEqual => {
                return Ok(Some(Rc::new(Value::Bool(self.is_equal(lhs, rhs)))));
            },
            TokenType::Comma => {
                return Ok(Some(rhs.clone()));
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
                Rc::new(Value::Nil)
            }
        };
        if let Some(distance) = self.locals.get(&Expr::Assign(assign_expr.to_owned())) {
            self.environment.borrow_mut().assign_at(*distance, &assign_expr.name, value.clone());
        }
        else if let Err((token, message)) = self.environment.borrow_mut().assign(&assign_expr.name, value.clone()) {
            return Err(ErrType::Err(token, message));
        };
        Ok(Some(value))
    }

    fn visit_ternary_expr(&mut self, ternary_expr: &TernaryExpr) -> Result<Option<Self::R>, Self::E> {
        let condition = self.visit_expr(&*ternary_expr.condition)?.unwrap();
        if self.is_truthy(condition) {
            self.visit_expr(&*ternary_expr.then_expr)
        } else {
            self.visit_expr(&*ternary_expr.else_expr)
        }
    }

    fn visit_logical_expr(&mut self, logical_expr: &LogicalExpr) -> Result<Option<Self::R>, Self::E> {
        let left = self.visit_expr(&logical_expr.lhs)?;
        if let Some(left) = left {
            if logical_expr.operator.token_type == TokenType::Or {
                if self.is_truthy(left.clone()) {
                    return Ok(Some(left));
                }
            } else {
                if !self.is_truthy(left.clone()) {
                    return Ok(Some(left));
                }
            }
        }
        self.visit_expr(&logical_expr.rhs)
    }

    fn visit_call_expr(&mut self, call_expr: &CallExpr) -> Result<Option<Self::R>, Self::E> {
        let callee = self.visit_expr(&call_expr.name)?;
        let callable = if callee.is_some() && let Some(callable) = callee.unwrap().as_callable() {
            callable
        } else {
            return Err(ErrType::Err(call_expr.paren.clone(), "Can only call functions and classes.".to_string()));
        };
        if call_expr.args.len() != callable.borrow().arity() {
            return Err(ErrType::Err(call_expr.paren.clone(), format!("Expected {} arguments but got {}.", call_expr.args.len(), callable.borrow().arity())));
        }
        let mut arguments = Vec::new();
        for argument in &call_expr.args {
            arguments.push(self.visit_expr(argument)?.unwrap());
        }
        Ok(Some(callable.borrow().call(self, arguments)))
    }

    fn visit_get_expr(&mut self, get_expr: &GetExpr) -> Result<Option<Self::R>, Self::E> {
        let object = self.visit_expr(&get_expr.object)?;
        if let Some(object) = object {
            if let Value::Instance(instance) = &*object {
                let binding = instance.borrow();
                let field = binding.get(&get_expr.name)?;
                return Ok(Some(field));
            } else if let Value::Array(array) = &*object {
                // TODO: handle array methods
            }
        }
        Err(ErrType::Err(get_expr.name.clone(), "Only instances have properties.".to_string()))
    }

    fn visit_set_expr(&mut self, set_expr: &SetExpr) -> Result<Option<Self::R>, Self::E> {
        let object = self.visit_expr(&set_expr.object)?;
        if object.is_some() && let Some(instance) = object.unwrap().as_instance() {
            let value = self.visit_expr(&set_expr.value)?;
            if let Some(value) = value.as_ref() {
                instance.borrow_mut().set(&set_expr.name, value.to_owned());
            };
            return Ok(value);
        }
        Err(ErrType::Err(set_expr.name.clone(), "Only instances have fields.".to_string()))
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> Result<Option<Self::R>, Self::E> {
        Ok(self.look_up_variable(&identifier.name, Expr::Identifier(identifier.to_owned()))?)
    }

    fn visit_this(&mut self, this: &This) -> Result<Option<Self::R>, Self::E> {
        Ok(self.look_up_variable(&this.keyword, Expr::This(this.to_owned()))?)
    }

    fn visit_super(&mut self, super_expr: &Super) -> Result<Option<Self::R>, Self::E> {
        let distance = self.locals.get(&Expr::Super(super_expr.to_owned())).unwrap();
        let superclass = self.environment.borrow().get_at(*distance, "super".to_string());
        let instance = self.environment.borrow().get_at(*distance-1, "this".to_string());
        if let (Value::Class(superclass), Value::Instance(instance)) = (&*superclass, &*instance) {
            let method = superclass.borrow().find_method(super_expr.method.text.clone());
            if let Some(method) = method {
                return Ok(Some(Rc::new(Value::Function(Rc::new(RefCell::new(method.borrow().bind(instance.clone())))))));
            } else {
                return Err(ErrType::Err(super_expr.method.clone(), format!("Undefined property'{}'.", super_expr.method.text)));
            }
        }

        // unreachable
        Ok(None)
    }

    fn visit_array_expr(&mut self, array_expr: &ArrayExpr) -> Result<Option<Self::R>, Self::E> {
        let mut elements = Vec::new();
        for element in &array_expr.elements {
            elements.push(self.visit_expr(element)?.unwrap());
        }
        Ok(Some(Rc::new(Value::Array(Rc::new(RefCell::new(Array::new(elements)))))))
    }

    fn visit_subscript_get_expr(&mut self, subscript_get_expr: &SubscriptGetExpr) -> Result<Option<Self::R>, Self::E> {
        let value = self.visit_expr(&*subscript_get_expr.array)?;
        if let Some(value) = value && let Some(array) = value.as_array() {
            let value = self.visit_expr(&*subscript_get_expr.index)?;
            if let Some(value) = value &&
               let Some(index) = value.as_number() && index.fract() == 0.0 {
                let element = array.borrow().get(index as isize);
                if let Some(element) = element {
                    return Ok(Some(element))
                }
                return Err(ErrType::Err(subscript_get_expr.bracket.clone(), "Index out of range.".to_string()));
            }
            return Err(ErrType::Err(subscript_get_expr.bracket.clone(), "Index must be an integer.".to_string()));
        }
        Err(ErrType::Err(subscript_get_expr.bracket.clone(), "Only arrays can be indexed.".to_string()))
    }

    fn visit_subscript_set_expr(&mut self, subscript_set_expr: &SubscriptSetExpr) -> Result<Option<Self::R>, Self::E> {
        let value = self.visit_expr(&*subscript_set_expr.array)?;
        if let Some(value) = value && let Some(array) = value.as_array() {
            let value = self.visit_expr(&*subscript_set_expr.index)?;
            if let Some(value) = value &&
               let Some(index) = value.as_number() && index.fract() == 0.0 {
                let value = self.visit_expr(&*subscript_set_expr.value)?.unwrap();
                if array.borrow_mut().set(index as isize, value.clone()) {
                    return Ok(Some(value));
                }
                return Err(ErrType::Err(subscript_set_expr.bracket.clone(), "Index out of range.".to_string()));
            }
            return Err(ErrType::Err(subscript_set_expr.bracket.clone(), "Index must be an integer.".to_string()));
        }
        Err(ErrType::Err(subscript_set_expr.bracket.clone(), "Only arrays can be indexed.".to_string()))
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> Result<Option<Self::R>, Self::E> {
        let value = {
            if let Some(initializer) = &var_decl.initializer &&
               let Some(value) = self.visit_expr(initializer)?
            {
                value
            } else {
                Rc::new(Value::Nil)
            }
        };
        self.environment.borrow_mut().define(var_decl.name.text.clone(), value);
        Ok(None)
    }

    fn visit_fun_decl(&mut self, fun_decl: &FunDecl) -> Result<Option<Self::R>, Self::E> {
        let function = Function::new(fun_decl.clone(), self.environment.clone(), false);
        self.environment.borrow_mut().define(
            fun_decl.name.text.clone(), 
            Rc::new(Value::Function(Rc::new(RefCell::new(function)))));
        Ok(None)
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) -> Result<Option<Self::R>, Self::E> {
        let superclass = if let Some(identifier) = &class_decl.superclass {
            let superclass = self.visit_identifier(identifier)?;
            if let Some(superclass) = superclass && let Value::Class(superclass) = &*superclass {
                Some(superclass.clone())
            } else {
                return Err(ErrType::Err(identifier.name.clone(), "Superclass must be a class.".to_string()));
            }
        } else {
            None
        };

        self.environment.borrow_mut().define(class_decl.name.text.clone(), Rc::new(Value::Nil));

        if let Some(superclass) = &superclass {
            self.environment = Rc::new(RefCell::new(Environment::new(Some(self.environment.clone()))));
            self.environment.borrow_mut().define("super".to_string(), Rc::new(Value::Class(superclass.clone())));
        }

        let mut methods = HashMap::new();
        for method in &class_decl.methods {
            let is_initializer = method.name.text == "init";
            let function = Function::new(method.clone(), self.environment.clone(), is_initializer);
            methods.insert(method.name.text.clone(), Rc::new(RefCell::new(function)));
        }

        let class = Rc::new(RefCell::new(Class::new(class_decl.name.text.clone(), methods, superclass.clone())));
        class.borrow_mut().set_weak_self(Rc::downgrade(&class));
        
        if superclass.is_some() {
            let enclosing = self.environment.borrow().enclosing.clone().unwrap();
            self.environment = enclosing;
        }

        if let Err((token, message)) = 
            self.environment.borrow_mut().assign(
                &class_decl.name, 
                Rc::new(Value::Class(class.clone()))) {
            return Err(ErrType::Err(token, message));
        }
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
            if let Err(error) = self.visit_stmt(&*while_stmt.stmt) {
                match error {
                    ErrType::Break => break,
                    ErrType::Continue(environment) => {
                        if let Some(update) = &while_stmt.for_update {
                            let previous = self.environment.clone();
                            self.environment = environment; // Enter the environment for update
                            if let Err(error) = self.visit_stmt(&**update) {
                                self.environment = previous;
                                return Err(error);
                            }
                            self.environment = previous;
                        }
                        continue;
                    },
                    _ => return Err(error)
                }
            };
        }
        Ok(None)
    }

    fn visit_break_stmt(&mut self) -> Result<Option<Self::R>, Self::E> {
        Err(ErrType::Break)
    }

    fn visit_continue_stmt(&mut self) -> Result<Option<Self::R>, Self::E> {
        Err(ErrType::Continue(self.environment.clone()))
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) -> Result<Option<Self::R>, Self::E> {
        let value = if let Some(value) = &return_stmt.value {
            self.visit_expr(value)?.unwrap()
        } else {
            Rc::new(Value::Nil)
        };
        Err(ErrType::Return(value))
    }

    fn visit_print_stmt(&mut self, print_stmt: &PrintStmt) -> Result<Option<Self::R>, Self::E> {
        let value = self.visit_expr(&print_stmt.expr)?;
        if let Some(value) = value {
            println!("{value}");
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
        self.globals.borrow_mut().define("clock".to_string(), Rc::new(
            Value::NativeFunction(Rc::new(RefCell::new(NativeFunction::new("clock".to_string()))))
        ));
        for stmt in stmts {
            if let Err(ErrType::Err(token, message)) = self.visit_stmt(stmt) {
                println!("Runtime error: {} {} {}", token.start, token.end, message);
                break;
            }
        }
    }

    fn is_truthy(&self, value: Rc<Value>) -> bool {
        match *value {
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
        self.environment = environment.clone();
        for stmt in stmts {
            let result = self.visit_stmt(stmt);
            if let Err(error) = result {
                self.environment = previous;
                match error {
                    ErrType::Continue(_) => {
                        return Err(ErrType::Continue(environment));
                    }
                    _ => {
                        return Err(error);
                    }
                }
            }
        }
        self.environment = previous;
        Ok(())
    }

    pub fn resolve_depth(&mut self, expr: Expr, depth: usize) {
        self.locals.insert(expr, depth);
    }

    fn look_up_variable(&self, name: &Token, expr: Expr) -> Result<Option<Rc<Value>>, ErrType> {
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
    NativeFunction(Rc<RefCell<NativeFunction>>),
    Class(Rc<RefCell<Class>>),
    Instance(Rc<RefCell<Instance>>),
    Array(Rc<RefCell<Array>>),
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

    fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(value.clone())
        } else {
            None
        }
    }

    fn as_string(&self) -> Option<String> {
        if let Self::String(value) = self {
            Some(value.clone())
        } else {
            None
        }
    }

    fn as_number(&self) -> Option<f64> {
        if let Self::Number(value) = self {
            Some(value.clone())
        } else {
            None
        }
    }

    fn as_instance(&self) -> Option<Rc<RefCell<Instance>>> {
        if let Self::Instance(instance) = self {
            Some(instance.clone())
        } else {
            None
        }
    }

    fn as_callable(&self) -> Option<Rc<RefCell<dyn Callable>>> {
        match self {
            Self::Function(function) => Some(Rc::clone(function) as Rc<RefCell<dyn Callable>>),
            Self::NativeFunction(native_function) => Some(Rc::clone(native_function) as Rc<RefCell<dyn Callable>>),
            Self::Class(class) => Some(Rc::clone(class) as Rc<RefCell<dyn Callable>>),
            _ => None
        }
    }

    fn as_array(&self) -> Option<Rc<RefCell<Array>>> {
        if let Self::Array(array) = self {
            Some(array.clone())
        } else {
            None
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Number(value) => write!(f, "{value}"),
            Self::Function(fun) => write!(f, "{}", fun.borrow().to_string()),
            Self::NativeFunction(_) => write!(f, "<native fn>"),
            Self::Class(class) => write!(f, "{}", class.borrow().to_string()),
            Self::Instance(instance) => write!(f, "{}", instance.borrow().to_string()),
            Self::Array(array) => write!(f, "{}", array.borrow().to_string()),
            Self::Nil => write!(f, "nil")
        }
    }
}

pub enum ErrType {
    Err(Token, String),
    Return(Rc<Value>),
    Break,
    Continue(Rc<RefCell<Environment>>)
}