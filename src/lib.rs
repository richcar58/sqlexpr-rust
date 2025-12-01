// SQL Expression Parser Library
//
// This library provides a parser for SQL boolean expressions following the
// grammar defined in SqlExprParser-EBNF-Final.ebnf
//
// The parser enforces type safety at the grammar level: all top-level expressions
// must evaluate to boolean values, while arithmetic and value expressions can only
// appear as operands to relational operators.

pub mod ast;
pub mod lexer;
pub mod parser;

// Re-export main types for convenient access
pub use ast::{
    BooleanExpr, RelationalExpr, ValueExpr, ValueLiteral,
    EqualityOp, ComparisonOp,
};
pub use parser::{parse, ParseError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parse() {
        let result = parse("x > 5");
        assert!(result.is_ok());
    }

    #[test]
    fn test_complex_expression() {
        let result = parse("(a + b) > 10 AND name LIKE '%test%'");
        assert!(result.is_ok());
    }
}
