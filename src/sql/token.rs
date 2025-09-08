#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    Create,
    Table,
    Insert,
    Into,
    Values,
    Select,
    From,
    Int,
    Varchar,

    // Identifier
    Ident(String),

    // Literals
    Integer(i64),
    String(String),

    // Symbols
    LParen,    // (
    RParen,    // )
    Comma,     // ,
    Semicolon, // ;

    // End of input
    Eof,
}

#[derive(Debug, PartialEq)]
pub enum LexerError {
    InvalidCharacter(char),
    UnterminatedString,
}
