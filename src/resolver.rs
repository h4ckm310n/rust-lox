use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ast::{expr::*, stmt::*}, error::ErrorReporter, interpreter::Interpreter, token::Token, visit::*};

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scope_stack: Vec<HashMap<String, bool>>,
    current_function: Rc<RefCell<FunctionType>>,
    current_class: Rc<RefCell<ClassType>>,
    pub had_error: RefCell<bool>
}

impl Visitor for Resolver {
    type R = i8;
    type E = (Token, String);
    
    fn visit_block(&mut self, block: &Block) -> Result<Option<Self::R>, Self::E> {
        self.begin_scope();
        self.default_visit_block(block)?;
        self.end_scope();
        Ok(None)
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) -> Result<Option<Self::R>, Self::E> {
        if *self.current_function.borrow() == FunctionType::None {
            self.error(return_stmt.keyword.start, return_stmt.keyword.end, "Can't return from top-level code.".to_string());
        }
        if return_stmt.value.is_some() && *self.current_function.borrow() == FunctionType::Initializer {
            self.error(return_stmt.keyword.start, return_stmt.keyword.end, "Can't return a value from initializer.".to_string());
        }
        self.default_visit_return_stmt(return_stmt)
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> Result<Option<Self::R>, Self::E> {
        self.declare(var_decl.name.clone());
        self.default_visit_var_decl(var_decl)?;
        self.define(var_decl.name.clone());
        Ok(None)
    }

    fn visit_fun_decl(&mut self, fun_decl: &FunDecl) -> Result<Option<Self::R>, Self::E> {
        self.declare(fun_decl.name.clone());
        self.define(fun_decl.name.clone());
        self.resolve_function(fun_decl, FunctionType::Function);
        Ok(None)
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) -> Result<Option<Self::R>, Self::E> {
        let enclosing_class = self.current_class.clone();
        self.current_class = Rc::new(RefCell::new(ClassType::Class));
        self.declare(class_decl.name.clone());
        self.define(class_decl.name.clone());
        if let Some(superclass) = &class_decl.superclass && superclass.name == class_decl.name {
            self.error(superclass.name.start, superclass.name.end, "A class can't inherit from itself.".to_string());        
        }
        if let Some(superclass) = &class_decl.superclass {
            self.current_class = Rc::new(RefCell::new(ClassType::SubClass));
            self.visit_identifier(superclass)?;
            self.begin_scope();
            self.scope_stack.last_mut().unwrap().insert("super".to_string(), true);
        }
        self.begin_scope();
        self.scope_stack.last_mut().unwrap().insert("this".to_string(), true);
        for method in &class_decl.methods {
            let declaration = if method.name.text == "init" {
                FunctionType::Initializer
            } else {
                FunctionType::Method
            };
            self.resolve_function(method, declaration);
        }
        self.end_scope();
        if class_decl.superclass.is_some() {
            self.end_scope();
        }
        self.current_class = enclosing_class;
        Ok(None)
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> Result<Option<Self::R>, Self::E> {
        if let Some(scope) = self.scope_stack.last()
           && let Some(state) = scope.get(&identifier.name.text) 
           && *state == false {
            self.error(identifier.name.start, identifier.name.end, "Can't read local variable in its own initializer.".to_string());
        }
        self.resolve_local(Expr::Identifier(identifier.clone()), &identifier.name);
        self.default_visit_identifier(identifier)?;
        Ok(None)
    }

    fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_assign_expr(assign_expr)?;
        self.resolve_local(Expr::Assign(assign_expr.clone()), &assign_expr.name);
        Ok(None)
    }

    fn visit_this(&mut self, this: &This) -> Result<Option<Self::R>, Self::E> {
        if *self.current_class.borrow() == ClassType::None {
            self.error(this.keyword.start, this.keyword.end, "Can't use 'this' outside of a class.".to_string());
            return Ok(None);
        }
        self.resolve_local(Expr::This(this.clone()), &this.keyword);
        Ok(None)
    }

    fn visit_super(&mut self, super_expr: &Super) -> Result<Option<Self::R>, Self::E> {
        if *self.current_class.borrow() == ClassType::None {
            self.error(super_expr.keyword.start, super_expr.keyword.end, "Can't use 'super' outside of a class.".to_string());
        } else if *self.current_class.borrow() != ClassType::SubClass {
            self.error(super_expr.keyword.start, super_expr.keyword.end, "Can't use 'super' in a class with no superclass.".to_string());
        }
        self.resolve_local(Expr::Super(super_expr.clone()), &super_expr.keyword);
        Ok(None)
    }
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Self {
            interpreter: interpreter,
            scope_stack: Vec::new(),
            current_function: Rc::new(RefCell::new(FunctionType::None)),
            current_class: Rc::new(RefCell::new(ClassType::None)),
            had_error: RefCell::new(false)
        }
    }

    pub fn resolve(&mut self, stmts: &Vec<Stmt>) {
        for stmt in stmts {
            self.visit_stmt(stmt);
        }
    }

    fn begin_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scope_stack.pop();
    }

    fn declare(&mut self, name: Token) {
        let mut err = false;
        if let Some(scope) = self.scope_stack.last_mut() {
            if scope.contains_key(&name.text) {
                err = true;
            } else {
                scope.insert(name.text, false);
            }
        }  
        if err {
            self.error(name.start, name.end, "Already a variable with this name in this scope.".to_string());
        }      
    }

    fn define(&mut self, name: Token) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name.text, true);
        }
    }

    fn resolve_local(&mut self, expr: Expr, name: &Token) {
        let mut i = self.scope_stack.len();
        for scope in self.scope_stack.iter().rev() {
            i -= 1;
            if scope.contains_key(&name.text) {
                self.interpreter.borrow_mut().resolve_depth(expr.clone(),self.scope_stack.len()-1-i);
            }
        }
    }

    fn resolve_function(&mut self, fun_decl: &FunDecl, function_type: FunctionType) {
        let enclosing_function = self.current_function.clone();
        self.current_function = Rc::new(RefCell::new(function_type));
        self.begin_scope();
        for param in &fun_decl.params {
            self.declare(param.clone());
            self.define(param.clone());
        }
        self.default_visit_fun_decl(fun_decl);
        self.end_scope();
        self.current_function = enclosing_function.clone();
    }
}

impl ErrorReporter for Resolver {
    fn error(&self, start: usize, end: usize, error_content: String) {
        *self.had_error.borrow_mut() = true;
        println!("Resolve error: {start} {end}: {error_content}");
    }
}

#[derive(PartialEq)]
enum FunctionType {
    None,
    Function,
    Initializer,
    Method
}

#[derive(PartialEq)]
enum ClassType {
    None,
    Class,
    SubClass
}