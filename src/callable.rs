use crate::ast::{Expr, Stmt};
use crate::environment::{self, Environment};
use crate::interpreter::Interpreter;
use crate::token::{Literal, Token};
use crate::LoxError;

pub(crate) trait Callable {
    fn new(declaration: Stmt) -> Option<Self>
    where
        Self: Sized;
    fn call(
        &self,
        interpreter: &mut Interpreter,
        environment: &Environment,
        arguments: Vec<Literal>,
    ) -> Result<Literal, LoxError>;
    fn arity(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct Function {
    name: Token,
    params: Vec<String>,
    body: Vec<Stmt>,
}

impl Function {
    pub(crate) fn name(&self) -> &Token {
        &self.name
    }
}

impl Callable for Function {
    fn new(declaration: Stmt) -> Option<Self> {
        match declaration {
            Stmt::Function { name, params, body } => {
                let params = params
                    .iter()
                    .map(|param| param.lexeme().to_string())
                    .collect();
                Some(Self { name, params, body })
            }
            _ => None,
        }
    }

    fn call(
        &self,
        interpreter: &mut Interpreter,
        environment: &Environment,
        arguments: Vec<Literal>,
    ) -> Result<Literal, LoxError> {
        let mut environment = Environment::from_parent(environment);

        for (n, param) in self.params.iter().enumerate() {
            // TODO: Is this unwrap guaranteed by invariants from parsing process?
            environment.define(param.to_string(), arguments.get(n).unwrap().clone());
        }

        interpreter.execute_block(self.body.clone(), &mut environment)?;

        Ok(Literal::Nil)
    }

    fn arity(&self) -> usize {
        self.params.len()
    }
}
