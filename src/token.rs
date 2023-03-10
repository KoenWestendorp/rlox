use std::{fmt::Display, ops::Deref};

use crate::{
    callable::{Callable, Function},
    environment::Environment,
};

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: usize,
    col: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: usize,
        col: usize,
    ) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
            col,
        }
    }

    pub fn token_type(&self) -> TokenType {
        self.token_type
    }

    pub fn lexeme(&self) -> &str {
        self.lexeme.as_ref()
    }

    pub fn literal(&self) -> Option<Literal> {
        self.literal.clone()
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub(crate) fn col(&self) -> usize {
        self.col
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            token_type, lexeme, ..
        } = self;
        match &self.literal {
            None => write!(f, "{token_type:?} {lexeme}"),
            Some(literal) => write!(f, "{token_type:?} {lexeme} {literal}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Identifier(String),
    Fun(Box<Function>),
    String(String),
    Number(f64),
    Nil,
    Bool(bool),
}

impl Literal {
    fn identifier(&self) -> Option<&String> {
        match self {
            Literal::Identifier(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn string(&self) -> Option<&String> {
        match self {
            Literal::String(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn number(&self) -> Option<f64> {
        match self {
            Literal::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub(crate) fn bool(&self) -> Option<bool> {
        match self {
            Literal::Bool(b) => Some(*b),
            _ => None,
        }
    }

    ///
    ///
    /// In Lox, `false` and `nil` are falsey.
    /// Everything else is truthy.
    pub(crate) fn is_truthy(&self) -> bool {
        match self {
            // TODO: Profile these two options.
            Literal::Nil => false,
            Literal::Bool(b) => *b,
            // vs
            // Literal::Bool(false) | Literal::Nil => false,
            // (curiosity bikeshed)
            _ => true,
        }
    }

    pub(crate) fn is_equal(left: Literal, right: Literal) -> Self {
        let equality = match (left, right) {
            (Literal::Identifier(a), Literal::Identifier(b)) => a == b,
            (Literal::Fun(a), Literal::Fun(b)) => a.name().lexeme() == b.name().lexeme(),
            (Literal::String(a), Literal::String(b)) => a == b,
            (Literal::Number(a), Literal::Number(b)) => a == b,
            (Literal::Nil, Literal::Nil) => true,
            (Literal::Bool(a), Literal::Bool(b)) => a == b,
            _ => false,
        };

        Self::Bool(equality)
    }

    pub(crate) fn operate_string(&self, f: impl Fn(String) -> String) -> Option<Self> {
        self.string().map(|s| Self::String(f(s.clone())))
    }

    pub(crate) fn operate_number(&self, f: impl Fn(f64) -> f64) -> Option<Self> {
        self.number().map(|n| Self::Number(f(n)))
    }

    pub(crate) fn operate_bool(&self, f: impl Fn(bool) -> bool) -> Option<Self> {
        self.bool().map(|b| Self::Bool(f(b)))
    }

    pub(crate) fn operate_truthy(&self, f: impl Fn(bool) -> bool) -> Self {
        Self::Bool(f(self.is_truthy()))
    }

    pub(crate) fn operate_number_binary(
        &self,
        right: Self,
        f: impl Fn(f64, f64) -> f64,
    ) -> Option<Self> {
        let left = self;
        let right = right.number()?;
        left.operate_number(|n| f(n, right))
    }

    pub(crate) fn callable(&self) -> Option<impl Callable> {
        match self {
            Self::Fun(fun) => Some(*fun.clone()),
            _ => None,
        }
    }
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Literal::Identifier(i) => write!(f, "<{i}>"),
            Literal::Fun(fun) => {
                let name = fun.deref().name().lexeme();
                write!(f, "<fn {name}>")
            }
            Literal::String(s) => write!(f, "{s}"),
            Literal::Number(n) => write!(f, "{n}"),
            Literal::Nil => write!(f, "nil"),
            Literal::Bool(b) => write!(f, "{b}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Dot,
    Comma,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    This,
    True,
    Var,
    While,

    Eof,
}
