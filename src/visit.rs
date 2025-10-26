use crate::ast::{expr::*, stmt::*};

pub trait Visitor {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        visit_stmt(self, stmt);
    }

    fn visit_expr_stmt(&mut self, expr_stmt: &ExprStmt) {
        visit_expr_stmt(self, expr_stmt);
    }

    fn visit_print_stmt(&mut self, print_stmt: &PrintStmt) {
        visit_print_stmt(self, print_stmt);
    }

    fn visit_if_stmt(&mut self, if_stmt: &IfStmt) {
        visit_if_stmt(self, if_stmt);
    }

    fn visit_while_stmt(&mut self, while_stmt: &WhileStmt) {
        visit_while_stmt(self, while_stmt);
    }

    fn visit_return_stmt(&mut self, return_stmt: &ReturnStmt) {
        visit_return_stmt(self, return_stmt);
    }

    fn visit_block(&mut self, block: &Block) {
        visit_block(self, block);
    }

    fn visit_var_decl(&mut self, var_decl: &VarDecl) {
        visit_var_decl(self, var_decl);
    }

    fn visit_fun_decl(&mut self, fun_decl: &FunDecl) {
        visit_fun_decl(self, fun_decl);
    }

    fn visit_class_decl(&mut self, class_decl: &ClassDecl) {
        visit_class_decl(self, class_decl);
    }

    fn visit_expr(&mut self, expr: &Expr) {
        visit_expr(self, expr);
    }

    fn visit_binary_expr(&mut self, binary_expr: &BinaryExpr) {
        visit_binary_expr(self, binary_expr);
    }

    fn visit_logical_expr(&mut self, logical_expr: &LogicalExpr) {
        visit_logical_expr(self, logical_expr);
    }

    fn visit_unary_expr(&mut self, unary_expr: &UnaryExpr) {
        visit_unary_expr(self, unary_expr);
    }

    fn visit_literal_expr(&mut self, literal_expr: &LiteralExpr) {
        visit_literal_expr(self, literal_expr);
    }

    fn visit_grouping_expr(&mut self, grouping_expr: &GroupingExpr) {
        visit_grouping_expr(self, grouping_expr);
    }

    fn visit_identifier(&mut self, identifier: &Identifier) {
        visit_identifier(self, identifier);
    }

    fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) {
        visit_assign_expr(self, assign_expr);
    }

    fn visit_call_expr(&mut self, call_expr: &CallExpr) {
        visit_call_expr(self, call_expr);
    }

    fn visit_get_expr(&mut self, get_expr: &GetExpr) {
        visit_get_expr(self, get_expr);
    }

    fn visit_set_expr(&mut self, set_expr: &SetExpr) {
        visit_set_expr(self, set_expr);
    }

    fn visit_this(&mut self, this: &This) {
        visit_this(self, this);
    }

    fn visit_super(&mut self, super_expr: &Super) {
        visit_super(self, super_expr);
    }
}

pub fn visit_stmt<V: Visitor + ?Sized>(v: &mut V, stmt: &Stmt) {
    match stmt {
        Stmt::Expr(expr_stmt) => v.visit_expr_stmt(expr_stmt),
        Stmt::Print(print_stmt) => v.visit_print_stmt(print_stmt),
        Stmt::If(if_stmt) => v.visit_if_stmt(if_stmt),
        Stmt::While(while_stmt) => v.visit_while_stmt(while_stmt),
        Stmt::Return(return_stmt) => v.visit_return_stmt(return_stmt),
        Stmt::Block(block) => v.visit_block(block),
        Stmt::VarDecl(var_decl) => v.visit_var_decl(var_decl),
        Stmt::FunDecl(fun_decl) => v.visit_fun_decl(fun_decl),
        Stmt::ClassDecl(class_decl) => v.visit_class_decl(class_decl),
    }
}

pub fn visit_expr_stmt<V: Visitor + ?Sized>(v: &mut V, expr_stmt: &ExprStmt) {
    v.visit_expr(&expr_stmt.expr);
}

pub fn visit_print_stmt<V: Visitor + ?Sized>(v: &mut V, print_stmt: &PrintStmt) {
    v.visit_expr(&print_stmt.expr);
}

pub fn visit_if_stmt<V: Visitor + ?Sized>(v: &mut V, if_stmt: &IfStmt) {
    v.visit_expr(&if_stmt.condition);
    v.visit_stmt(&*if_stmt.then_stmt);
    if let Some(else_stmt) = &if_stmt.else_stmt {
        v.visit_stmt(&**else_stmt);
    }
}

