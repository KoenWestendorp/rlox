use std::collections::HashMap;

use crate::token::{Literal, Token};
use crate::LoxError;

type Object = Literal;

#[derive(Debug, Clone)]
pub(crate) struct Environment {
    fallback: Option<Box<Self>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            fallback: None,
            values: HashMap::new(),
        }
    }

    pub(crate) fn from_parent(environment: &Environment) -> Self {
        Self {
            fallback: Some(Box::new(environment.clone())),
            values: HashMap::new(),
        }
    }

    pub(crate) fn fallback(self) -> Option<Environment> {
        self.fallback.map(|env| *env)
    }
}

impl Environment {
    pub(crate) fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    /// Get the Literal value bound to a variable.
    ///
    /// # Errors
    ///
    /// This function will return an error if the variable is not found.
    pub(crate) fn get_var(&self, name: &Token) -> Result<&Object, LoxError> {
        let lexeme = name.lexeme().to_owned();
        match self.fallback {
            // If there is no enclosing `fallback` environment, get the variable name from this
            // environment.
            None => self.values.get(&lexeme),
            // Otherwise, try to get it from this environment, but when it is not present, get it
            // from the enclosing environment.
            Some(ref fallback) => match self.values.get(&lexeme) {
                None => return fallback.get_var(name),
                value => value,
            },
        }
        .ok_or_else(|| LoxError::from_token(name, format!("Undefined variable '{lexeme}'.")))
    }

    /// Assign another Literal value to a variable.
    ///
    /// # Errors
    ///
    /// This function will return an error if the variable is not found.
    pub(crate) fn assign(&mut self, name: Token, value: Literal) -> Result<Literal, LoxError> {
        let lexeme = name.lexeme().to_owned();
        if self.values.contains_key(&lexeme) {
            // The variable exists in the current scope. Nice. We assign the value to this
            // variable and return the value.
            self.values.insert(lexeme, value.clone());
            return Ok(value);
        }

        // The variable does not exist in the current scope. Let's try whether it is in the
        // previous scope.
        if let Some(ref mut fallback) = self.fallback {
            return fallback.assign(name, value);
        }

        Err(LoxError::from_token(
            &name,
            format!("Undefined variable '{lexeme}'."),
        ))
    }
}
