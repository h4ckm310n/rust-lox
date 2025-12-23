use std::any::Any;
use crate::ast::{expr::*, stmt::*};

pub trait Visitor {
    type R: Any;
    type E: Any;
    
    fn visit_stmt(&mut self, stmt: &Stmt) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_stmt(stmt)
    }

    fn visit_expr_stmt(&mut self, expr_stmt: &ExprStmt) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_expr_stmt(expr_stmt)
    }

    fn visit_print_stmt(&mut self, print_stmt: &PrintStmt) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_print_stmt(print_stmt)
    }

    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_if_stmt(if_stmt)
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_while_stmt(while_stmt)
    }

    fn visit_break_stmt(&mut self) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_break_stmt()
    }

    fn visit_continue_stmt(&mut self) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_continue_stmt()
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_return_stmt(return_stmt)
    }

    fn visit_block(&mut self, block: &Block) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_block(block)
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_var_decl(var_decl)
    }

    fn visit_fun_decl(&mut self, fun_decl: &FunDecl) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_fun_decl(fun_decl)
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_class_decl(class_decl)
    }

    fn visit_expr(&mut self, expr: &Expr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_expr(expr)
    }

    fn visit_binary_expr(&mut self, binary_expr: &BinaryExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_binary_expr(binary_expr)
    }

    fn visit_logical_expr(&mut self, logical_expr: &LogicalExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_logical_expr(logical_expr)
    }

    fn visit_unary_expr(&mut self, unary_expr: &UnaryExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_unary_expr(unary_expr)
    }

    fn visit_literal_expr(&mut self, literal_expr: &LiteralExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_literal_expr(literal_expr)
    }

    fn visit_grouping_expr(&mut self, grouping_expr: &GroupingExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_grouping_expr(grouping_expr)
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_identifier(identifier)
    }

    fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_assign_expr(assign_expr)
    }

    fn visit_ternary_expr(&mut self, ternary_expr: &TernaryExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_ternary_expr(ternary_expr)
    }

    fn visit_call_expr(&mut self, call_expr: &CallExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_call_expr(call_expr)
    }

    fn visit_get_expr(&mut self, get_expr: &GetExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_get_expr(get_expr)
    }

    fn visit_set_expr(&mut self, set_expr: &SetExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_set_expr(set_expr)
    }

    fn visit_this(&mut self, this: &This) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_this(this)
    }

    fn visit_super(&mut self, super_expr: &Super) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_super(super_expr)
    }

    fn visit_array_expr(&mut self, array_expr: &ArrayExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_array_expr(array_expr)
    }

    fn visit_subscript_get_expr(&mut self, subscript_get_expr: &SubscriptGetExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_subscript_get_expr(subscript_get_expr)
    }

    fn visit_subscript_set_expr(&mut self, subscript_set_expr: &SubscriptSetExpr) -> Result<Option<Self::R>, Self::E> {
        self.default_visit_subscript_set_expr(subscript_set_expr)
    }

    fn default_visit_stmt(&mut self, stmt: &Stmt) -> Result<Option<Self::R>, Self::E> {
        match stmt {
            Stmt::Expr(expr_stmt) => self.visit_expr_stmt(expr_stmt),
            Stmt::Print(print_stmt) => self.visit_print_stmt(print_stmt),
            Stmt::If(if_stmt) => self.visit_if_stmt(if_stmt),
            Stmt::While(while_stmt) => self.visit_while_stmt(while_stmt),
            Stmt::Break => self.visit_break_stmt(),
            Stmt::Continue => self.visit_continue_stmt(),
            Stmt::Return(return_stmt) => self.visit_return_stmt(return_stmt),
            Stmt::Block(block) => self.visit_block(block),
            Stmt::VarDecl(var_decl) => self.visit_var_decl(var_decl),
            Stmt::FunDecl(fun_decl) => self.visit_fun_decl(fun_decl),
            Stmt::ClassDecl(class_decl) => self.visit_class_decl(class_decl),
        }
    }

    fn default_visit_expr_stmt(&mut self, expr_stmt: &ExprStmt) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&expr_stmt.expr)
    }

    fn default_visit_print_stmt(&mut self, print_stmt: &PrintStmt) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&print_stmt.expr)
    }

    fn default_visit_if_stmt(&mut self, if_stmt: &IfStmt) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&if_stmt.condition)?;
        self.visit_stmt(&*if_stmt.then_stmt)?;
        if let Some(else_stmt) = &if_stmt.else_stmt {
            self.visit_stmt(&**else_stmt)?;
        }
        Ok(None)
    }

    fn default_visit_while_stmt(&mut self, while_stmt: &WhileStmt) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&while_stmt.condition)?;
        self.visit_stmt(&*while_stmt.stmt)?;
        Ok(None)
    }

    fn default_visit_break_stmt(&mut self) -> Result<Option<Self::R>, Self::E> {
        Ok(None)
    }

    fn default_visit_continue_stmt(&mut self) -> Result<Option<Self::R>, Self::E> {
        Ok(None)
    }

    fn default_visit_return_stmt(&mut self, return_stmt: &ReturnStmt) -> Result<Option<Self::R>, Self::E> {
        if let Some(value) = &return_stmt.value {
            return self.visit_expr(value);
        }
        Ok(None)
    }

    fn default_visit_block(&mut self, block: &Block) -> Result<Option<Self::R>, Self::E> {
        for stmt in &block.stmts {
            self.visit_stmt(stmt)?;
        }
        Ok(None)
    }

    fn default_visit_var_decl(&mut self, var_decl: &VarDecl) -> Result<Option<Self::R>, Self::E> {
        if let Some(initializer) = &var_decl.initializer {
            return self.visit_expr(initializer);
        }
        Ok(None)
    }

    fn default_visit_fun_decl(&mut self, fun_decl: &FunDecl) -> Result<Option<Self::R>, Self::E> {
        for stmt in &fun_decl.body {
            self.visit_stmt(stmt)?;
        }
        Ok(None)
    }

    fn default_visit_class_decl(&mut self, class_decl: &ClassDecl) -> Result<Option<Self::R>, Self::E> {
        if let Some(superclass) = &class_decl.superclass {
            self.visit_identifier(superclass)?;
        }
        for method in &class_decl.methods {
            self.visit_fun_decl(method)?;
        }
        Ok(None)
    }

    fn default_visit_expr(&mut self, expr: &Expr) -> Result<Option<Self::R>, Self::E> {
        match expr {
            Expr::Binary(binary_expr) => self.visit_binary_expr(binary_expr),
            Expr::Logical(logical_expr) => self.visit_logical_expr(logical_expr),
            Expr::Unary(unary_expr) => self.visit_unary_expr(unary_expr),
            Expr::Literal(literal_expr) => self.visit_literal_expr(literal_expr),
            Expr::Grouping(grouping_expr) => self.visit_grouping_expr(grouping_expr),
            Expr::Identifier(identifier) => self.visit_identifier(identifier),
            Expr::Assign(assign_expr) => self.visit_assign_expr(assign_expr),
            Expr::Call(call_expr) => self.visit_call_expr(call_expr),
            Expr::Get(get_expr) => self.visit_get_expr(get_expr),
            Expr::Set(set_expr) => self.visit_set_expr(set_expr),
            Expr::Ternary(ternary_expr) => self.visit_ternary_expr(ternary_expr),
            Expr::This(this) => self.visit_this(this),
            Expr::Super(super_expr) => self.visit_super(super_expr),
            Expr::Array(array_expr) => self.visit_array_expr(array_expr),
            Expr::SubscriptGet(subscript_get_expr) => self.visit_subscript_get_expr(subscript_get_expr),
            Expr::SubscriptSet(subscript_set_expr) => self.visit_subscript_set_expr(subscript_set_expr)
        }
    }

    fn default_visit_binary_expr(&mut self, binary_expr: &BinaryExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*binary_expr.lhs)?;
        self.visit_expr(&*binary_expr.rhs)?;
        Ok(None)
    }

    fn default_visit_logical_expr(&mut self, logical_expr: &LogicalExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*logical_expr.lhs)?;
        self.visit_expr(&*logical_expr.rhs)?;
        Ok(None)
    }

    fn default_visit_unary_expr(&mut self, unary_expr: &UnaryExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*unary_expr.expr)
    }

    fn default_visit_literal_expr(&mut self, _literal_expr: &LiteralExpr) -> Result<Option<Self::R>, Self::E> {
        Ok(None)
    }

    fn default_visit_grouping_expr(&mut self, grouping_expr: &GroupingExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*grouping_expr.expr)
    }

    fn default_visit_identifier(&mut self, _identifier: &Identifier) -> Result<Option<Self::R>, Self::E> {
        Ok(None)
    }

    fn default_visit_assign_expr(&mut self, assign_expr: &AssignExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*assign_expr.value)
    }

    fn default_visit_ternary_expr(&mut self, ternary_expr: &TernaryExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*ternary_expr.condition)?;
        self.visit_expr(&*ternary_expr.then_expr)?;
        self.visit_expr(&*ternary_expr.else_expr)?;
        Ok(None)
    }

    fn default_visit_call_expr(&mut self, call_expr: &CallExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*call_expr.name)?;
        for arg in &call_expr.args {
            self.visit_expr(arg)?;
        }
        Ok(None)
    }

    fn default_visit_get_expr(&mut self, get_expr: &GetExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*get_expr.object)
    }

    fn default_visit_set_expr(&mut self, set_expr: &SetExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*set_expr.object)?;
        self.visit_expr(&*set_expr.value)?;
        Ok(None)
    }

    fn default_visit_this(&mut self, _this: &This) -> Result<Option<Self::R>, Self::E> {
        Ok(None)
    }

    fn default_visit_super(&mut self, _super_expr: &Super) -> Result<Option<Self::R>, Self::E> {
        Ok(None)
    }

    fn default_visit_array_expr(&mut self, array_expr: &ArrayExpr) -> Result<Option<Self::R>, Self::E> {
        for element in &array_expr.elements {
            self.visit_expr(element)?;
        }
        Ok(None)
    }

    fn default_visit_subscript_get_expr(&mut self, subscript_get_expr: &SubscriptGetExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*subscript_get_expr.array)?;
        self.visit_expr(&*subscript_get_expr.index)?;
        Ok(None)
    }

    fn default_visit_subscript_set_expr(&mut self, subscript_set_expr: &SubscriptSetExpr) -> Result<Option<Self::R>, Self::E> {
        self.visit_expr(&*subscript_set_expr.array)?;
        self.visit_expr(&*subscript_set_expr.index)?;
        self.visit_expr(&*subscript_set_expr.value)?;
        Ok(None)
    }
}
