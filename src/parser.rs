use crate::ast::{Expr, Stmt};
use crate::token::TokenType::{self, *};
use crate::token::{Literal, Token};
use crate::LoxError;

type ReturnOrError = Result<Stmt, LoxError>;

/// The parser type.
///
/// Implements a parser according to the following expression grammar:
///
/// ```
/// program        → declaration* EOF ;
///
/// declaration    → funDecl
///                | varDecl
///                | statement ;
///
/// statement      → exprStmt
///                | forStmt
///                | ifStmt
///                | printStmt
///                | returnStmt
///                | whileStmt
///                | block ;
///
/// funDecl        → "fun" function ;
/// function       → IDENTIFIER "(" parameters? ")" block ;
/// parameters     → IDENTIFIER ( "," IDENTIFIER )* ;
///
/// forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
///                  expression? ";"
///                  expression? ")" statement ;
///
/// returnStmt     → "return" expression? ";" ;
///
/// whileStmt      → "while" "(" expression ")" statement ;
///
/// ifStmt         → "if" "(" expression ")" statement
///                ( "else" statement )? ;
///
/// block          → "{" declaration* "}" ;
///
/// varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
///
/// exprStmt       → expression ";" ;
/// printStmt      → "print" expression ";" ;
///
/// expression     → assignment ;
/// assignment     → IDENTIFIER "=" assignment
///                | logic_or ;
/// logic_or       → logic_and ( "or" logic_and )* ;
/// logic_and      → equality ( "and" equality )* ;
///
/// equality       → comparison ( ( "!=" | "==" ) comparison )* ;
/// comparison     → term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
/// term           → factor ( ( "-" | "+" ) factor )* ;
/// factor         → unary ( ( "/" | "*" ) unary )* ;
/// unary          → ( "!" | "-" ) unary | call ;
/// call           → primary ( "(" arguments? ")" )* ;
/// arguments      → expression ( "," expression )* ;
/// primary        → "true" | "false" | "nil"
///                | NUMBER | STRING
///                | "(" expression ")"
///                | IDENTIFIER ;
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
        self.assignment()
    }

    /// statement      → exprStmt
    ///                | forStmt
    ///                | ifStmt
    ///                | printStmt
    ///                | returnStmt
    ///                | whileStmt
    ///                | block ;
    fn declaration(&mut self) -> Result<Stmt, LoxError> {
        if self.match_token_type(Fun) {
            return self.function("function");
        }
        let res = if self.match_token_type(Var) {
            self.var_declaration()
        } else {
            self.statement()
        };

        if res.is_err() {
            self.synchronize()
        }

        res
    }

    /// statement      → exprStmt
    ///                | forStmt
    ///                | ifStmt
    ///                | printStmt
    ///                | whileStmt
    ///                | block ;
    fn statement(&mut self) -> Result<Stmt, LoxError> {
        if self.match_token_type(For) {
            return self.for_statement();
        }
        if self.match_token_type(If) {
            return self.if_statement();
        }
        if self.match_token_type(Print) {
            return self.print_statement();
        }
        if self.match_token_type(Return) {
            return self.return_statement();
        }
        if self.match_token_type(While) {
            return self.while_statement();
        }
        if self.match_token_type(LeftBrace) {
            return Ok(Stmt::Block {
                statements: self.block()?,
            });
        }

        self.expression_statement()
    }

    /// forStmt        → "for" "(" ( varDecl | exprStmt | ";" )
    ///                  expression? ";"
    ///                  expression? ")" statement ;
    fn for_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(LeftParen, "Expect '(' after for.".to_string())?;
        let initializer = if self.match_token_type(Semicolon) {
            None
        } else if self.match_token_type(Var) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };
        let condition = if !self.check(Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(Semicolon, "Expect ';' after loop condition.".to_string())?;
        let increment = if !self.check(RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(RightParen, "Expect ')' after for clauses.".to_string())?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block {
                statements: vec![
                    body.clone(),
                    Stmt::Expression {
                        expression: increment,
                    },
                ],
            }
        }
        let condition = if condition.is_none() {
            Expr::Literal {
                value: Literal::Bool(true),
            }
        } else {
            condition.unwrap()
        };
        let mut body = Stmt::While {
            condition,
            body: Box::new(body),
        };
        if let Some(initializer) = initializer {
            body = Stmt::Block {
                statements: vec![initializer, body.clone()],
            }
        }

        Ok(body)
    }

    /// whileStmt      → "while" "(" expression ")" statement ;
    fn while_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(LeftParen, "Expect '(' after while.".to_string())?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ')' after while condition.".to_string())?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    /// ifStmt         → "if" "(" expression ")" statement
    ///                ( "else" statement )? ;
    fn if_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(LeftParen, "Expect '(' after if.".to_string())?;
        let condition = self.expression()?;
        self.consume(RightParen, "Expect ')' after if condition.".to_string())?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token_type(Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    /// exprStmt       → expression ";" ;
    fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
        let value = self.expression()?;
        self.consume(Semicolon, "Expect ';' after expression.".to_string())?;

        Ok(Stmt::Expression { expression: value })
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, LoxError> {
        let name = self
            .consume(Identifier, format!("Expect {kind} name."))?
            .clone();
        self.consume(LeftParen, format!("Expect '(' after {kind} name."))?;
        let mut params = Vec::new();
        if !self.check(RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(LoxError::from_token(
                        self.peek(),
                        "Can't have more than 255 parameters.".to_string(),
                    ));
                }

                params.push(
                    self.consume(Identifier, "Expect parameter name.".to_string())?
                        .clone(),
                );

                if !self.match_token_type(Comma) {
                    break;
                }
            }
        }
        self.consume(RightParen, "Expect ')' after parameters.".to_string())?;

        self.consume(LeftBrace, format!("Expect '{{' before {kind} body."))?;
        let body = self.block()?;

        Ok(Stmt::Function { name, params, body })
    }

    /// block          → "{" declaration* "}" ;
    fn block(&mut self) -> Result<Vec<Stmt>, LoxError> {
        let mut statements = Vec::new();

        while !self.check(RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(RightBrace, "Expect '}' after block.".to_string())?;
        Ok(statements)
    }

    /// assignment     → IDENTIFIER "=" assignment
    ///                | logic_or ;
    fn assignment(&mut self) -> Result<Expr, LoxError> {
        let expr = self.logic_or()?;

        if self.match_token_type(Equal) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable { name } = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            return Err(LoxError::from_token(
                &equals,
                "Invalid assignment target.".to_string(),
            ));
        }

        Ok(expr)
    }

    /// logic_or       → logic_and ( "or" logic_and )* ;
    fn logic_or(&mut self) -> Result<Expr, LoxError> {
        let expr = self.logic_and()?;

        if self.match_token_type(Or) {
            let operator = self.previous().clone();
            let right = self.logic_and()?;
            let expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
            return Ok(expr);
        }

        Ok(expr)
    }

    /// logic_and      → equality ( "and" equality )* ;
    fn logic_and(&mut self) -> Result<Expr, LoxError> {
        let expr = self.equality()?;

        if self.match_token_type(And) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            let expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
            return Ok(expr);
        }

        Ok(expr)
    }

    /// printStmt      → "print" expression ";" ;
    fn print_statement(&mut self) -> Result<Stmt, LoxError> {
        let value = self.expression()?;
        self.consume(Semicolon, "Expect ';' after value.".to_string())?;

        Ok(Stmt::Print { expression: value })
    }

    /// returnStmt     → "return" expression? ";" ;
    fn return_statement(&mut self) -> ReturnOrError {
        let keyword = self.previous().clone();
        // Distinguish between `return;` and `return val;`
        let value = if self.check(Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(Semicolon, "Expect ';' after return value.".to_string())?;

        Ok(Stmt::Return { keyword, value })
    }

    /// varDecl        → "var" IDENTIFIER ( "=" expression )? ";" ;
    fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
        let name = self
            .consume(Identifier, "Expect variable name.".to_string())?
            .clone();

        let initializer = if self.match_token_type(Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            Semicolon,
            "Expect ';' after variable declaration.".to_string(),
        )?;

        Ok(Stmt::Var { name, initializer })
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

    /// unary          → ( "!" | "-" ) unary | call ;
    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.match_(&[Bang, Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    /// call           → primary ( "(" arguments? ")" )* ;
    fn call(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token_type(LeftParen) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, LoxError> {
        let arguments = self.arguments()?;
        let paren = self.consume(RightParen, "Expect ')' after arguments.".to_string())?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren: paren.clone(),
            arguments,
        })
    }

    /// arguments      → expression ( "," expression )* ;
    fn arguments(&mut self) -> Result<Vec<Expr>, LoxError> {
        let mut arguments = Vec::new();
        if !self.check(RightParen) {
            loop {
                if arguments.len() >= 255 {
                    return Err(LoxError::from_token(
                        self.peek(),
                        "Can't have more than 255 arguments.".to_string(),
                    ));
                }
                arguments.push(self.expression()?);
                if !self.match_token_type(Comma) {
                    break;
                }
            }
        }

        Ok(arguments)
    }

    /// primary        → "true" | "false" | "nil"
    ///                | NUMBER | STRING
    ///                | "(" expression ")"
    ///                | IDENTIFIER ;
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

        if self.match_token_type(Identifier) {
            return Ok(Expr::Variable {
                name: self.previous().clone(),
            });
        }

        if self.match_token_type(LeftParen) {
            let expr = self.expression()?;
            self.consume(RightParen, "Expect ')' after expression.".to_string())?;
            return Ok(Expr::Grouping {
                expression: Box::new(expr),
            });
        }

        let unexpected = self.peek();
        Err(LoxError::from_token(
            unexpected,
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
        until: TokenType,
        message: std::string::String,
    ) -> Result<&Token, LoxError> {
        if self.check(until) {
            return Ok(self.advance());
        }

        // If we do not encounter the check, we have have an error on our hands.
        let unexpected = self.peek();
        Err(LoxError::from_token(unexpected, message))
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

    pub(crate) fn parse(mut self) -> Result<Vec<Stmt>, LoxError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?)
        }

        Ok(statements)
    }
}
