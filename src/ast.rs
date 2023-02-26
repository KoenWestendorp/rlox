use std::fmt::{Arguments, Display};

use crate::token::{Literal, Token, TokenType};

type WrappedExpr = Box<Expr>;

#[derive(Debug, Clone)]
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
    Logical {
        left: WrappedExpr,
        operator: Token,
        right: WrappedExpr,
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
    Call {
        callee: WrappedExpr,
        paren: Token,
        arguments: Vec<Expr>,
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
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let op = match operator.token_type() {
                    TokenType::Or => "or",
                    TokenType::And => "and",
                    _ => unreachable!(),
                };
                write!(f, "{left} {op} {right}")
            }
            Expr::Unary { operator, right } => write!(f, "({} {right})", operator.lexeme()),
            Expr::Binary {
                left,
                operator,
                right,
            } => write!(f, "({left} {} {right})", operator.lexeme()),
            Expr::Call {
                callee, arguments, ..
            } => {
                let mut arguments: String = arguments.iter().map(|a| format!("{a}, ")).collect();
                // TODO: This string manipulation is inellegant. intersperse would work nicely but
                // it is unstable.
                arguments.truncate(arguments.len() - 2);
                write!(f, "{callee}({arguments})")
            }
            Expr::Grouping { expression } => write!(f, "{expression}"),
        }
    }
}

type WrappedStmt = Box<Stmt>;

#[derive(Debug, Clone)]
pub(crate) enum Stmt {
    Block {
        statements: Vec<Stmt>,
    },
    Expression {
        expression: Expr,
    },
    Function {
        name: Token,
        params: Vec<Token>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: WrappedStmt,
        else_branch: Option<WrappedStmt>,
    },
    Print {
        expression: Expr,
    },
    Return {
        keyword: Token,
        value: Option<Expr>,
    },
    Var {
        name: Token,
        initializer: Option<Expr>,
    },
    While {
        condition: Expr,
        body: WrappedStmt,
    },
}

impl Display for Stmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Stmt::Block { statements } => write!(
                f,
                "{{ {} }}",
                statements
                    .iter()
                    .map(|stmt| stmt.to_string())
                    .collect::<Vec<_>>()
                    .join("  ")
            ),
            Stmt::Function { name, params, body } => write!(f, "<fn {name}>", name = name.lexeme()),
            Stmt::Expression { expression } => write!(f, "{expression}"),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                write!(f, "if ({condition}) {then_branch}")?;
                if let Some(else_branch) = else_branch {
                    write!(f, " else {else_branch}")?;
                };
                Ok(())
            }
            Stmt::Print { expression } => write!(f, "print {expression}"),
            Stmt::Return { value, .. } => {
                if let Some(value) = value {
                    write!(f, "return {value}")
                } else {
                    write!(f, "return")
                }
            }
            Stmt::Var {
                name,
                initializer: Some(init),
            } => write!(f, "var {name} = {init}"),
            Stmt::Var {
                name,
                initializer: None,
            } => write!(f, "var {name}"),
            Stmt::While { condition, body } => write!(f, "while ({condition}) {body}"),
        }
    }
}
