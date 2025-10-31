use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{ast::{expr::*, stmt::*}, error::ErrorReporter, interpreter::Interpreter, token::Token, visit::*};

pub struct Resolver {
    interpreter: Rc<RefCell<Interpreter>>,
    scope_stack: Vec<HashMap<String, bool>>
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

    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> Result<Option<Self::R>, Self::E> {
        self.declare(var_decl.name.clone());
        self.default_visit_var_decl(var_decl)?;
        self.define(var_decl.name.clone());
        Ok(None)
    }

    fn visit_fun_decl(&mut self, fun_decl: &FunDecl) -> Result<Option<Self::R>, Self::E> {
        self.declare(fun_decl.name.clone());
        self.define(fun_decl.name.clone());
        self.resolve_function(fun_decl);
        Ok(None)
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) -> Result<Option<Self::R>, Self::E> {
        self.declare(class_decl.name.clone());
        self.define(class_decl.name.clone());
        if let Some(superclass) = &class_decl.superclass && superclass.name == class_decl.name {
            self.error(superclass.name.start, superclass.name.end, "A class can't inherit from itself.".to_string());
            return Err((superclass.name.clone(), "".to_string()));
        }
        self.begin_scope();
        self.scope_stack.last_mut().unwrap().insert("this".to_string(), true);
        self.default_visit_class_decl(class_decl)?;
        self.end_scope();
        Ok(None)
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> Result<Option<Self::R>, Self::E> {
        if let Some(scope) = self.scope_stack.last()
           && let Some(state) = scope.get(&identifier.name.text) 
           && *state == false {
            self.error(identifier.name.start, identifier.name.end, "Can't read local variable in its own initializer.".to_string());
            return Err((identifier.name.clone(), "".to_string()));
        }
        self.resolve_local(Expr::Identifier(identifier.clone()), identifier.name.clone());
        self.default_visit_identifier(identifier)?;
        Ok(None)
    }

    fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_assign_expr(assign_expr)?;
        self.resolve_local(Expr::Assign(assign_expr.clone()), assign_expr.name.clone());
        Ok(None)
    }
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<Interpreter>>) -> Self {
        Self {
            interpreter: interpreter,
            scope_stack: Vec::new()
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

    fn resolve_local(&mut self, expr: Expr, name: Token) {
        let mut i = self.scope_stack.len();
        for scope in self.scope_stack.iter().rev() {
            i -= 1;
            if scope.contains_key(&name.text) {
                self.interpreter.borrow_mut().resolve_depth(expr.clone(),self.scope_stack.len()-1-i);
            }
        }
    }

    fn resolve_function(&mut self, fun_decl: &FunDecl) {
        self.begin_scope();
        for param in &fun_decl.params {
            self.declare(param.clone());
            self.define(param.clone());
        }
        self.default_visit_fun_decl(fun_decl);
        self.end_scope();
    }
}

impl ErrorReporter for Resolver {
    fn error(&self, start: usize, end: usize, error_content: String) {
        
    }
}