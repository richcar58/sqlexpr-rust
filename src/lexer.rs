// Lexer/Tokenizer for SQL Expression Parser
//
// This module handles tokenization of SQL expressions, including:
// - Case-insensitive keywords
// - String literals with SQL-style escaping
// - Numeric literals (decimal, hex, octal, floating-point)
// - Comments (line and block)
// - Whitespace handling

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords (case-insensitive)
    And,
    Or,
    Not,
    Between,
    Like,
    Escape,
    In,
    Is,
    True,
    False,
    Null,

    // Operators
    Equal,              // =
    NotEqual,           // <> or !=
    GreaterThan,        // >
    GreaterOrEqual,     // >=
    LessThan,           // <
    LessOrEqual,        // <=
    Plus,               // +
    Minus,              // -
    Star,               // *
    Slash,              // /
    Percent,            // %

    // Delimiters
    LeftParen,          // (
    RightParen,         // )
    Comma,              // ,

    // Literals
    Identifier(String),
    StringLiteral(String),
    IntegerLiteral(i64),
    FloatLiteral(f64),

    // End of input
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::And => write!(f, "AND"),
            Token::Or => write!(f, "OR"),
            Token::Not => write!(f, "NOT"),
            Token::Between => write!(f, "BETWEEN"),
            Token::Like => write!(f, "LIKE"),
            Token::Escape => write!(f, "ESCAPE"),
            Token::In => write!(f, "IN"),
            Token::Is => write!(f, "IS"),
            Token::True => write!(f, "TRUE"),
            Token::False => write!(f, "FALSE"),
            Token::Null => write!(f, "NULL"),
            Token::Equal => write!(f, "="),
            Token::NotEqual => write!(f, "<>"),
            Token::GreaterThan => write!(f, ">"),
            Token::GreaterOrEqual => write!(f, ">="),
            Token::LessThan => write!(f, "<"),
            Token::LessOrEqual => write!(f, "<="),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::Comma => write!(f, ","),
            Token::Identifier(s) => write!(f, "identifier '{}'", s),
            Token::StringLiteral(s) => write!(f, "string '{}'", s),
            Token::IntegerLiteral(n) => write!(f, "integer {}", n),
            Token::FloatLiteral(n) => write!(f, "float {}", n),
            Token::Eof => write!(f, "end of input"),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Lexer {
            input: chars,
            position: 0,
            current_char,
        }
    }

    /// Format error message with position and input context
    fn format_error(&self, message: &str) -> String {
        format!("{} near position {} in:\n  {}",
            message,
            self.position,
            String::from_iter(&self.input))
    }

    /// Advance to the next character
    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    /// Peek at the next character without advancing
    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    /// Skip whitespace characters
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Skip line comment (-- to end of line)
    fn skip_line_comment(&mut self) {
        // Skip the '--'
        self.advance();
        self.advance();

        // Skip until newline or EOF
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
    }

    /// Skip block comment (/* ... */)
    fn skip_block_comment(&mut self) -> Result<(), String> {
        // Skip the '/*'
        self.advance();
        self.advance();

        // Look for '*/'
        while let Some(ch) = self.current_char {
            if ch == '*' && self.peek() == Some('/') {
                self.advance(); // skip '*'
                self.advance(); // skip '/'
                return Ok(());
            }
            self.advance();
        }

        Err(self.format_error("Unterminated block comment"))
    }

    /// Read an identifier or keyword
    fn read_identifier(&mut self) -> String {
        let mut result = String::new();

        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        result
    }

    /// Check if identifier is a keyword (case-insensitive)
    fn keyword_or_identifier(&self, s: &str) -> Token {
        match s.to_uppercase().as_str() {
            "AND" => Token::And,
            "OR" => Token::Or,
            "NOT" => Token::Not,
            "BETWEEN" => Token::Between,
            "LIKE" => Token::Like,
            "ESCAPE" => Token::Escape,
            "IN" => Token::In,
            "IS" => Token::Is,
            "TRUE" => Token::True,
            "FALSE" => Token::False,
            "NULL" => Token::Null,
            _ => Token::Identifier(s.to_string()),
        }
    }

    /// Read a string literal with SQL-style escaping
    fn read_string_literal(&mut self) -> Result<String, String> {
        let mut result = String::new();

        // Skip opening quote
        self.advance();

        while let Some(ch) = self.current_char {
            if ch == '\'' {
                // Check for escaped quote ('')
                if self.peek() == Some('\'') {
                    result.push('\'');
                    self.advance(); // skip first '
                    self.advance(); // skip second '
                } else {
                    // End of string
                    self.advance(); // skip closing '
                    return Ok(result);
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err(self.format_error("Unterminated string literal"))
    }

    /// Read a numeric literal (integer, long, hex, octal, or float)
    fn read_number(&mut self) -> Result<Token, String> {
        // Check for hex (0x or 0X)
        if self.current_char == Some('0') && matches!(self.peek(), Some('x') | Some('X')) {
            return self.read_hex_literal();
        }

        // Check for octal (starts with 0)
        if self.current_char == Some('0') && self.peek().map_or(false, |c| c.is_ascii_digit()) {
            return self.read_octal_literal();
        }

        // Read decimal or floating point
        let mut num_str = String::new();
        let mut is_float = false;

        // Read integer part
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for decimal point
        if self.current_char == Some('.') && self.peek().map_or(false, |c| c.is_ascii_digit() || c == 'e' || c == 'E') {
            is_float = true;
            num_str.push('.');
            self.advance();

            // Read fractional part
            while let Some(ch) = self.current_char {
                if ch.is_ascii_digit() {
                    num_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Check for exponent
        if matches!(self.current_char, Some('e') | Some('E')) {
            is_float = true;
            num_str.push('e');
            self.advance();

            // Optional sign
            if matches!(self.current_char, Some('+') | Some('-')) {
                num_str.push(self.current_char.unwrap());
                self.advance();
            }

            // Exponent digits
            while let Some(ch) = self.current_char {
                if ch.is_ascii_digit() {
                    num_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Check for long suffix (l or L) - treat as regular integer
        if matches!(self.current_char, Some('l') | Some('L')) && !is_float {
            self.advance();
            let value = num_str.parse::<i64>()
                .map_err(|e| self.format_error(&format!("Invalid integer literal: {}", e)))?;
            return Ok(Token::IntegerLiteral(value));
        }

        // Parse as float or integer
        if is_float {
            let value = num_str.parse::<f64>()
                .map_err(|e| self.format_error(&format!("Invalid float literal: {}", e)))?;
            Ok(Token::FloatLiteral(value))
        } else {
            let value = num_str.parse::<i64>()
                .map_err(|e| self.format_error(&format!("Invalid integer literal: {}", e)))?;
            Ok(Token::IntegerLiteral(value))
        }
    }

    /// Read hexadecimal literal (0x...)
    fn read_hex_literal(&mut self) -> Result<Token, String> {
        // Skip '0x' or '0X'
        self.advance();
        self.advance();

        let mut hex_str = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_ascii_hexdigit() {
                hex_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if hex_str.is_empty() {
            return Err(self.format_error("Invalid hexadecimal literal: no digits after 0x"));
        }

        let value = i64::from_str_radix(&hex_str, 16)
            .map_err(|e| self.format_error(&format!("Invalid hexadecimal literal: {}", e)))?;
        Ok(Token::IntegerLiteral(value))
    }

    /// Read octal literal (0...)
    fn read_octal_literal(&mut self) -> Result<Token, String> {
        let mut octal_str = String::new();

        while let Some(ch) = self.current_char {
            if ch >= '0' && ch <= '7' {
                octal_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let value = i64::from_str_radix(&octal_str, 8)
            .map_err(|e| self.format_error(&format!("Invalid octal literal: {}", e)))?;
        Ok(Token::IntegerLiteral(value))
    }

    /// Read floating point literal starting with '.'
    fn read_float_starting_with_dot(&mut self) -> Result<Token, String> {
        let mut num_str = String::from("0.");

        // Skip the '.'
        self.advance();

        // Read fractional part
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        // Check for exponent
        if matches!(self.current_char, Some('e') | Some('E')) {
            num_str.push('e');
            self.advance();

            // Optional sign
            if matches!(self.current_char, Some('+') | Some('-')) {
                num_str.push(self.current_char.unwrap());
                self.advance();
            }

            // Exponent digits
            while let Some(ch) = self.current_char {
                if ch.is_ascii_digit() {
                    num_str.push(ch);
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let value = num_str.parse::<f64>()
            .map_err(|e| self.format_error(&format!("Invalid float literal: {}", e)))?;
        Ok(Token::FloatLiteral(value))
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Result<Token, String> {
        loop {
            // Skip whitespace
            self.skip_whitespace();

            let ch = match self.current_char {
                Some(c) => c,
                None => return Ok(Token::Eof),
            };

            // Check for comments
            if ch == '-' && self.peek() == Some('-') {
                self.skip_line_comment();
                continue;
            }

            if ch == '/' && self.peek() == Some('*') {
                self.skip_block_comment()?;
                continue;
            }

            // Single-character tokens
            match ch {
                '(' => {
                    self.advance();
                    return Ok(Token::LeftParen);
                }
                ')' => {
                    self.advance();
                    return Ok(Token::RightParen);
                }
                ',' => {
                    self.advance();
                    return Ok(Token::Comma);
                }
                '+' => {
                    self.advance();
                    return Ok(Token::Plus);
                }
                '-' => {
                    self.advance();
                    return Ok(Token::Minus);
                }
                '*' => {
                    self.advance();
                    return Ok(Token::Star);
                }
                '/' => {
                    self.advance();
                    return Ok(Token::Slash);
                }
                '%' => {
                    self.advance();
                    return Ok(Token::Percent);
                }
                '=' => {
                    self.advance();
                    return Ok(Token::Equal);
                }
                '!' => {
                    if self.peek() == Some('=') {
                        self.advance();
                        self.advance();
                        return Ok(Token::NotEqual);
                    }
                    return Err(self.format_error(&format!("Unexpected character: '{}'", ch)));
                }
                '<' => {
                    self.advance();
                    if self.current_char == Some('>') {
                        self.advance();
                        return Ok(Token::NotEqual);
                    } else if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token::LessOrEqual);
                    }
                    return Ok(Token::LessThan);
                }
                '>' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        return Ok(Token::GreaterOrEqual);
                    }
                    return Ok(Token::GreaterThan);
                }
                '\'' => {
                    let s = self.read_string_literal()?;
                    return Ok(Token::StringLiteral(s));
                }
                '.' => {
                    // Check if this is a float starting with '.'
                    if self.peek().map_or(false, |c| c.is_ascii_digit()) {
                        return self.read_float_starting_with_dot();
                    }
                    return Err(self.format_error(&format!("Unexpected character: '{}'", ch)));
                }
                _ => {
                    // Identifiers and keywords
                    if ch.is_alphabetic() || ch == '_' || ch == '$' {
                        let ident = self.read_identifier();
                        return Ok(self.keyword_or_identifier(&ident));
                    }

                    // Numbers
                    if ch.is_ascii_digit() {
                        return self.read_number();
                    }

                    return Err(self.format_error(&format!("Unexpected character: '{}'", ch)));
                }
            }
        }
    }

    /// Tokenize the entire input
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("AND or Not BETWEEN");
        assert_eq!(lexer.next_token().unwrap(), Token::And);
        assert_eq!(lexer.next_token().unwrap(), Token::Or);
        assert_eq!(lexer.next_token().unwrap(), Token::Not);
        assert_eq!(lexer.next_token().unwrap(), Token::Between);
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("'hello' 'it''s me'");
        assert_eq!(lexer.next_token().unwrap(), Token::StringLiteral("hello".to_string()));
        assert_eq!(lexer.next_token().unwrap(), Token::StringLiteral("it's me".to_string()));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 0x1A 077 3.14 1e-5 100L");
        assert_eq!(lexer.next_token().unwrap(), Token::IntegerLiteral(42));
        assert_eq!(lexer.next_token().unwrap(), Token::IntegerLiteral(26)); // 0x1A
        assert_eq!(lexer.next_token().unwrap(), Token::IntegerLiteral(63)); // 077 octal
        assert_eq!(lexer.next_token().unwrap(), Token::FloatLiteral(3.14));
        assert!(matches!(lexer.next_token().unwrap(), Token::FloatLiteral(_)));
        assert_eq!(lexer.next_token().unwrap(), Token::IntegerLiteral(100)); // 100L treated as integer
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("x -- comment\ny /* block */ z");
        assert!(matches!(lexer.next_token().unwrap(), Token::Identifier(_)));
        assert!(matches!(lexer.next_token().unwrap(), Token::Identifier(_)));
        assert!(matches!(lexer.next_token().unwrap(), Token::Identifier(_)));
    }
}
