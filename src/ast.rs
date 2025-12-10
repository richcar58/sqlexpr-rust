//! AST (Abstract Syntax Tree) definitions for SQL Expression Parser
//!
//! This module defines the AST structure that corresponds to the EBNF grammar.
//! The design enforces type safety at the grammar level: all top-level expressions
//! must be boolean, while arithmetic/value expressions can only appear as operands
//! to relational operators.

use std::fmt;

// ============================================================================
// BOOLEAN EXPRESSION HIERARCHY (Top Level - Always boolean)
// ============================================================================

/// Root expression type - must evaluate to boolean
#[derive(Debug, Clone, PartialEq)]
pub enum BooleanExpr {
    /// Logical OR operation (lowest precedence)
    Or(Box<BooleanExpr>, Box<BooleanExpr>),

    /// Logical AND operation
    And(Box<BooleanExpr>, Box<BooleanExpr>),

    /// Logical NOT operation
    Not(Box<BooleanExpr>),

    /// Boolean literal (TRUE or FALSE)
    Literal(bool),

    /// Variable reference (type checked at runtime)
    Variable(String),

    /// Relational expression (comparisons that produce boolean results)
    Relational(RelationalExpr),
}

// ============================================================================
// RELATIONAL EXPRESSIONS (Bridge between boolean and value expressions)
// ============================================================================

/// Relational expressions - produce boolean results from value comparisons
#[derive(Debug, Clone, PartialEq)]
pub enum RelationalExpr {
    /// Equality comparison: =, <>, !=
    Equality {
        left: ValueExpr,
        op: EqualityOp,
        right: ValueExpr,
    },

    /// Simple comparison: >, >=, <, <=
    Comparison {
        left: ValueExpr,
        op: ComparisonOp,
        right: ValueExpr,
    },

    /// LIKE pattern matching
    Like {
        expr: ValueExpr,
        pattern: String,
        escape: Option<String>,
        negated: bool,
    },

    /// BETWEEN range check
    Between {
        expr: ValueExpr,
        lower: ValueExpr,
        upper: ValueExpr,
        negated: bool,
    },

    /// IN list membership
    In {
        expr: ValueExpr,
        values: Vec<ValueLiteral>,
        negated: bool,
    },

    /// IS NULL / IS NOT NULL
    IsNull {
        expr: ValueExpr,
        negated: bool,
    },
}

/// Equality operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqualityOp {
    Equal,        // =
    NotEqual,     // <> or !=
}

/// Simple comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    GreaterThan,       // >
    GreaterOrEqual,    // >=
    LessThan,          // <
    LessOrEqual,       // <=
}

// ============================================================================
// VALUE EXPRESSION HIERARCHY (Operands only - numeric/string values)
// ============================================================================

/// Value expressions - can only appear as operands to relational operators
#[derive(Debug, Clone, PartialEq)]
pub enum ValueExpr {
    /// Binary addition
    Add(Box<ValueExpr>, Box<ValueExpr>),

    /// Binary subtraction
    Subtract(Box<ValueExpr>, Box<ValueExpr>),

    /// Binary multiplication
    Multiply(Box<ValueExpr>, Box<ValueExpr>),

    /// Binary division
    Divide(Box<ValueExpr>, Box<ValueExpr>),

    /// Binary modulo
    Modulo(Box<ValueExpr>, Box<ValueExpr>),

    /// Unary plus
    UnaryPlus(Box<ValueExpr>),

    /// Unary minus (negation)
    UnaryMinus(Box<ValueExpr>),

    /// Literal value
    Literal(ValueLiteral),

    /// Variable reference
    Variable(String),
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum ValueLiteral {
    /// Integer literal (decimal, hex, octal, or with L/l suffix)
    Integer(i64),

    /// Floating point literal
    Float(f64),

    /// String literal
    String(String),

    /// NULL literal
    Null,

    /// Boolean literal (can be used as value in comparisons)
    Boolean(bool),
}

// ============================================================================
// DISPLAY IMPLEMENTATIONS
// ============================================================================

impl fmt::Display for BooleanExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BooleanExpr::Or(left, right) => write!(f, "({} OR {})", left, right),
            BooleanExpr::And(left, right) => write!(f, "({} AND {})", left, right),
            BooleanExpr::Not(expr) => write!(f, "NOT {}", expr),
            BooleanExpr::Literal(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            BooleanExpr::Variable(name) => write!(f, "{}", name),
            BooleanExpr::Relational(rel) => write!(f, "{}", rel),
        }
    }
}

impl fmt::Display for RelationalExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RelationalExpr::Equality { left, op, right } => {
                write!(f, "{} {} {}", left, op, right)
            }
            RelationalExpr::Comparison { left, op, right } => {
                write!(f, "{} {} {}", left, op, right)
            }
            RelationalExpr::Like { expr, pattern, escape, negated } => {
                if *negated {
                    write!(f, "{} NOT LIKE '{}'", expr, pattern)?;
                } else {
                    write!(f, "{} LIKE '{}'", expr, pattern)?;
                }
                if let Some(esc) = escape {
                    write!(f, " ESCAPE '{}'", esc)?;
                }
                Ok(())
            }
            RelationalExpr::Between { expr, lower, upper, negated } => {
                if *negated {
                    write!(f, "{} NOT BETWEEN {} AND {}", expr, lower, upper)
                } else {
                    write!(f, "{} BETWEEN {} AND {}", expr, lower, upper)
                }
            }
            RelationalExpr::In { expr, values, negated } => {
                if *negated {
                    write!(f, "{} NOT IN (", expr)?;
                } else {
                    write!(f, "{} IN (", expr)?;
                }
                for (i, val) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, ")")
            }
            RelationalExpr::IsNull { expr, negated } => {
                if *negated {
                    write!(f, "{} IS NOT NULL", expr)
                } else {
                    write!(f, "{} IS NULL", expr)
                }
            }
        }
    }
}

impl fmt::Display for EqualityOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EqualityOp::Equal => write!(f, "="),
            EqualityOp::NotEqual => write!(f, "<>"),
        }
    }
}

impl fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOp::GreaterThan => write!(f, ">"),
            ComparisonOp::GreaterOrEqual => write!(f, ">="),
            ComparisonOp::LessThan => write!(f, "<"),
            ComparisonOp::LessOrEqual => write!(f, "<="),
        }
    }
}

impl fmt::Display for ValueExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueExpr::Add(left, right) => write!(f, "({} + {})", left, right),
            ValueExpr::Subtract(left, right) => write!(f, "({} - {})", left, right),
            ValueExpr::Multiply(left, right) => write!(f, "({} * {})", left, right),
            ValueExpr::Divide(left, right) => write!(f, "({} / {})", left, right),
            ValueExpr::Modulo(left, right) => write!(f, "({} % {})", left, right),
            ValueExpr::UnaryPlus(expr) => write!(f, "+{}", expr),
            ValueExpr::UnaryMinus(expr) => write!(f, "-{}", expr),
            ValueExpr::Literal(lit) => write!(f, "{}", lit),
            ValueExpr::Variable(name) => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for ValueLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueLiteral::Integer(n) => write!(f, "{}", n),
            ValueLiteral::Float(n) => write!(f, "{}", n),
            ValueLiteral::String(s) => write!(f, "'{}'", s),
            ValueLiteral::Null => write!(f, "NULL"),
            ValueLiteral::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
        }
    }
}
