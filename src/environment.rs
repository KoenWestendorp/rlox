use std::{collections::HashMap, rc::Rc};

use crate::{
    token::{Literal, Token},
    LoxError,
};

type Object = Literal;

#[derive(Debug, Clone)]
pub(crate) struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
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
    pub(crate) fn get(&self, name: Token) -> Result<&Object, LoxError> {
        // let lexeme = name.lexeme().to_owned();
        //
        // match &self.enclosing {
        //     // If there is no enclosing environment, get the variable name from this environment.
        //     None => self.values.get(&lexeme),
        //     // Otherwise get it from the enclosing environment.
        //     Some(enclosing) => return enclosing.get(name),
        // }
        // .ok_or(LoxError::from_token(
        //     name,
        //     format!("Undefined variable '{lexeme}'."),
        // ))
        let lexeme = name.lexeme().to_owned();
        self.values.get(&lexeme).ok_or(LoxError::from_token(
            name,
            format!("Undefined variable '{lexeme}'."),
        ))
    }

    /// Assign another Literal value to a variable.
    ///
    /// # Errors
    ///
    /// This function will return an error if the variable is not found.
    pub(crate) fn assign(&mut self, name: Token, value: Literal) -> Result<Literal, LoxError> {
        let lexeme = name.lexeme().to_owned();
        if self.values.contains_key(&lexeme) {
            self.values.insert(lexeme, value.clone());
            return Ok(value);
        }

        Err(LoxError::from_token(
            name,
            format!("Undefined variable '{lexeme}'."),
        ))
    }

    // pub(crate) fn enclosing(&self) -> Option<Rc<Environment>> {
    //     self.enclosing.map(|ref env| Rc::clone(env))
    // }
}
