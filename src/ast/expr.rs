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
    Call(CallExpr),
    Get(GetExpr),
    Set(SetExpr),
    Ternary(TernaryExpr),
    This(This),
    Super(Super),
    Array(ArrayExpr),
    SubscriptGet(SubscriptGetExpr),
    SubscriptSet(SubscriptSetExpr)
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

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct GetExpr {
    pub object: Box<Expr>,
    pub name: Token
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SetExpr {
    pub object: Box<Expr>,
    pub name: Token,
    pub value: Box<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct TernaryExpr {
    pub condition: Box<Expr>,
    pub then_expr: Box<Expr>,
    pub else_expr: Box<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct This {
    pub keyword: Token
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Super {
    pub keyword: Token,
    pub method: Token
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct ArrayExpr {
    pub elements: Vec<Expr>
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SubscriptGetExpr {
    pub array: Box<Expr>,
    pub index: Box<Expr>,
    pub bracket: Token
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct SubscriptSetExpr {
    pub array: Box<Expr>,
    pub index: Box<Expr>,
    pub value: Box<Expr>,
    pub bracket: Token
}