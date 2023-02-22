use std::fmt::Display;

use crate::token::{Literal, Token};

trait Nary {
    fn interpret(&self);
    fn resolve(&self);
    fn analyze(&self);
}

type WrappedExpr = Box<Expr>;

pub(crate) enum Expr {
    Literal {
        value: Literal,
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
