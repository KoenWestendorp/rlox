use std::{
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::{
    token::{Literal, Token},
    LoxError,
};

type Object = Literal;

#[derive(Debug, Clone)]
pub(crate) struct Environment {
    pub(crate) parent: Option<Weak<Self>>,
    values: HashMap<String, Object>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
        }
    }

    pub(crate) fn from_parent(parent: &Rc<Self>) -> Self {
        Self {
            parent: Some(Rc::downgrade(parent)),
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
    pub(crate) fn get(&self, name: &Token) -> Result<Object, LoxError> {
        // let lexeme = name.lexeme().to_owned();
        //
        // match self.parent {
        //     // If there is no enclosing environment, get the variable name from this environment.
        //     None => self.values.get(&lexeme),
        //     // Otherwise get it from the enclosing environment.
        //     Some(ref parent) => {
        //         if let Some(parent) = parent.upgrade() {
        //             return parent.get(name);
        //         } else {
        //             self.values.get(&lexeme)
        //         }
        //     }
        // }
        // .ok_or_else(|| {
        //     LoxError::from_token(name.clone(), format!("Undefined variable '{lexeme}'."))
        // })
        // .cloned()
        let lexeme = name.lexeme().to_owned();
        self.values
            .get(&lexeme)
            .ok_or(LoxError::from_token(
                name.clone(),
                format!("Undefined variable '{lexeme}'."),
            ))
            .cloned()
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
