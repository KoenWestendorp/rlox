use std::collections::HashMap;

use crate::{
    token::{Literal, Token},
    LoxError,
};

type Object = Literal;

pub(crate) struct Environment {
    values: HashMap<String, Object>,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub(crate) fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub(crate) fn get(&self, name: Token) -> Result<&Object, LoxError> {
        let lexeme = name.lexeme().to_owned();
        self.values.get(&lexeme).ok_or(LoxError::from_token(
            name,
            format!("Undefined variable '{lexeme}'."),
        ))
    }
}
