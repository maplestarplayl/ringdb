use std::{iter::Peekable, str::Chars};

use crate::sql::token::{LexerError, Token};

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().peekable(),
        }
    }

    pub fn read_identifier(&mut self, first: char) -> String {
        let mut ident = String::new();
        ident.push(first);

        while let Some(&ch) = self.chars.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        ident
    }

    pub fn read_number(&mut self, first: char) -> i64 {
        let mut number = String::new();
        number.push(first);

        while let Some(&ch) = self.chars.peek() {
            if ch.is_ascii_digit() {
                number.push(ch);
                self.chars.next();
            } else {
                break;
            }
        }
        number.parse().unwrap_or(0)
    }

    fn read_string(&mut self) -> Result<String, LexerError> {
        let mut s = String::new();
        loop {
            match self.chars.next() {
                Some('\'') => return Ok(s),
                Some(c) => s.push(c),
                None => return Err(LexerError::UnterminatedString),
            }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else {
                break;
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        match self.chars.next() {
            Some(c) => {
                let token = match c {
                    '(' => Ok(Token::LParen),
                    ')' => Ok(Token::RParen),
                    ',' => Ok(Token::Comma),
                    ';' => Ok(Token::Semicolon),
                    '\'' => self.read_string().map(Token::String),
                    c if c.is_alphabetic() => {
                        let ident = self.read_identifier(c);
                        // 检查是否是关键字
                        match ident.to_uppercase().as_str() {
                            "CREATE" => Ok(Token::Create),
                            "TABLE" => Ok(Token::Table),
                            "INSERT" => Ok(Token::Insert),
                            "INTO" => Ok(Token::Into),
                            "VALUES" => Ok(Token::Values),
                            "SELECT" => Ok(Token::Select),
                            "FROM" => Ok(Token::From),
                            "INT" => Ok(Token::Int),
                            "VARCHAR" => Ok(Token::Varchar),
                            _ => Ok(Token::Ident(ident)),
                        }
                    }
                    c if c.is_ascii_digit() => Ok(Token::Integer(self.read_number(c))),
                    _ => Err(LexerError::InvalidCharacter(c)),
                };
                Some(token)
            }
            None => None,
        }
    }
}
