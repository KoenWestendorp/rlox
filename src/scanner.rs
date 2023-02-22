use crate::{
    token::{Literal, Token, TokenType},
    LoxError,
};

pub(crate) struct Scanner<'s> {
    source: &'s str,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    /// 1-indexed line number.
    line: usize,
}

impl<'s> Scanner<'s> {
    pub(crate) fn new(source: &'s str) -> Self {
        Self {
            source,
            tokens: Vec::default(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub(crate) fn scan_tokens(mut self) -> Result<Vec<Token>, LoxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.push_new_token_at_line(TokenType::Eof, "".to_string(), None, self.line);
        Ok(self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<(), LoxError> {
        use TokenType::*;
        match self.advance() {
            // Good old single-characters. Nothing very spicy.
            '(' => self.push_token(LeftParen),
            ')' => self.push_token(RightParen),
            '{' => self.push_token(LeftBrace),
            '}' => self.push_token(RightBrace),
            ',' => self.push_token(Comma),
            '.' => self.push_token(Dot),
            '-' => self.push_token(Minus),
            '+' => self.push_token(Plus),
            ';' => self.push_token(Semicolon),
            '*' => self.push_token(Star),

            // Two-character or single-character?
            '!' => self.push_token_if_match_next('=', BangEqual, Bang),
            '=' => self.push_token_if_match_next('=', EqualEqual, Equal),
            '<' => self.push_token_if_match_next('=', LessEqual, Less),
            '>' => self.push_token_if_match_next('=', GreaterEqual, Greater),

            // Is it a comment or a slash...?
            '/' => {
                if self.match_next('/') {
                    // A comment goes until the end of the line.
                    while {
                        let c = self.peek();
                        c.is_some() && c.unwrap() != '\n'
                    } {
                        self.advance();
                    }
                } else {
                    self.push_token(Slash)
                }
            }

            // Onto the next line!
            '\n' => self.line += 1,
            // Ignore other whitespace.
            c if c.is_whitespace() => {}

            // String literals.
            '"' => self.string()?,

            // Number literals.
            c if c.is_ascii_digit() => self.number()?,

            // Identifier literals.
            c if c.is_ascii_alphabetic() => self.identifier()?,

            // Anything else, we throw an error.
            _ => {
                return Err(LoxError::new(
                    self.line,
                    self.col(),
                    "Unexpected character.".to_string(),
                ))
            }
        }

        Ok(())
    }

    fn char_at(&self, index: usize) -> char {
        self.source.as_bytes()[index] as char
    }

    fn current_char(&self) -> char {
        self.char_at(self.current)
    }

    pub(crate) fn advance(&mut self) -> char {
        let c = self.current_char();
        self.current += 1;
        c
    }

    pub(crate) fn push_token(&mut self, token_type: TokenType) {
        self.push_new_token(token_type, None)
    }

    fn push_new_token(&mut self, token_type: TokenType, literal: Option<Literal>) {
        let text = self.source[self.start..self.current].to_owned();
        self.push_new_token_at_line(token_type, text, literal, self.line)
    }

    fn push_new_token_at_line(
        &mut self,
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        line: usize,
    ) {
        self.tokens
            .push(Token::new(token_type, lexeme, literal, line))
    }

    pub(crate) fn col(&self) -> usize {
        // TODO: I wonder whether this unwrap_or is actually ever hit. Is there an actual case
        // where None might occur? (curiosity bikeshed)
        match self.source[..self.current].lines().last() {
            None => 0,
            Some(line) => line.len(),
        }
    }

    /// Return `true` and advance if the current source `char` equals `expected`. Otherwise, return
    /// false and remain at the current position.
    pub(crate) fn match_next(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.current_char() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    // TODO: Fix this terrible, terrible name...
    pub(crate) fn push_token_if_match_next(
        &mut self,
        expected: char,
        match_tt: TokenType,
        no_match_tt: TokenType,
    ) {
        let token_type = if self.match_next(expected) {
            match_tt
        } else {
            no_match_tt
        };
        self.push_token(token_type);
    }

    /// Look ahead at the next character without consuming it.
    pub(crate) fn peek(&self) -> Option<char> {
        if self.is_at_end() {
            return None;
        }

        Some(self.current_char())
    }

    pub(crate) fn peek_next(&self) -> Option<char> {
        let next_index = self.current + 1;
        if next_index >= self.source.len() {
            return None;
        }

        Some(self.char_at(next_index))
    }

    pub(crate) fn string(&mut self) -> Result<(), LoxError> {
        // TODO: This is some terrible work. There must be a nice way to do this. Shame let
        // chaining is not yet here...
        while {
            let c = self.peek();
            c.is_some() && c.unwrap() != '"'
        } {
            if self.peek() == Some('\n') {
                self.line += 1
            }
            self.advance();
        }

        if self.is_at_end() {
            // We have reached the end of the source code without termination of the string
            // literal.
            return Err(LoxError::new(
                self.line,
                self.col(),
                "Unterminated string.".to_string(),
            ));
        }

        // We advance for the closing ".
        self.advance();

        // Trim the surrounding quotes.
        let value = self.source[self.start + 1..self.current - 1].to_owned();
        self.push_new_token(TokenType::String, Some(Literal::String(value)));

        Ok(())
    }

    pub(crate) fn number(&mut self) -> Result<(), LoxError> {
        while {
            let c = self.peek();
            c.is_some() && c.unwrap().is_ascii_digit()
        } {
            self.advance();
        }

        // Look for the fractional part.
        if self.peek() == Some('.') && self.peek_next().unwrap_or(' ').is_ascii_digit() {
            // Consume the '.'.
            self.advance();

            while {
                let c = self.peek();
                c.is_some() && c.unwrap().is_ascii_digit()
            } {
                self.advance();
            }
        }

        // TODO: I actually don't think it is entirely safe to unwrap here... We'll see how it
        // works in practice, and might later take a look at the possible failure modes.
        let value = self.source[self.start..self.current].parse().unwrap();
        self.push_new_token(TokenType::Number, Some(Literal::Number(value)));
        Ok(())
    }

    pub(crate) fn identifier(&mut self) -> Result<(), LoxError> {
        while {
            let c = self.peek();
            c.is_some() && c.unwrap().is_ascii_alphanumeric()
        } {
            self.advance();
        }

        use TokenType::*;
        let token_type = match &self.source[self.start..self.current] {
            "and" => And,
            "class" => Class,
            "else" => Else,
            "false" => False,
            "fun" => Fun,
            "for" => For,
            "if" => If,
            "nil" => Nil,
            "or" => Or,
            "print" => Print,
            "return" => Return,
            "this" => This,
            "true" => True,
            "var" => Var,
            "while" => While,
            _ => Identifier,
        };

        self.push_token(token_type);

        Ok(())
    }
}
