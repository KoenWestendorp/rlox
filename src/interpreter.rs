use std::cmp::Ordering;

use crate::{
    ast::{Expr, Stmt},
    token::{Literal, TokenType},
    LoxError,
};

pub(crate) struct Interpreter;

impl Interpreter {
    fn evaluate(expr: Expr) -> Result<Literal, LoxError> {
        match expr {
            Expr::Literal { value } => Ok(value),
            Expr::Unary { operator, right } => {
                let right = Self::evaluate(*right)?;
                match operator.token_type() {
                    TokenType::Bang => Ok(right.operate_truthy(|n| !n)),
                    TokenType::Minus => right
                        .operate_number(|n| -n)
                        .ok_or(LoxError::unexpected_type(&operator)),
                    _ => unreachable!(),
                }
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                // NOTE: The order of the left and right evaluations is significant. This
                // determines the order in which binary expressions are evaluated. In our case:
                // left-to-right.
                let left = Self::evaluate(*left)?;
                let right = Self::evaluate(*right)?;
                match operator.token_type() {
                    TokenType::Minus => left
                        .operate_number_binary(right, |l, r| l - r)
                        .ok_or(LoxError::unexpected_type(&operator)),
                    TokenType::Plus => {
                        // FIXME: We can do this better by matching on the result of
                        // operate_number. Like, seriously, we can create a beautiful match here.
                        if left.number().is_some() && right.number().is_some() {
                            return left
                                .operate_number_binary(right, |l, r| l + r)
                                .ok_or(LoxError::unexpected_type(&operator));
                        }
                        if left.string().is_some() && right.string().is_some() {
                            let right =
                                right.string().ok_or(LoxError::unexpected_type(&operator))?;
                            return left
                                .operate_string(|left| format!("{left}{right}"))
                                .ok_or(LoxError::unexpected_type(&operator));
                        }
                        Err(LoxError::unexpected_type(&operator))
                    }
                    TokenType::Slash => left
                        .operate_number_binary(right, |l, r| l / r)
                        .ok_or(LoxError::unexpected_type(&operator)),
                    TokenType::Star => left
                        .operate_number_binary(right, |l, r| l * r)
                        .ok_or(LoxError::unexpected_type(&operator)),
                    // FIXME: Use a macro for these suckers?
                    TokenType::Greater => {
                        use Literal::*;
                        return match (left, right) {
                            (Number(l), Number(r)) => Some(Bool(l > r)),
                            (Bool(l), Bool(r)) => Some(Bool(l > r)),
                            (l, r) => Some(Bool(l.is_truthy() > r.is_truthy())),
                        }
                        .ok_or(LoxError::unexpected_type(&operator));
                    }
                    TokenType::GreaterEqual => {
                        use Literal::*;
                        return match (left, right) {
                            (Number(l), Number(r)) => Some(Bool(l >= r)),
                            (Bool(l), Bool(r)) => Some(Bool(l >= r)),
                            (l, r) => Some(Bool(l.is_truthy() >= r.is_truthy())),
                        }
                        .ok_or(LoxError::unexpected_type(&operator));
                    }
                    TokenType::Less => {
                        use Literal::*;
                        return match (left, right) {
                            (Number(l), Number(r)) => Some(Bool(l < r)),
                            (Bool(l), Bool(r)) => Some(Bool(l < r)),
                            (l, r) => Some(Bool(l.is_truthy() < r.is_truthy())),
                        }
                        .ok_or(LoxError::unexpected_type(&operator));
                    }
                    TokenType::LessEqual => {
                        use Literal::*;
                        return match (left, right) {
                            (Number(l), Number(r)) => Some(Bool(l <= r)),
                            (Bool(l), Bool(r)) => Some(Bool(l <= r)),
                            (l, r) => Some(Bool(l.is_truthy() <= r.is_truthy())),
                        }
                        .ok_or(LoxError::unexpected_type(&operator));
                    }
                    // This unwrap should be fine because we apply it to the result of is_equal,
                    // which is always Literal::Bool(...), so the type is always as expected.
                    TokenType::BangEqual => {
                        Ok(Literal::is_equal(left, right).operate_bool(|b| !b).unwrap())
                    }
                    TokenType::EqualEqual => Ok(Literal::is_equal(left, right)),
                    _ => todo!(),
                }
            }
            Expr::Grouping { expression } => Self::evaluate(*expression),
        }
    }

    pub(crate) fn interpret(statements: Vec<Stmt>) -> Result<String, LoxError> {
        for statement in statements {
            Self::execute(statement)?;
        }

        // TODO this is wrong of course. (temp)
        Ok(String::new())
    }

    fn execute(statement: Stmt) -> Result<Literal, LoxError> {
        match statement {
            Stmt::Expression { expression } => Self::evaluate(*expression),
            Stmt::Print { expression } => {
                println!("{}", Self::evaluate(*expression)?);
                Ok(Literal::Nil)
            }
        }
    }
}
