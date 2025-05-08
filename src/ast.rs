use crate::{
    token::{Identifier, Token},
    visitor::{StatementVisitor, Visitor},
};

pub enum Statement {
    Expression(ExpressionStatement),
    Print(PrintStatement),
    Variable(VariableStatement),
}

pub struct ExpressionStatement {
    pub expression: Box<Expr>,
}

pub struct PrintStatement {
    pub expression: Box<Expr>,
}

pub struct VariableStatement {
    pub name: Box<Identifier>,
    pub value: Box<Expr>,
}

pub trait Stmt {
    fn accept<T: StatementVisitor>(&self, visitor: &mut T) -> T::Output;
}

impl Stmt for Statement {
    fn accept<T: StatementVisitor>(&self, visitor: &mut T) -> T::Output {
        visitor.visit_statement(self)
    }
}

pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    Literal(Literal),
    Unary(Unary),
    Variable(Variable),
    Assignment(Assignment),
}

pub trait Node {
    fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output;
}

impl Node for Expr {
    fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        match self {
            Expr::Binary(it) => it.accept(visitor),
            Expr::Grouping(it) => it.accept(visitor),
            Expr::Literal(it) => it.accept(visitor),
            Expr::Unary(it) => it.accept(visitor),
            Expr::Variable(it) => it.accept(visitor),
            Expr::Assignment(it) => it.accept(visitor),
        }
    }
}

pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Box<Token>,
    pub right: Box<Expr>,
}

impl Node for Binary {
    fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        visitor.visit_binary(self)
    }
}

pub struct Grouping {
    pub expression: Box<Expr>,
}

impl Node for Grouping {
    fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
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
    fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        visitor.visit_literal(self)
    }
}
pub struct Unary {
    pub operator: Box<Token>,
    pub right: Box<Expr>,
}

impl Node for Unary {
    fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        visitor.visit_unary(self)
    }
}
pub struct Variable {
    pub token: Box<Identifier>,
}

impl Node for Variable {
    fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        visitor.visit_variable(self)
    }
}

pub struct Assignment {
    pub name: Box<Identifier>,
    pub value: Box<Expr>,
}

impl Node for Assignment {
    fn accept<T: Visitor>(&self, visitor: &mut T) -> T::Output {
        visitor.visit_assignment(self)
    }
}
