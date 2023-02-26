mod ast;
mod callable;
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

use environment::Environment;
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

    fn from_token(token: &Token, message: String) -> Self {
        match token.token_type() {
            TokenType::Eof => {
                Self::with_place(token.line(), token.col(), "at end".to_string(), message)
            }
            _ => Self::with_place(
                token.line(),
                token.col(),
                format!("at '{}'", token.lexeme()),
                message,
            ),
        }
    }

    pub(crate) fn unexpected_type(token: &Token) -> LoxError {
        LoxError::from_token(token, format!("Unexpected type of token {token}"))
    }

    pub(crate) fn return_unwind(keyword: &Token) -> LoxError {
        LoxError::from_token(keyword, "RETURN".to_string())
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

    let parser = Parser::new(tokens);
    let parsed = parser.parse()?;

    let mut interpreter = Interpreter::new();
    let evaluated = interpreter.interpret(parsed)?;

    Ok(evaluated)
}

fn run_with_env(source: &str, environment: &mut Environment) -> Result<String, LoxError> {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;

    let parser = Parser::new(tokens);
    let parsed = parser.parse()?;

    let mut interpreter = Interpreter::new();
    let evaluated = interpreter.interpret_with_env(parsed, environment)?;

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

    let mut env = Environment::new();

    let mut line = String::new();
    loop {
        print!("> ");
        stdout.flush()?;
        if reader.read_line(&mut line)? == 0 {
            // EOF encountered. Bye.
            break;
        }
        match run_with_env(&line, &mut env) {
            Ok(output) => write!(stdout, "{output}")?,
            Err(e) => eprintln!("{e}"),
        }
        line.clear();
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    match args.len() {
        1 => run_prompt()?,
        _ => match args.nth(1).unwrap().as_str() {
            "run" => run_file(&args.next().unwrap())?,
            "batch" => {
                for file in args.collect::<Vec<_>>() {
                    eprintln!("\nRunning '{file}'...");
                    run_file(&file)?
                }
            }
            _ => {
                eprintln!("Usage:");
                eprintln!("\trlox run [script]");
                eprintln!("\trlox batch [script] [...]");
                eprintln!("\trlox");
                exit(64);
            }
        },
    }

    Ok(())
}
