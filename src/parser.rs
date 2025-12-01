// Recursive Descent Parser for SQL Expression Grammar
//
// This module implements a parser that follows the EBNF grammar specification.
// It uses recursive descent parsing with proper operator precedence.

use crate::ast::*;
use crate::lexer::{Lexer, Token};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    pretty_print: bool,
    input: String,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Parse error: {}", self.message)
    }
}

impl std::error::Error for ParseError {}

type ParseResult<T> = Result<T, ParseError>;

impl Parser {
    pub fn new(input: &str) -> Result<Self, ParseError> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()
            .map_err(|e| ParseError { message: e })?;

        // Check SQLEXPR_PRETTY environment variable
        let pretty_print = std::env::var("SQLEXPR_PRETTY")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Parser {
            tokens,
            position: 0,
            pretty_print,
            input: input.to_string(),
        })
    }

    /// Get current token
    fn current_token(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }

    /// Peek at next token
    fn peek_token(&self) -> &Token {
        self.tokens.get(self.position + 1).unwrap_or(&Token::Eof)
    }

    /// Advance to next token
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    /// Expect a specific token and advance
    fn expect(&mut self, expected: Token) -> ParseResult<()> {
        if self.current_token() == &expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected {}, got {}", expected, self.current_token()),
            })
        }
    }

    /// Parse the entry point: BooleanExpression
    pub fn parse(&mut self) -> ParseResult<BooleanExpr> {
        let expr = self.parse_boolean_expression()?;
        if self.current_token() != &Token::Eof {
            return Err(ParseError {
                message: format!("Unexpected token after expression: {}", self.current_token()),
            });
        }

        // Pretty print if enabled
        if self.pretty_print {
            self.print_ast(&expr);
        }

        Ok(expr)
    }

    /// Pretty print the AST with indentation
    fn print_ast(&self, expr: &BooleanExpr) {
        println!("Input: {}", self.input);
        println!("AST:");
        self.print_boolean_expr(expr, 0);
        println!();
    }

    fn print_boolean_expr(&self, expr: &BooleanExpr, indent: usize) {
        let prefix = " ".repeat(indent);
        match expr {
            BooleanExpr::Or(left, right) => {
                println!("{}Or", prefix);
                self.print_boolean_expr(left, indent + 3);
                self.print_boolean_expr(right, indent + 3);
            }
            BooleanExpr::And(left, right) => {
                println!("{}And", prefix);
                self.print_boolean_expr(left, indent + 3);
                self.print_boolean_expr(right, indent + 3);
            }
            BooleanExpr::Not(inner) => {
                println!("{}Not", prefix);
                self.print_boolean_expr(inner, indent + 3);
            }
            BooleanExpr::Literal(b) => {
                println!("{}BooleanLiteral: {}", prefix, b);
            }
            BooleanExpr::Variable(name) => {
                println!("{}Variable: {}", prefix, name);
            }
            BooleanExpr::Relational(rel) => {
                println!("{}Relational", prefix);
                self.print_relational_expr(rel, indent + 3);
            }
        }
    }

    fn print_relational_expr(&self, expr: &RelationalExpr, indent: usize) {
        let prefix = " ".repeat(indent);
        match expr {
            RelationalExpr::Equality { left, op, right } => {
                println!("{}Equality: {:?}", prefix, op);
                self.print_value_expr(left, indent + 3);
                self.print_value_expr(right, indent + 3);
            }
            RelationalExpr::Comparison { left, op, right } => {
                println!("{}Comparison: {:?}", prefix, op);
                self.print_value_expr(left, indent + 3);
                self.print_value_expr(right, indent + 3);
            }
            RelationalExpr::Like { expr, pattern, escape, negated } => {
                println!("{}Like: negated={}, pattern='{}', escape={:?}",
                    prefix, negated, pattern, escape);
                self.print_value_expr(expr, indent + 3);
            }
            RelationalExpr::Between { expr, lower, upper, negated } => {
                println!("{}Between: negated={}", prefix, negated);
                self.print_value_expr(expr, indent + 3);
                self.print_value_expr(lower, indent + 3);
                self.print_value_expr(upper, indent + 3);
            }
            RelationalExpr::In { expr, values, negated } => {
                println!("{}In: negated={}, values={:?}", prefix, negated, values);
                self.print_value_expr(expr, indent + 3);
            }
            RelationalExpr::IsNull { expr, negated } => {
                println!("{}IsNull: negated={}", prefix, negated);
                self.print_value_expr(expr, indent + 3);
            }
        }
    }

    fn print_value_expr(&self, expr: &ValueExpr, indent: usize) {
        let prefix = " ".repeat(indent);
        match expr {
            ValueExpr::Add(left, right) => {
                println!("{}Add", prefix);
                self.print_value_expr(left, indent + 3);
                self.print_value_expr(right, indent + 3);
            }
            ValueExpr::Subtract(left, right) => {
                println!("{}Subtract", prefix);
                self.print_value_expr(left, indent + 3);
                self.print_value_expr(right, indent + 3);
            }
            ValueExpr::Multiply(left, right) => {
                println!("{}Multiply", prefix);
                self.print_value_expr(left, indent + 3);
                self.print_value_expr(right, indent + 3);
            }
            ValueExpr::Divide(left, right) => {
                println!("{}Divide", prefix);
                self.print_value_expr(left, indent + 3);
                self.print_value_expr(right, indent + 3);
            }
            ValueExpr::Modulo(left, right) => {
                println!("{}Modulo", prefix);
                self.print_value_expr(left, indent + 3);
                self.print_value_expr(right, indent + 3);
            }
            ValueExpr::UnaryPlus(inner) => {
                println!("{}UnaryPlus", prefix);
                self.print_value_expr(inner, indent + 3);
            }
            ValueExpr::UnaryMinus(inner) => {
                println!("{}UnaryMinus", prefix);
                self.print_value_expr(inner, indent + 3);
            }
            ValueExpr::Literal(lit) => {
                println!("{}Literal: {:?}", prefix, lit);
            }
            ValueExpr::Variable(name) => {
                println!("{}Variable: {}", prefix, name);
            }
        }
    }

    // ========================================================================
    // BOOLEAN EXPRESSION PARSING
    // ========================================================================

    /// BooleanExpression = BooleanOrExpression
    fn parse_boolean_expression(&mut self) -> ParseResult<BooleanExpr> {
        self.parse_boolean_or_expression()
    }

    /// BooleanOrExpression = BooleanAndExpression { "OR" BooleanAndExpression }
    fn parse_boolean_or_expression(&mut self) -> ParseResult<BooleanExpr> {
        let mut left = self.parse_boolean_and_expression()?;

        while self.current_token() == &Token::Or {
            self.advance();
            let right = self.parse_boolean_and_expression()?;
            left = BooleanExpr::Or(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// BooleanAndExpression = BooleanTerm { "AND" BooleanTerm }
    fn parse_boolean_and_expression(&mut self) -> ParseResult<BooleanExpr> {
        let mut left = self.parse_boolean_term()?;

        while self.current_token() == &Token::And {
            self.advance();
            let right = self.parse_boolean_term()?;
            left = BooleanExpr::And(Box::new(left), Box::new(right));
        }

        Ok(left)
    }

    /// BooleanTerm = "NOT" BooleanTerm
    ///             | "(" BooleanExpression ")"
    ///             | BooleanLiteral
    ///             | Variable
    ///             | RelationalExpression
    fn parse_boolean_term(&mut self) -> ParseResult<BooleanExpr> {
        match self.current_token() {
            Token::Not => {
                self.advance();
                let expr = self.parse_boolean_term()?;
                Ok(BooleanExpr::Not(Box::new(expr)))
            }
            Token::LeftParen => {
                // Need to distinguish between:
                // 1. (boolean_expr) like (x > 5) or (x > 5 AND y < 10)
                // 2. (value_expr) OP value like (x + y) > 10
                //
                // Strategy: Look ahead past the '(' to see what's inside
                // If we see patterns like "x >" or "NOT" or "TRUE/FALSE" followed by operators,
                // it's likely a boolean expression
                self.advance(); // consume '('

                // Special case: check if this is a parenthesized boolean expression
                // by looking for boolean operators or seeing if it's a complete relational expr
                let saved_pos = self.position;

                // Try parsing as a boolean expression first
                match self.parse_boolean_expression() {
                    Ok(expr) => {
                        if self.current_token() == &Token::RightParen {
                            self.advance(); // consume ')'
                            Ok(expr)
                        } else {
                            // Failed to find closing paren, might be (value_expr) OP ...
                            // Backtrack and try as relational
                            self.position = saved_pos - 1; // go back before '('
                            let rel = self.parse_relational_expression()?;
                            Ok(BooleanExpr::Relational(rel))
                        }
                    }
                    Err(_) => {
                        // Failed to parse as boolean, try as relational
                        self.position = saved_pos - 1; // go back before '('
                        let rel = self.parse_relational_expression()?;
                        Ok(BooleanExpr::Relational(rel))
                    }
                }
            }
            Token::True => {
                self.advance();
                Ok(BooleanExpr::Literal(true))
            }
            Token::False => {
                self.advance();
                Ok(BooleanExpr::Literal(false))
            }
            Token::Identifier(_) => {
                // Could be a variable or start of relational expression
                // We need to look ahead to determine which
                if self.is_relational_operator_ahead() {
                    let rel = self.parse_relational_expression()?;
                    Ok(BooleanExpr::Relational(rel))
                } else {
                    // It's a variable (boolean at runtime)
                    if let Token::Identifier(name) = self.current_token() {
                        let name = name.clone();
                        self.advance();
                        Ok(BooleanExpr::Variable(name))
                    } else {
                        unreachable!()
                    }
                }
            }
            _ => {
                // Default case: try to parse as relational expression
                // This includes literals, etc.
                let rel = self.parse_relational_expression()?;
                Ok(BooleanExpr::Relational(rel))
            }
        }
    }

    /// Check if a relational operator follows
    fn is_relational_operator_ahead(&self) -> bool {
        // Look ahead to see if there's a relational operator
        let next = self.peek_token();
        matches!(next,
            Token::Equal | Token::NotEqual |
            Token::GreaterThan | Token::GreaterOrEqual |
            Token::LessThan | Token::LessOrEqual |
            Token::Like | Token::Between | Token::In | Token::Is |
            Token::Not  // For NOT LIKE, NOT BETWEEN, NOT IN
        )
    }

    // ========================================================================
    // RELATIONAL EXPRESSION PARSING
    // ========================================================================

    /// RelationalExpression = EqualityExpression
    ///                      | ComparisonExpression
    ///                      | IsNullExpression
    fn parse_relational_expression(&mut self) -> ParseResult<RelationalExpr> {
        let left = self.parse_value_expression()?;

        match self.current_token() {
            Token::Equal => {
                self.advance();
                let right = self.parse_value_expression()?;
                Ok(RelationalExpr::Equality {
                    left,
                    op: EqualityOp::Equal,
                    right,
                })
            }
            Token::NotEqual => {
                self.advance();
                let right = self.parse_value_expression()?;
                Ok(RelationalExpr::Equality {
                    left,
                    op: EqualityOp::NotEqual,
                    right,
                })
            }
            Token::GreaterThan => {
                self.advance();
                let right = self.parse_value_expression()?;
                Ok(RelationalExpr::Comparison {
                    left,
                    op: ComparisonOp::GreaterThan,
                    right,
                })
            }
            Token::GreaterOrEqual => {
                self.advance();
                let right = self.parse_value_expression()?;
                Ok(RelationalExpr::Comparison {
                    left,
                    op: ComparisonOp::GreaterOrEqual,
                    right,
                })
            }
            Token::LessThan => {
                self.advance();
                let right = self.parse_value_expression()?;
                Ok(RelationalExpr::Comparison {
                    left,
                    op: ComparisonOp::LessThan,
                    right,
                })
            }
            Token::LessOrEqual => {
                self.advance();
                let right = self.parse_value_expression()?;
                Ok(RelationalExpr::Comparison {
                    left,
                    op: ComparisonOp::LessOrEqual,
                    right,
                })
            }
            Token::Like => {
                self.advance();
                let pattern = self.expect_string_literal()?;
                let escape = if self.current_token() == &Token::Escape {
                    self.advance();
                    Some(self.expect_string_literal()?)
                } else {
                    None
                };
                Ok(RelationalExpr::Like {
                    expr: left,
                    pattern,
                    escape,
                    negated: false,
                })
            }
            Token::Not => {
                self.advance();
                match self.current_token() {
                    Token::Like => {
                        self.advance();
                        let pattern = self.expect_string_literal()?;
                        let escape = if self.current_token() == &Token::Escape {
                            self.advance();
                            Some(self.expect_string_literal()?)
                        } else {
                            None
                        };
                        Ok(RelationalExpr::Like {
                            expr: left,
                            pattern,
                            escape,
                            negated: true,
                        })
                    }
                    Token::Between => {
                        self.advance();
                        let lower = self.parse_value_expression()?;
                        self.expect(Token::And)?;
                        let upper = self.parse_value_expression()?;
                        Ok(RelationalExpr::Between {
                            expr: left,
                            lower,
                            upper,
                            negated: true,
                        })
                    }
                    Token::In => {
                        self.advance();
                        let values = self.parse_string_list()?;
                        Ok(RelationalExpr::In {
                            expr: left,
                            values,
                            negated: true,
                        })
                    }
                    _ => Err(ParseError {
                        message: format!("Expected LIKE, BETWEEN, or IN after NOT, got {}", self.current_token()),
                    }),
                }
            }
            Token::Between => {
                self.advance();
                let lower = self.parse_value_expression()?;
                self.expect(Token::And)?;
                let upper = self.parse_value_expression()?;
                Ok(RelationalExpr::Between {
                    expr: left,
                    lower,
                    upper,
                    negated: false,
                })
            }
            Token::In => {
                self.advance();
                let values = self.parse_string_list()?;
                Ok(RelationalExpr::In {
                    expr: left,
                    values,
                    negated: false,
                })
            }
            Token::Is => {
                self.advance();
                let negated = if self.current_token() == &Token::Not {
                    self.advance();
                    true
                } else {
                    false
                };
                self.expect(Token::Null)?;
                Ok(RelationalExpr::IsNull {
                    expr: left,
                    negated,
                })
            }
            _ => Err(ParseError {
                message: format!("Expected relational operator, got {}", self.current_token()),
            }),
        }
    }

    /// Expect a string literal token
    fn expect_string_literal(&mut self) -> ParseResult<String> {
        match self.current_token() {
            Token::StringLiteral(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(ParseError {
                message: format!("Expected string literal, got {}", self.current_token()),
            }),
        }
    }

    /// Parse value literal list for IN operator: "(" Literal { "," Literal } ")"
    /// Supports both string and numeric literals
    fn parse_string_list(&mut self) -> ParseResult<Vec<ValueLiteral>> {
        self.expect(Token::LeftParen)?;

        let mut values = vec![self.expect_value_literal()?];

        while self.current_token() == &Token::Comma {
            self.advance();
            values.push(self.expect_value_literal()?);
        }

        self.expect(Token::RightParen)?;
        Ok(values)
    }

    /// Expect a value literal token (string, integer, float, etc.)
    /// Also handles unary minus for negative numbers
    fn expect_value_literal(&mut self) -> ParseResult<ValueLiteral> {
        // Handle unary minus for negative numbers
        let is_negative = if self.current_token() == &Token::Minus {
            self.advance();
            true
        } else {
            false
        };

        match self.current_token().clone() {
            Token::StringLiteral(s) => {
                if is_negative {
                    return Err(ParseError {
                        message: "Cannot apply unary minus to string literal".to_string(),
                    });
                }
                self.advance();
                Ok(ValueLiteral::String(s))
            }
            Token::IntegerLiteral(n) => {
                self.advance();
                Ok(ValueLiteral::Integer(if is_negative { -n } else { n }))
            }
            Token::FloatLiteral(f) => {
                self.advance();
                Ok(ValueLiteral::Float(if is_negative { -f } else { f }))
            }
            Token::Null => {
                if is_negative {
                    return Err(ParseError {
                        message: "Cannot apply unary minus to NULL".to_string(),
                    });
                }
                self.advance();
                Ok(ValueLiteral::Null)
            }
            Token::True => {
                if is_negative {
                    return Err(ParseError {
                        message: "Cannot apply unary minus to boolean".to_string(),
                    });
                }
                self.advance();
                Ok(ValueLiteral::Boolean(true))
            }
            Token::False => {
                if is_negative {
                    return Err(ParseError {
                        message: "Cannot apply unary minus to boolean".to_string(),
                    });
                }
                self.advance();
                Ok(ValueLiteral::Boolean(false))
            }
            _ => Err(ParseError {
                message: format!("Expected literal value, got {}", self.current_token()),
            }),
        }
    }

    // ========================================================================
    // VALUE EXPRESSION PARSING
    // ========================================================================

    /// ValueExpression = AddExpression
    fn parse_value_expression(&mut self) -> ParseResult<ValueExpr> {
        self.parse_add_expression()
    }

    /// AddExpression = MultExpression { ( "+" | "-" ) MultExpression }
    fn parse_add_expression(&mut self) -> ParseResult<ValueExpr> {
        let mut left = self.parse_mult_expression()?;

        loop {
            match self.current_token() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_mult_expression()?;
                    left = ValueExpr::Add(Box::new(left), Box::new(right));
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_mult_expression()?;
                    left = ValueExpr::Subtract(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// MultExpression = UnaryValueExpression { ( "*" | "/" | "%" ) UnaryValueExpression }
    fn parse_mult_expression(&mut self) -> ParseResult<ValueExpr> {
        let mut left = self.parse_unary_value_expression()?;

        loop {
            match self.current_token() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_unary_value_expression()?;
                    left = ValueExpr::Multiply(Box::new(left), Box::new(right));
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_unary_value_expression()?;
                    left = ValueExpr::Divide(Box::new(left), Box::new(right));
                }
                Token::Percent => {
                    self.advance();
                    let right = self.parse_unary_value_expression()?;
                    left = ValueExpr::Modulo(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    /// UnaryValueExpression = "+" UnaryValueExpression
    ///                      | "-" UnaryValueExpression
    ///                      | ValuePrimary
    fn parse_unary_value_expression(&mut self) -> ParseResult<ValueExpr> {
        match self.current_token() {
            Token::Plus => {
                self.advance();
                let expr = self.parse_unary_value_expression()?;
                Ok(ValueExpr::UnaryPlus(Box::new(expr)))
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary_value_expression()?;
                Ok(ValueExpr::UnaryMinus(Box::new(expr)))
            }
            _ => self.parse_value_primary(),
        }
    }

    /// ValuePrimary = ValueLiteral
    ///              | Variable
    ///              | "(" ValueExpression ")"
    fn parse_value_primary(&mut self) -> ParseResult<ValueExpr> {
        match self.current_token().clone() {
            Token::IntegerLiteral(n) => {
                self.advance();
                Ok(ValueExpr::Literal(ValueLiteral::Integer(n)))
            }
            Token::FloatLiteral(n) => {
                self.advance();
                Ok(ValueExpr::Literal(ValueLiteral::Float(n)))
            }
            Token::StringLiteral(s) => {
                self.advance();
                Ok(ValueExpr::Literal(ValueLiteral::String(s)))
            }
            Token::Null => {
                self.advance();
                Ok(ValueExpr::Literal(ValueLiteral::Null))
            }
            Token::True => {
                self.advance();
                Ok(ValueExpr::Literal(ValueLiteral::Boolean(true)))
            }
            Token::False => {
                self.advance();
                Ok(ValueExpr::Literal(ValueLiteral::Boolean(false)))
            }
            Token::Identifier(name) => {
                self.advance();
                Ok(ValueExpr::Variable(name))
            }
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_value_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            _ => Err(ParseError {
                message: format!("Expected value expression, got {}", self.current_token()),
            }),
        }
    }
}

/// Public API function to parse a SQL expression string
pub fn parse(input: &str) -> Result<BooleanExpr, ParseError> {
    let mut parser = Parser::new(input)?;
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_comparison() {
        let result = parse("x > 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_boolean_and() {
        let result = parse("x > 5 AND y < 10");
        assert!(result.is_ok());
    }

    #[test]
    fn test_like_operator() {
        let result = parse("name LIKE '%test%'");
        assert!(result.is_ok());
    }

    #[test]
    fn test_between() {
        let result = parse("age BETWEEN 18 AND 65");
        assert!(result.is_ok());
    }

    #[test]
    fn test_in_operator() {
        let result = parse("status IN ('active', 'pending')");
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_null() {
        let result = parse("value IS NULL");
        assert!(result.is_ok());
    }

    #[test]
    fn test_arithmetic_in_comparison() {
        let result = parse("(a + b) > (c - d)");
        assert!(result.is_ok());
    }
}
