use std::fmt::Display;

use crate::token::{Literal, Token};

trait Nary {
    fn interpret(&self);
    fn resolve(&self);
    fn analyze(&self);
}

type WrappedExpr = Box<Expr>;

#[derive(Debug)]
pub(crate) enum Expr {
    Literal {
        value: Literal,
    },
    Variable {
        name: Token,
    },
    Assign {
        name: Token,
        value: WrappedExpr,
    },
    Unary {
        operator: Token,
        right: WrappedExpr,
    },
    Binary {
        left: WrappedExpr,
        operator: Token,
        right: WrappedExpr,
    },
    Grouping {
        expression: WrappedExpr,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal { value } => write!(f, "{value}"),
            Expr::Variable { name } => write!(f, "{name}"),
            Expr::Assign { name, value } => write!(f, "{name} = {value}"),
            Expr::Unary { operator, right } => write!(f, "({} {right})", operator.lexeme()),
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({left} {} {right})", operator.lexeme()),
            Expr::Grouping { expression } => write!(f, "{expression}"),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Stmt {
    Expression {
        expression: WrappedExpr,
    },
    Print {
        expression: WrappedExpr,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Expression { expression } => write!(f, "{expression}"),
            Stmt::Print { expression } => write!(f, "print {expression}"),
            Stmt::Var {
                name,
                initializer: Some(init),
            } => write!(f, "var {name} = {init}"),
            Stmt::Var {
                name,
                initializer: None,
            } => write!(f, "var {name}"),
        }
    }
}
