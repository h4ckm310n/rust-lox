use std::{cell::RefCell, collections::HashMap};

use crate::{ast::{expr::*, stmt::*}, error::ErrorReporter, token::Token, visit::*};

pub struct Resolver {
    scope_stack: Vec<HashMap<String, bool>>,
    locals: RefCell<HashMap<Expr, usize>>,
}

impl Visitor for Resolver {
    fn visit_block(&mut self, block: &Block) {
        self.begin_scope();
        visit_block(self, block);
        self.end_scope();
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        self.declare(var_decl.name.clone());
        visit_var_decl(self, var_decl);
        self.define(var_decl.name.clone());
    }

    fn visit_fun_decl(&mut self, fun_decl: &FunDecl) {
        self.declare(fun_decl.name.clone());
        self.define(fun_decl.name.clone());
        self.resolve_function(fun_decl);
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) {
        self.declare(class_decl.name.clone());
        self.define(class_decl.name.clone());
        if let Some(superclass) = &class_decl.superclass && superclass.name == class_decl.name {
            self.error(superclass.name.start, superclass.name.end, "A class can't inherit from itself.".to_string());
            return;
        }
        self.begin_scope();
        self.scope_stack.last_mut().unwrap().insert("this".to_string(), true);
        visit_class_decl(self, class_decl);
        self.end_scope();
    }

    fn visit_identifier(&mut self, identifier: &Identifier) {
        if let Some(scope) = self.scope_stack.last()
           && let Some(state) = scope.get(&identifier.name.text) 
           && *state == false {
            self.error(identifier.name.start, identifier.name.end, "Can't read local variable in its own initializer.".to_string());
            return;
        }
        self.resolve_local(Expr::Identifier(identifier.clone()), identifier.name.clone());
        visit_identifier(self, identifier);
    }

    fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) {
        visit_assign_expr(self, assign_expr);
        self.resolve_local(Expr::Assign(assign_expr.clone()), assign_expr.name.clone());
    }
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scope_stack: Vec::new(),
            locals: RefCell::new(HashMap::new())
        }
    }

    pub fn resolve(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts {
            self.visit_stmt(&stmt);
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
                self.resolve_depth(expr.clone(),self.scope_stack.len()-1-i);
            }
        }
    }

    fn resolve_function(&mut self, fun_decl: &FunDecl) {
        self.begin_scope();
        for param in &fun_decl.params {
            self.declare(param.clone());
            self.define(param.clone());
        }
        visit_fun_decl(self, fun_decl);
        self.end_scope();
    }

    fn resolve_depth(&self, expr: Expr, depth: usize) {
        self.locals.borrow_mut().insert(expr, depth);
    }
}

impl ErrorReporter for Resolver {
    fn error(&self, start: usize, end: usize, error_content: String) {
        
    }
}