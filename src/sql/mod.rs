pub mod ast;
pub mod lexer;
pub mod parser;
pub mod token;

pub use ast::Statement;
pub use parser::{Parser, ParserError};

/// 解析一个SQL字符串并返回一个AST语句。
/// 这是提供给外部使用的主要函数。
pub fn parse_sql(sql: &str) -> Result<Statement, ParserError> {
    let mut parser = Parser::new(sql);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::parse_sql;

    #[test]
    fn test_parse_sql() {
        // Test valid SQL statements
        let valid_statements = vec![
            "CREATE TABLE users (id INT, name VARCHAR);",
            "INSERT INTO users VALUES (1, 'Alice');",
            "SELECT id, name FROM users;",
            "SELECT name FROM users", // Test without semicolon
        ];

        for sql in valid_statements {
            let result = parse_sql(sql);
            assert!(result.is_ok(), "Failed to parse valid SQL: {}", sql);
        }

        // Test invalid SQL statements
        let invalid_statements = vec!["CREATE users (id INT);", "SELECT id, name FROM;"];

        for sql in invalid_statements {
            let result = parse_sql(sql);
            assert!(result.is_err(), "Expected error for invalid SQL: {}", sql);
        }
    }
}
