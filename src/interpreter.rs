use std::cell::{Cell, UnsafeCell};

use crate::{
    ast::{Expr, Stmt},
    environment::Environment,
    token::{Literal, TokenType},
    LoxError,
};

#[derive(Debug, Clone)]
pub(crate) struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    fn evaluate(&mut self, expr: Expr) -> Result<Literal, LoxError> {
        match expr {
            Expr::Literal { value } => Ok(value),
            // TODO: I don't know whether this is right but we'll see.
            Expr::Variable { name } => self.environment.get(&name),
            Expr::Assign { name, value } => {
                let value = self.evaluate(*value)?;
                self.environment.assign(name, value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(*left)?;

                // TODO: Try some different arrangements to see whether it makes a
                // performance impact. I feel there is a really cool optimalisation
                // hiding here.

                // NOTE: We evaluate the left operand first, and return early if it is truthy
                // in case of 'or' operator, or falsey in case of 'and' operator.
                //
                // This means that this implementation short-circuits on logical operators :)
                match operator.token_type() {
                    TokenType::Or => {
                        if left.is_truthy() {
                            return Ok(left);
                        }
                    }
                    TokenType::And => {
                        if !left.is_truthy() {
                            return Ok(left);
                        }
                    }

                    _ => unreachable!(),
                }

                self.evaluate(*right)
            }
            Expr::Unary { operator, right } => {
                let right = self.evaluate(*right)?;
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
                let left = self.evaluate(*left)?;
                let right = self.evaluate(*right)?;
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
            Expr::Grouping { expression } => self.evaluate(*expression),
        }
    }

    fn execute(&mut self, statement: Stmt) -> Result<Literal, LoxError> {
        match statement {
            Stmt::Block { statements } => {
                self.execute_block(statements)?;
                Ok(Literal::Nil)
            }
            Stmt::Expression { expression } => self.evaluate(expression),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // NOTE: I stray from the book here, because I just really, really like expression
                // based languages. If, in this implementation, returns the result literal from
                // the executed branch.
                if self.evaluate(condition)?.is_truthy() {
                    self.execute(*then_branch)
                } else if let Some(else_branch) = else_branch {
                    self.execute(*else_branch)
                } else {
                    Ok(Literal::Nil)
                }
            }
            Stmt::Print { expression } => {
                println!("{}", self.evaluate(expression)?);
                Ok(Literal::Nil)
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(init) = initializer {
                    self.evaluate(init)?
                } else {
                    Literal::Nil
                };
                self.environment.define(name.lexeme().to_string(), value);
                Ok(Literal::Nil)
            }
            Stmt::While { condition, body } => {
                // TODO: These clones might actually give us undesirable and incorrect behaviour.
                while self.evaluate(condition.clone())?.is_truthy() {
                    self.execute(*body.clone())?;
                }
                Ok(Literal::Nil)
            }
        }
    }

    fn execute_block(&mut self, statements: Vec<Stmt>) -> Result<(), LoxError> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    pub(crate) fn interpret(&mut self, statements: Vec<Stmt>) -> Result<String, LoxError> {
        for statement in statements {
            self.execute(statement)?;
        }

        // TODO this is wrong of course. (temp)
        Ok(String::new())
    }
}
