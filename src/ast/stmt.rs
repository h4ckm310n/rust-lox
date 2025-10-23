use crate::{ast::expr::Expr, token::Token};

pub enum Stmt {
    Expr(ExprStmt),
    Print(PrintStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(ReturnStmt),
    Block(Block),
    VarDecl(VarDecl),
    FunDecl(FunDecl)
}

pub struct ExprStmt {
    pub expr: Expr
}

pub struct PrintStmt {
    pub expr: Expr
}

pub struct IfStmt {
    pub condition: Expr,
    pub then_stmt: Box<Stmt>,
    pub else_stmt: Option<Box<Stmt>>
}

pub struct WhileStmt {
    pub condition: Expr,
    pub stmt: Box<Stmt>
}

pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Option<Expr>
}

pub struct Block {
    pub stmts: Vec<Stmt>
}

pub struct VarDecl {
    pub name: Token,
    pub initializer: Option<Expr>
}

pub struct FunDecl {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Box<Stmt>
}