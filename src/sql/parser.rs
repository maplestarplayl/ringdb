use std::iter::Peekable;

use crate::sql::{
    ast::{Column, DataType, Statement, Value},
    lexer::Lexer,
    token::Token,
};

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken(Token),
    LexerError(String),
    Eof,
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::UnexpectedToken(t) => write!(f, "Unexpected token: {:?}", t),
            ParserError::LexerError(e) => write!(f, "Lexer error: {}", e),
            ParserError::Eof => write!(f, "Unexpected end of input"),
        }
    }
}

pub struct Parser<'a> {
    tokens: Peekable<Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            tokens: Lexer::new(input).peekable(),
        }
    }

    pub fn parse(&mut self) -> Result<Statement, ParserError> {
        let statement = self.parse_statement()?;

        match self.peek_token()? {
            Token::Semicolon => {
                self.next_token()?;
            }
            Token::Eof => {}
            t => return Err(ParserError::UnexpectedToken(t.clone())),
        }

        Ok(statement)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParserError> {
        match self.peek_token()? {
            Token::Create => self.parse_create(),
            Token::Select => self.parse_select(),
            Token::Insert => self.parse_insert(),
            t => Err(ParserError::UnexpectedToken(t.clone())),
        }
    }

    fn parse_create(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::Create)?;
        self.expect_token(Token::Table)?;
        let table_name = self.expect_identifier()?;
        self.expect_token(Token::LParen)?;

        let mut columns = Vec::new();
        if !self.check_token(Token::RParen) {
            loop {
                let col_name = self.expect_identifier()?;
                let data_type = match self.next_token()? {
                    Token::Int => DataType::Int,
                    Token::Varchar => DataType::Varchar,
                    t => return Err(ParserError::UnexpectedToken(t)),
                };
                columns.push(Column {
                    name: col_name,
                    data_type,
                });
                if !self.consume_if(Token::Comma) {
                    break;
                }
            }
        }
        self.expect_token(Token::RParen)?;
        Ok(Statement::CreateTable {
            table_name,
            columns,
        })
    }

    fn parse_select(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::Select)?;
        let mut columns = Vec::new();
        loop {
            columns.push(self.expect_identifier()?);
            if !self.consume_if(Token::Comma) {
                break;
            }
        }
        self.expect_token(Token::From)?;
        let table_name = self.expect_identifier()?;
        Ok(Statement::Select {
            table_name,
            columns,
        })
    }

    fn parse_insert(&mut self) -> Result<Statement, ParserError> {
        self.expect_token(Token::Insert)?;
        self.expect_token(Token::Into)?;
        let table_name = self.expect_identifier()?;
        self.expect_token(Token::Values)?;
        self.expect_token(Token::LParen)?;

        let mut values = Vec::new();
        if !self.check_token(Token::RParen) {
            loop {
                let value = match self.next_token()? {
                    Token::Integer(i) => Value::Integer(i),
                    Token::String(s) => Value::String(s),
                    t => return Err(ParserError::UnexpectedToken(t)),
                };
                values.push(value);
                if !self.consume_if(Token::Comma) {
                    break;
                }
            }
        }
        self.expect_token(Token::RParen)?;
        Ok(Statement::Insert { table_name, values })
    }

    // === Helper Functions ===
    fn next_token(&mut self) -> Result<Token, ParserError> {
        self.tokens
            .next()
            .unwrap_or(Ok(Token::Eof))
            .map_err(|e| ParserError::LexerError(format!("{:?}", e)))
    }

    fn peek_token(&mut self) -> Result<&Token, ParserError> {
        self.tokens
            .peek()
            .unwrap_or(&Ok(Token::Eof))
            .as_ref()
            .map_err(|e| ParserError::LexerError(format!("{:?}", e)))
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), ParserError> {
        let token = self.next_token()?;
        if token == expected {
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken(token))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParserError> {
        match self.next_token()? {
            Token::Ident(name) => Ok(name),
            t => Err(ParserError::UnexpectedToken(t)),
        }
    }

    fn check_token(&mut self, expected: Token) -> bool {
        matches!(self.peek_token(), Ok(t) if *t == expected)
    }

    fn consume_if(&mut self, expected: Token) -> bool {
        if self.check_token(expected) {
            self.next_token().is_ok()
        } else {
            false
        }
    }
}
