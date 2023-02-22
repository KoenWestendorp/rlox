use crate::{
    ast::Expr,
    token::{
        Literal, Token,
        TokenType::{self, *},
    },
    LoxError,
};

/// The parser type.
///
/// Implements a parser according to the following expression grammar:
///
/// ```
/// expression     → equality ;
/// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
/// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
/// term           → factor ( ( "-" | "+" ) factor )* ;
/// factor         → unary ( ( "/" | "*" ) unary )* ;
/// unary          → ( "!" | "-" ) unary
///                | primary ;
/// primary        → NUMBER | STRING | "true" | "false" | "nil"
///                | "(" expression ")" ;
/// ```
pub(crate) struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// expression     → equality ;
    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.equality()
    }

    /// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
    fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.comparison()?;

        while self.match_(&[BangEqual, EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    /// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
    fn comparison(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.term()?;

        while self.match_(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    /// term           → factor ( ( "-" | "+" ) factor )* ;
    fn term(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.factor()?;

        while self.match_(&[Minus, Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    /// factor         → unary ( ( "/" | "*" ) unary )* ;
    fn factor(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.unary()?;

        while self.match_(&[Slash, Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    /// unary          → ( "!" | "-" ) unary
    ///                | primary ;
    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.match_(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.primary()
    }

    /// primary        → NUMBER | STRING | "true" | "false" | "nil"
    ///                | "(" expression ")" ;
    fn primary(&mut self) -> Result<Expr, LoxError> {
        if self.match_token_type(False) {
            return Ok(Expr::Literal {
                value: Literal::Bool(false),
            });
        }
        if self.match_token_type(True) {
            return Ok(Expr::Literal {
                value: Literal::Bool(true),
            });
        }
        if self.match_token_type(Nil) {
            return Ok(Expr::Literal {
                value: Literal::Nil,
            });
        }

        if self.match_(&[Number, String]) {
            return Ok(Expr::Literal {
                // I believe the use of previous after we have checked it using
                // match_token_type allows us to safely unwrap here.
                value: self.previous().literal().unwrap(),
            });
        }

        if self.match_token_type(LeftParen) {
            let expr = self.expression()?;
            self.consume(RightParen, "Expect ')' after expression.".to_string())?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        Err(LoxError::new(
            self.peek().line(),
            42,
            "Expect expression.".to_string(),
        ))
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type() == Eof
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1
        }
        self.previous()
    }

    fn match_(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            // TODO: Consider expressing this as:
            // if match_token_type(token_type) { return true }
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn match_token_type(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            return true;
        }

        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.peek().token_type() == token_type
    }

    fn consume(
        &mut self,
        token_type: TokenType,
        message: std::string::String,
    ) -> Result<&Token, LoxError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        let p = self.peek();
        Err(LoxError::new(p.line(), 69, message))
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type() == Semicolon {
                return;
            }

            match self.peek().token_type() {
                Class | Fun | Var | For | If | While | Print | Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    pub(crate) fn parse(mut self) -> Result<Expr, LoxError> {
        self.expression()
    }
}
