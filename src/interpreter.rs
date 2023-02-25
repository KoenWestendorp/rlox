use crate::ast::{Expr, Stmt};
use crate::callable::{Callable, Function};
use crate::environment::Environment;
use crate::token::{Literal, TokenType};
use crate::LoxError;

#[derive(Debug, Clone)]
pub(crate) struct Interpreter {
    globals: Box<Environment>,
    // environment: Environment,
}

impl Interpreter {
    pub(crate) fn new() -> Self {
        Self {
            globals: Box::new(Environment::new()), // environment: Environment::new(),
        }
    }

    fn evaluate(&mut self, expr: Expr, environment: &mut Environment) -> Result<Literal, LoxError> {
        match expr {
            Expr::Literal { value } => Ok(value),
            // TODO: I don't know whether this is right but we'll see.
            Expr::Variable { ref name } => environment.get_var(name).cloned(),
            Expr::Assign { name, value } => {
                let value = self.evaluate(*value, environment)?;
                environment.assign(name, value)
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                let left = self.evaluate(*left, environment)?;

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

                self.evaluate(*right, environment)
            }
            Expr::Unary { operator, right } => {
                let right = self.evaluate(*right, environment)?;
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
                let left = self.evaluate(*left, environment)?;
                let right = self.evaluate(*right, environment)?;
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
                        match (left, right) {
                            (Number(l), Number(r)) => Some(Bool(l > r)),
                            (Bool(l), Bool(r)) => Some(Bool(l > r)),
                            (l, r) => Some(Bool(l.is_truthy() > r.is_truthy())),
                        }
                        .ok_or(LoxError::unexpected_type(&operator))
                    }
                    TokenType::GreaterEqual => {
                        use Literal::*;
                        match (left, right) {
                            (Number(l), Number(r)) => Some(Bool(l >= r)),
                            (Bool(l), Bool(r)) => Some(Bool(l >= r)),
                            (l, r) => Some(Bool(l.is_truthy() >= r.is_truthy())),
                        }
                        .ok_or(LoxError::unexpected_type(&operator))
                    }
                    TokenType::Less => {
                        use Literal::*;
                        match (left, right) {
                            (Number(l), Number(r)) => Some(Bool(l < r)),
                            (Bool(l), Bool(r)) => Some(Bool(l < r)),
                            (l, r) => Some(Bool(l.is_truthy() < r.is_truthy())),
                        }
                        .ok_or(LoxError::unexpected_type(&operator))
                    }
                    TokenType::LessEqual => {
                        use Literal::*;
                        match (left, right) {
                            (Number(l), Number(r)) => Some(Bool(l <= r)),
                            (Bool(l), Bool(r)) => Some(Bool(l <= r)),
                            (l, r) => Some(Bool(l.is_truthy() <= r.is_truthy())),
                        }
                        .ok_or(LoxError::unexpected_type(&operator))
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
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.evaluate(*callee, environment)?;
                let mut argument_literals = Vec::new();
                for argument in arguments {
                    argument_literals.push(self.evaluate(argument, environment)?);
                }
                let arguments = argument_literals;

                let function = callee.callable().ok_or(LoxError::from_token(
                    &paren,
                    "Can only call functions and classes.".to_string(),
                ))?;

                if arguments.len() != function.arity() {
                    return Err(LoxError::from_token(
                        &paren,
                        format!(
                            "Expected {arity} + arguments but got {len}.",
                            arity = function.arity(),
                            len = arguments.len()
                        ),
                    ));
                }

                function.call(self, environment, arguments)
            }
            Expr::Grouping { expression } => self.evaluate(*expression, environment),
        }
    }

    fn execute(
        &mut self,
        statement: Stmt,
        environment: &mut Environment,
    ) -> Result<Literal, LoxError> {
        match statement {
            Stmt::Block { statements } => {
                self.execute_block(statements, environment)?;
                Ok(Literal::Nil)
            }
            Stmt::Expression { expression } => self.evaluate(expression, environment),
            function @ Stmt::Function { .. } => {
                let function = Function::new(function).unwrap();
                environment.define(
                    function.name().lexeme().to_string(),
                    Literal::Fun(Box::new(function)),
                );

                Ok(Literal::Nil)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                // NOTE: I stray from the book here, because I just really, really like expression
                // based languages. If, in this implementation, returns the result literal from
                // the executed branch.
                if self.evaluate(condition, environment)?.is_truthy() {
                    self.execute(*then_branch, environment)
                } else if let Some(else_branch) = else_branch {
                    self.execute(*else_branch, environment)
                } else {
                    Ok(Literal::Nil)
                }
            }
            Stmt::Print { expression } => {
                println!("{}", self.evaluate(expression, environment)?);
                Ok(Literal::Nil)
            }
            Stmt::Var { name, initializer } => {
                let value = if let Some(init) = initializer {
                    self.evaluate(init, environment)?
                } else {
                    Literal::Nil
                };
                environment.define(name.lexeme().to_string(), value);
                Ok(Literal::Nil)
            }
            Stmt::While { condition, body } => {
                // TODO: These clones might actually give us undesirable and incorrect behaviour.
                while self.evaluate(condition.clone(), environment)?.is_truthy() {
                    self.execute(*body.clone(), environment)?;
                }
                Ok(Literal::Nil)
            }
        }
    }

    pub(crate) fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        environment: &mut Environment,
    ) -> Result<(), LoxError> {
        let mut block_env = Environment::from_parent(environment);
        for statement in statements {
            self.execute(statement, &mut block_env)?;
        }
        // std::mem::swap(&mut *block_env.fallback.unwrap(), environment);
        *environment = std::mem::take(&mut block_env.fallback()).unwrap();
        Ok(())
    }

    pub(crate) fn interpret(&mut self, statements: Vec<Stmt>) -> Result<String, LoxError> {
        let mut environment = Environment::new();
        self.interpret_with_env(statements, &mut environment)
    }

    pub(crate) fn interpret_with_env(
        &mut self,
        statements: Vec<Stmt>,
        environment: &mut Environment,
    ) -> Result<String, LoxError> {
        for statement in statements {
            self.execute(statement, environment)?;
        }

        // TODO this is wrong of course. (temp)
        Ok(String::new())
    }
}
