use crate::scanner::{Literal, Token};

pub enum Expr {
    Binary(BinaryExpr),
    Logical(LogicalExpr),
    Unary(UnaryExpr),
    Literal(LiteralExpr),
    Grouping(GroupingExpr),
    Identifier(Identifier),
    Assign(AssignExpr),
    Call(CallExpr),
    Unknown
}

pub struct BinaryExpr {
    pub op: Token,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>
}

pub struct UnaryExpr {
    pub op: Token,
    pub expr: Box<Expr>
}

pub struct LiteralExpr {
    pub content: Literal,
}

pub struct GroupingExpr {
    pub expr: Box<Expr>
}

pub struct Identifier {
    pub name: Token
}

pub struct AssignExpr {
    pub name: Token,
    pub value: Box<Expr>
}

pub struct LogicalExpr {
    pub operator: Token,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>
}

pub struct CallExpr {
    pub name: Box<Expr>,
    pub args: Vec<Expr>,
    pub paren: Token
}