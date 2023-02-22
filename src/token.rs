use std::fmt::Display;

use crate::LoxError;

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    lexeme: String,
    literal: Option<Literal>,
    line: usize,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: usize,
    ) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Identifier(String),
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

    #[must_use]
    pub(crate) fn is_nil(&self) -> bool {
        match self {
            Literal::Nil => true,
            _ => false,
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
        Self::Bool(left == right)
    }

    pub(crate) fn operate_identifier(&self, f: impl Fn(String) -> String) -> Option<Self> {
        self.identifier().map(|s| Self::String(f(s.clone())))
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
}

impl Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
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
