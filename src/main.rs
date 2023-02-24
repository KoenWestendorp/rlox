mod ast;
mod environment;
mod interpreter;
mod parser;
mod scanner;
mod token;

use std::error::Error;
use std::fmt::Display;
use std::fs::read_to_string;
use std::io::{self, stdin, stdout, BufRead, BufReader, Write};
use std::process::exit;

use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;
use token::{Token, TokenType};

#[derive(Debug, Clone)]
pub struct LoxError {
    line: usize,
    col: usize,
    place: String, // where
    message: String,
}

impl LoxError {
    fn new(line: usize, col: usize, message: String) -> Self {
        Self {
            line,
            col,
            place: String::new(),
            message,
        }
    }

    fn with_place(line: usize, col: usize, place: String, message: String) -> Self {
        Self {
            line,
            col,
            place,
            message,
        }
    }

    fn from_token(token: Token, message: String) -> Self {
        match token.token_type() {
            TokenType::Eof => Self::with_place(token.line(), 401, " at end".to_string(), message),
            _ => Self::with_place(
                token.line(),
                501,
                format!(" at '{}'", token.lexeme()),
                message,
            ),
        }
    }

    pub(crate) fn unexpected_type(token: &Token) -> LoxError {
        LoxError::new(
            token.line(),
            32323,
            format!("Unexpected type of token {token}"),
        )
    }
}

impl Error for LoxError {}

impl Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            line,
            col,
            place,
            message,
        } = self;
        write!(f, "[line {line}, col {col}] Error {place}: {message}")
    }
}

fn run(source: &str) -> Result<String, LoxError> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    // tokens.iter().for_each(|token| println!("{}", token));

    let parser = Parser::new(tokens);
    let parsed = parser.parse()?;
    // println!("{parsed:?}");

    let mut interpreter = Interpreter::new();
    let evaluated = interpreter.interpret(parsed)?;

    Ok(evaluated)
}

fn run_file(path: &String) -> Result<(), Box<dyn Error>> {
    let source = read_to_string(path)?;
    run(&source)?;
    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let mut reader = BufReader::new(stdin().lock());
    let mut stdout = stdout().lock();

    let mut line = String::new();
    loop {
        print!("> ");
        stdout.flush()?;
        if reader.read_line(&mut line)? == 0 {
            // EOF encountered. Bye.
            break;
        }
        match run(&line) {
            Ok(output) => writeln!(stdout, "{output}")?,
            Err(e) => eprintln!("{e}"),
        }
        line.clear();
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();

    // {
    //     use crate::ast::Expr::*;
    //     use crate::token::{Literal as TokenLiteral, Token, TokenType::*};
    //     let ast = Binary {
    //         left: Box::new(Unary {
    //             operator: Token::new(Minus, "-".to_string(), None, 1),
    //             right: Box::new(Literal {
    //                 value: TokenLiteral::Number(123.0),
    //             }),
    //         }),
    //         operator: Token::new(Star, "*".to_string(), None, 1),
    //         right: Box::new(Grouping {
    //             expression: Box::new(Literal {
    //                 value: TokenLiteral::Number(45.67),
    //             }),
    //         }),
    //     };
    //     println!("{ast}");
    // }

    match args.len() {
        1 => run_prompt()?,
        2 => run_file(&args.nth(1).unwrap())?,
        _ => {
            eprintln!("Usage: rlox [script]");
            exit(64);
        }
    }

    Ok(())
}
