use crate::{token::Token, visitor::Visitor};

pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
}

pub trait Node {
    fn accept<T: Visitor>(&self, visitor: &T) -> T::Output;
}

impl Node for Expr {
    fn accept<T: Visitor>(&self, visitor: &T) -> T::Output {
        match self {
            Expr::Binary(expr) => expr.accept(visitor),
            Expr::Grouping(expr) => expr.accept(visitor),
            Expr::Literal(expr) => expr.accept(visitor),
            Expr::Unary(expr) => expr.accept(visitor),
        }
    }
}

pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Box<Token>,
    pub right: Box<Expr>,
}

impl Node for Binary {
    fn accept<T: Visitor>(&self, visitor: &T) -> T::Output {
        visitor.visit_binary(self)
    }
}

pub struct Grouping {
    pub expression: Box<Expr>,
}

impl Node for Grouping {
    fn accept<T: Visitor>(&self, visitor: &T) -> T::Output {
        visitor.visit_grouping(self)
    }
}

#[derive(Debug, Clone)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

pub struct Literal {
    pub value: LiteralValue,
}

impl Node for Literal {
    fn accept<T: Visitor>(&self, visitor: &T) -> T::Output {
        visitor.visit_literal(self)
    }
}

pub struct Unary {
    pub operator: Box<Token>,
    pub right: Box<Expr>,
}

impl Node for Unary {
    fn accept<T: Visitor>(&self, visitor: &T) -> T::Output {
        visitor.visit_unary(self)
    }
}
