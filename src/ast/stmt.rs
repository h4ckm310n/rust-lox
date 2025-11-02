use crate::{ast::expr::{Expr, Identifier}, token::Token};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Stmt {
    Expr(ExprStmt),
    Print(PrintStmt),
    If(IfStmt),
    While(WhileStmt),
    Return(ReturnStmt),
    Block(Block),
    VarDecl(VarDecl),
    FunDecl(FunDecl),
    ClassDecl(ClassDecl)
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ExprStmt {
    pub expr: Expr
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct PrintStmt {
    pub expr: Expr
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_stmt: Box<Stmt>,
    pub else_stmt: Option<Box<Stmt>>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub stmt: Box<Stmt>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ReturnStmt {
    pub keyword: Token,
    pub value: Option<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct VarDecl {
    pub name: Token,
    pub initializer: Option<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct FunDecl {
    pub name: Token,
    pub params: Vec<Token>,
    pub body: Vec<Stmt>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ClassDecl {
    pub name: Token,
    pub superclass: Option<Identifier>,
    pub methods: Vec<FunDecl>
}