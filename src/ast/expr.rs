use crate::token::{Literal, Token};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum Expr {
    Binary(BinaryExpr),
    Logical(LogicalExpr),
    Unary(UnaryExpr),
    Literal(LiteralExpr),
    Grouping(GroupingExpr),
    Identifier(Identifier),
    Assign(AssignExpr),
    Call(CallExpr)
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct BinaryExpr {
    pub op: Token,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct UnaryExpr {
    pub op: Token,
    pub expr: Box<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct LiteralExpr {
    pub content: Literal,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct GroupingExpr {
    pub expr: Box<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Identifier {
    pub name: Token
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct AssignExpr {
    pub name: Token,
    pub value: Box<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct LogicalExpr {
    pub operator: Token,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CallExpr {
    pub name: Box<Expr>,
    pub args: Vec<Expr>,
    pub paren: Token
}