pub fn visit_while_stmt<V: Visitor + ?Sized>(v: &mut V, while_stmt: &WhileStmt) {
    v.visit_expr(&while_stmt.condition);
    v.visit_stmt(&*while_stmt.stmt);
}

pub fn visit_return_stmt<V: Visitor + ?Sized>(v: &mut V, return_stmt: &ReturnStmt) {
    if let Some(value) = &return_stmt.value {
        v.visit_expr(value);
    }
}

pub fn visit_block<V: Visitor + ?Sized>(v: &mut V, block: &Block) {
    for stmt in &block.stmts {
        v.visit_stmt(stmt);
    }
}

pub fn visit_var_decl<V: Visitor + ?Sized>(v: &mut V, var_decl: &VarDecl) {
    if let Some(initializer) = &var_decl.initializer {
        v.visit_expr(initializer);
    }
}

pub fn visit_fun_decl<V: Visitor + ?Sized>(v: &mut V, fun_decl: &FunDecl) {
    v.visit_stmt(&*fun_decl.body);
}

pub fn visit_class_decl<V: Visitor + ?Sized>(v: &mut V, class_decl: &ClassDecl) {
    if let Some(superclass) = &class_decl.superclass {
        v.visit_identifier(superclass);
    }
    for method in &class_decl.methods {
        v.visit_fun_decl(method);
    }
}

pub fn visit_expr<V: Visitor + ?Sized>(v: &mut V, expr: &Expr) {
    match expr {
        Expr::Binary(binary_expr) => v.visit_binary_expr(binary_expr),
        Expr::Logical(logical_expr) => v.visit_logical_expr(logical_expr),
        Expr::Unary(unary_expr) => v.visit_unary_expr(unary_expr),
        Expr::Literal(literal_expr) => v.visit_literal_expr(literal_expr),
        Expr::Grouping(grouping_expr) => v.visit_grouping_expr(grouping_expr),
        Expr::Identifier(identifier) => v.visit_identifier(identifier),
        Expr::Assign(assign_expr) => v.visit_assign_expr(assign_expr),
        Expr::Call(call_expr) => v.visit_call_expr(call_expr),
        Expr::Get(get_expr) => v.visit_get_expr(get_expr),
        Expr::Set(set_expr) => v.visit_set_expr(set_expr),
        Expr::This(this) => v.visit_this(this),
        Expr::Super(super_expr) => v.visit_super(super_expr)
    }
}

pub fn visit_binary_expr<V: Visitor + ?Sized>(v: &mut V, binary_expr: &BinaryExpr) {
    v.visit_expr(&*binary_expr.lhs);
    v.visit_expr(&*binary_expr.rhs);
}

pub fn visit_logical_expr<V: Visitor + ?Sized>(v: &mut V, logical_expr: &LogicalExpr) {
    v.visit_expr(&*logical_expr.lhs);
    v.visit_expr(&*logical_expr.rhs);
}

pub fn visit_unary_expr<V: Visitor + ?Sized>(v: &mut V, unary_expr: &UnaryExpr) {
    v.visit_expr(&*unary_expr.expr);
}

pub fn visit_literal_expr<V: Visitor + ?Sized>(v: &mut V, literal_expr: &LiteralExpr) {

}

pub fn visit_grouping_expr<V: Visitor + ?Sized>(v: &mut V, grouping_expr: &GroupingExpr) {
    v.visit_expr(&*grouping_expr.expr);
}

pub fn visit_identifier<V: Visitor + ?Sized>(v: &mut V, identifier: &Identifier) {

}

pub fn visit_assign_expr<V: Visitor + ?Sized>(v: &mut V, assign_expr: &AssignExpr) {
    v.visit_expr(&*assign_expr.value);
}

pub fn visit_call_expr<V: Visitor + ?Sized>(v: &mut V, call_expr: &CallExpr) {
    v.visit_expr(&*call_expr.name);
    for arg in &call_expr.args {
        v.visit_expr(arg);
    }
}

pub fn visit_get_expr<V: Visitor + ?Sized>(v: &mut V, get_expr: &GetExpr) {
    v.visit_expr(&*get_expr.object);
}

pub fn visit_set_expr<V: Visitor + ?Sized>(v: &mut V, set_expr: &SetExpr) {
    v.visit_expr(&*set_expr.object);
    v.visit_expr(&*set_expr.value);
}

pub fn visit_this<V: Visitor + ?Sized>(v: &mut V, this: &This) {

}

pub fn visit_super<V: Visitor + ?Sized>(v: &mut V, super_expr: &Super) {

}
