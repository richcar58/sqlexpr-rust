# Evaluator Implementation Plan

## Overview

This document provides a detailed plan for implementing the SQL expression evaluator using the Visitor Pattern with Mutable Context approach, based on the specified requirements.

---

## 1. Architecture Overview

### 1.1 Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Public Interface                         │
│  evaluate(input: &str, map: &HashMap<String, RuntimeValue>)│
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    Parse Phase                               │
│              parse(input) -> BooleanExpr                     │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                 Substitution Phase                           │
│         Replace all variables with RuntimeValues             │
│         Validate all variables are bound                     │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                 Evaluation Phase                             │
│         Traverse AST and compute result                      │
│         Type checking on demand                              │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
                    Result<bool>
```

### 1.2 File Structure

All implementation in `src/evaluator.rs`:
- Public types: `RuntimeValue`, `EvalError`
- Public function: `evaluate()`
- Internal types: `SubstitutedValue`, evaluation helper structs
- Internal functions: substitution, type checking, evaluation logic

---

## 2. Type Definitions

### 2.1 RuntimeValue (Public API)

```rust
/// Runtime values that can be bound to variables
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl RuntimeValue {
    /// Helper to get type name for error messages
    fn type_name(&self) -> &'static str {
        match self {
            RuntimeValue::Integer(_) => "integer",
            RuntimeValue::Float(_) => "float",
            RuntimeValue::String(_) => "string",
            RuntimeValue::Boolean(_) => "boolean",
            RuntimeValue::Null => "NULL",
        }
    }
}
```

### 2.2 EvalError (Enhanced for better reporting)

```rust
/// Evaluation errors with detailed context
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {
    /// Parse error (from parser)
    ParseError(String),

    /// Unbound variable
    UnboundVariable {
        variable: String,
        available: Vec<String>,  // Help user see what's available
    },

    /// Type mismatch in operation
    TypeError {
        operation: String,
        expected: String,
        got: String,
        location: String,  // e.g., "left operand of '>'"
    },

    /// NULL in invalid context
    NullError {
        operation: String,
        message: String,
    },

    /// Arithmetic error
    ArithmeticError {
        operation: String,
        message: String,  // e.g., "division by zero"
    },

    /// Incompatible types in binary operation
    TypeMismatch {
        operation: String,
        left_type: String,
        right_type: String,
    },
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            EvalError::UnboundVariable { variable, available } => {
                write!(f, "Unbound variable '{}'. Available variables: [{}]",
                    variable, available.join(", "))
            }
            EvalError::TypeError { operation, expected, got, location } => {
                write!(f, "Type error in {}: expected {} for {}, got {}",
                    operation, expected, location, got)
            }
            EvalError::NullError { operation, message } => {
                write!(f, "NULL error in {}: {}", operation, message)
            }
            EvalError::ArithmeticError { operation, message } => {
                write!(f, "Arithmetic error in {}: {}", operation, message)
            }
            EvalError::TypeMismatch { operation, left_type, right_type } => {
                write!(f, "Type mismatch in {}: cannot operate on {} and {}",
                    operation, left_type, right_type)
            }
        }
    }
}

impl std::error::Error for EvalError {}

pub type EvalResult<T> = Result<T, EvalError>;
```

### 2.3 Internal Types

```rust
/// Substituted values - what AST nodes become after variable substitution
#[derive(Debug, Clone, PartialEq)]
enum SubstitutedValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl SubstitutedValue {
    /// Convert from RuntimeValue
    fn from_runtime(rv: &RuntimeValue) -> Self {
        match rv {
            RuntimeValue::Integer(i) => SubstitutedValue::Integer(*i),
            RuntimeValue::Float(f) => SubstitutedValue::Float(*f),
            RuntimeValue::String(s) => SubstitutedValue::String(s.clone()),
            RuntimeValue::Boolean(b) => SubstitutedValue::Boolean(*b),
            RuntimeValue::Null => SubstitutedValue::Null,
        }
    }

    /// Convert from ValueLiteral
    fn from_literal(lit: &ValueLiteral) -> Self {
        match lit {
            ValueLiteral::Integer(i) => SubstitutedValue::Integer(*i),
            ValueLiteral::Float(f) => SubstitutedValue::Float(*f),
            ValueLiteral::String(s) => SubstitutedValue::String(s.clone()),
            ValueLiteral::Boolean(b) => SubstitutedValue::Boolean(*b),
            ValueLiteral::Null => SubstitutedValue::Null,
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            SubstitutedValue::Integer(_) => "integer",
            SubstitutedValue::Float(_) => "float",
            SubstitutedValue::String(_) => "string",
            SubstitutedValue::Boolean(_) => "boolean",
            SubstitutedValue::Null => "NULL",
        }
    }
}
```

---

## 3. Implementation Phases

### Phase 1: Core Infrastructure

**Goal:** Set up basic types and public interface

**Tasks:**
1. Define `RuntimeValue` enum
2. Define `EvalError` enum with Display implementation
3. Define public `evaluate()` function signature
4. Add basic error conversion from parse errors

**Code:**
```rust
use std::collections::HashMap;
use crate::{parse, BooleanExpr, ValueExpr, ValueLiteral, RelationalExpr};

/// Main evaluation function
pub fn evaluate(input: &str, bindings: &HashMap<String, RuntimeValue>)
    -> EvalResult<bool>
{
    // Parse the input
    let expr = parse(input)
        .map_err(|e| EvalError::ParseError(e.to_string()))?;

    // Substitute variables
    let substituted_expr = substitute_boolean_expr(&expr, bindings)?;

    // Evaluate
    eval_boolean_expr(&substituted_expr)
}
```

**Tests:**
- Test parse error propagation
- Test empty bindings on simple literal expressions

---

### Phase 2: Variable Substitution

**Goal:** Replace all variables with values from bindings

**Strategy:**
- Traverse AST recursively
- For each Variable node, look up in HashMap
- Return error if not found
- Build new "substituted" representation

**Code Structure:**
```rust
/// Substitute all variables in a boolean expression
fn substitute_boolean_expr(
    expr: &BooleanExpr,
    bindings: &HashMap<String, RuntimeValue>
) -> EvalResult<BooleanExpr> {
    match expr {
        BooleanExpr::Literal(b) => Ok(BooleanExpr::Literal(*b)),

        BooleanExpr::Variable(name) => {
            match bindings.get(name) {
                Some(RuntimeValue::Boolean(b)) => Ok(BooleanExpr::Literal(*b)),
                Some(other) => Err(EvalError::TypeError {
                    operation: "variable binding".to_string(),
                    expected: "boolean".to_string(),
                    got: other.type_name().to_string(),
                    location: format!("variable '{}'", name),
                }),
                None => Err(EvalError::UnboundVariable {
                    variable: name.clone(),
                    available: bindings.keys().cloned().collect(),
                }),
            }
        }

        BooleanExpr::And(left, right) => {
            Ok(BooleanExpr::And(
                Box::new(substitute_boolean_expr(left, bindings)?),
                Box::new(substitute_boolean_expr(right, bindings)?)
            ))
        }

        BooleanExpr::Or(left, right) => {
            Ok(BooleanExpr::Or(
                Box::new(substitute_boolean_expr(left, bindings)?),
                Box::new(substitute_boolean_expr(right, bindings)?)
            ))
        }

        BooleanExpr::Not(expr) => {
            Ok(BooleanExpr::Not(
                Box::new(substitute_boolean_expr(expr, bindings)?)
            ))
        }

        BooleanExpr::Relational(rel) => {
            Ok(BooleanExpr::Relational(
                substitute_relational_expr(rel, bindings)?
            ))
        }
    }
}

/// Substitute variables in value expressions
fn substitute_value_expr(
    expr: &ValueExpr,
    bindings: &HashMap<String, RuntimeValue>
) -> EvalResult<ValueExpr> {
    match expr {
        ValueExpr::Literal(lit) => Ok(ValueExpr::Literal(lit.clone())),

        ValueExpr::Variable(name) => {
            match bindings.get(name) {
                Some(RuntimeValue::Integer(i)) =>
                    Ok(ValueExpr::Literal(ValueLiteral::Integer(*i))),
                Some(RuntimeValue::Float(f)) =>
                    Ok(ValueExpr::Literal(ValueLiteral::Float(*f))),
                Some(RuntimeValue::String(s)) =>
                    Ok(ValueExpr::Literal(ValueLiteral::String(s.clone()))),
                Some(RuntimeValue::Boolean(b)) =>
                    Ok(ValueExpr::Literal(ValueLiteral::Boolean(*b))),
                Some(RuntimeValue::Null) =>
                    Ok(ValueExpr::Literal(ValueLiteral::Null)),
                None => Err(EvalError::UnboundVariable {
                    variable: name.clone(),
                    available: bindings.keys().cloned().collect(),
                }),
            }
        }

        ValueExpr::Add(l, r) => Ok(ValueExpr::Add(
            Box::new(substitute_value_expr(l, bindings)?),
            Box::new(substitute_value_expr(r, bindings)?)
        )),

        // Similar for Subtract, Multiply, Divide, Modulo, UnaryPlus, UnaryMinus
        // ... (all arithmetic operations)
    }
}

/// Substitute variables in relational expressions
fn substitute_relational_expr(
    expr: &RelationalExpr,
    bindings: &HashMap<String, RuntimeValue>
) -> EvalResult<RelationalExpr> {
    match expr {
        RelationalExpr::Equality { left, op, right } => {
            Ok(RelationalExpr::Equality {
                left: substitute_value_expr(left, bindings)?,
                op: *op,
                right: substitute_value_expr(right, bindings)?,
            })
        }

        RelationalExpr::Comparison { left, op, right } => {
            Ok(RelationalExpr::Comparison {
                left: substitute_value_expr(left, bindings)?,
                op: *op,
                right: substitute_value_expr(right, bindings)?,
            })
        }

        RelationalExpr::Like { expr, pattern, escape, negated } => {
            Ok(RelationalExpr::Like {
                expr: substitute_value_expr(expr, bindings)?,
                pattern: pattern.clone(),
                escape: escape.clone(),
                negated: *negated,
            })
        }

        RelationalExpr::Between { expr, lower, upper, negated } => {
            Ok(RelationalExpr::Between {
                expr: substitute_value_expr(expr, bindings)?,
                lower: substitute_value_expr(lower, bindings)?,
                upper: substitute_value_expr(upper, bindings)?,
                negated: *negated,
            })
        }

        RelationalExpr::In { expr, values, negated } => {
            Ok(RelationalExpr::In {
                expr: substitute_value_expr(expr, bindings)?,
                values: values.clone(),  // Literals don't need substitution
                negated: *negated,
            })
        }

        RelationalExpr::IsNull { expr, negated } => {
            Ok(RelationalExpr::IsNull {
                expr: substitute_value_expr(expr, bindings)?,
                negated: *negated,
            })
        }
    }
}
```

**Tests:**
- Test unbound variable detection
- Test type mismatch on boolean variable binding
- Test successful substitution of all variable types
- Test nested expressions with multiple variables
- Test error messages include available variables

---

### Phase 3: Value Evaluation (Arithmetic & Literals)

**Goal:** Evaluate value expressions to concrete values

**Strategy:**
- After substitution, all variables are replaced with literals
- Evaluate arithmetic operations with type checking
- Handle integer to float coercion on division and mixed operations

**Code Structure:**
```rust
/// Evaluate a value expression to a concrete value
fn eval_value_expr(expr: &ValueExpr) -> EvalResult<SubstitutedValue> {
    match expr {
        ValueExpr::Literal(lit) => Ok(SubstitutedValue::from_literal(lit)),

        ValueExpr::Variable(_) => {
            // Should never happen after substitution
            panic!("Unsubstituted variable in evaluation phase")
        }

        ValueExpr::Add(l, r) => eval_arithmetic_add(l, r),
        ValueExpr::Subtract(l, r) => eval_arithmetic_subtract(l, r),
        ValueExpr::Multiply(l, r) => eval_arithmetic_multiply(l, r),
        ValueExpr::Divide(l, r) => eval_arithmetic_divide(l, r),
        ValueExpr::Modulo(l, r) => eval_arithmetic_modulo(l, r),

        ValueExpr::UnaryPlus(e) => {
            let val = eval_value_expr(e)?;
            match val {
                SubstitutedValue::Integer(i) => Ok(SubstitutedValue::Integer(i)),
                SubstitutedValue::Float(f) => Ok(SubstitutedValue::Float(f)),
                SubstitutedValue::Null => Err(EvalError::NullError {
                    operation: "unary plus".to_string(),
                    message: "cannot apply unary plus to NULL".to_string(),
                }),
                _ => Err(EvalError::TypeError {
                    operation: "unary plus".to_string(),
                    expected: "numeric".to_string(),
                    got: val.type_name().to_string(),
                    location: "operand".to_string(),
                }),
            }
        }

        ValueExpr::UnaryMinus(e) => {
            let val = eval_value_expr(e)?;
            match val {
                SubstitutedValue::Integer(i) => Ok(SubstitutedValue::Integer(-i)),
                SubstitutedValue::Float(f) => Ok(SubstitutedValue::Float(-f)),
                SubstitutedValue::Null => Err(EvalError::NullError {
                    operation: "unary minus".to_string(),
                    message: "cannot apply unary minus to NULL".to_string(),
                }),
                _ => Err(EvalError::TypeError {
                    operation: "unary minus".to_string(),
                    expected: "numeric".to_string(),
                    got: val.type_name().to_string(),
                    location: "operand".to_string(),
                }),
            }
        }
    }
}

/// Arithmetic addition with type checking and coercion
fn eval_arithmetic_add(l: &ValueExpr, r: &ValueExpr) -> EvalResult<SubstitutedValue> {
    let left = eval_value_expr(l)?;
    let right = eval_value_expr(r)?;

    // Check for NULL
    if matches!(left, SubstitutedValue::Null) || matches!(right, SubstitutedValue::Null) {
        return Err(EvalError::NullError {
            operation: "addition".to_string(),
            message: "cannot add NULL values".to_string(),
        });
    }

    match (&left, &right) {
        (SubstitutedValue::Integer(a), SubstitutedValue::Integer(b)) => {
            Ok(SubstitutedValue::Integer(a + b))
        }
        (SubstitutedValue::Float(a), SubstitutedValue::Float(b)) => {
            Ok(SubstitutedValue::Float(a + b))
        }
        // Type coercion: int + float = float
        (SubstitutedValue::Integer(a), SubstitutedValue::Float(b)) => {
            Ok(SubstitutedValue::Float(*a as f64 + b))
        }
        (SubstitutedValue::Float(a), SubstitutedValue::Integer(b)) => {
            Ok(SubstitutedValue::Float(a + *b as f64))
        }
        _ => Err(EvalError::TypeMismatch {
            operation: "addition".to_string(),
            left_type: left.type_name().to_string(),
            right_type: right.type_name().to_string(),
        }),
    }
}

/// Division with mandatory float coercion
fn eval_arithmetic_divide(l: &ValueExpr, r: &ValueExpr) -> EvalResult<SubstitutedValue> {
    let left = eval_value_expr(l)?;
    let right = eval_value_expr(r)?;

    // Check for NULL
    if matches!(left, SubstitutedValue::Null) || matches!(right, SubstitutedValue::Null) {
        return Err(EvalError::NullError {
            operation: "division".to_string(),
            message: "cannot divide NULL values".to_string(),
        });
    }

    // Convert both to float for division
    let left_float = match left {
        SubstitutedValue::Integer(i) => i as f64,
        SubstitutedValue::Float(f) => f,
        _ => return Err(EvalError::TypeError {
            operation: "division".to_string(),
            expected: "numeric".to_string(),
            got: left.type_name().to_string(),
            location: "left operand".to_string(),
        }),
    };

    let right_float = match right {
        SubstitutedValue::Integer(i) => i as f64,
        SubstitutedValue::Float(f) => f,
        _ => return Err(EvalError::TypeError {
            operation: "division".to_string(),
            expected: "numeric".to_string(),
            got: right.type_name().to_string(),
            location: "right operand".to_string(),
        }),
    };

    if right_float == 0.0 {
        return Err(EvalError::ArithmeticError {
            operation: "division".to_string(),
            message: "division by zero".to_string(),
        });
    }

    Ok(SubstitutedValue::Float(left_float / right_float))
}

// Similar implementations for subtract, multiply, modulo
```

**Tests:**
- Test integer arithmetic (add, subtract, multiply, modulo)
- Test float arithmetic
- Test mixed integer/float with coercion
- Test division always returns float
- Test division by zero error
- Test NULL in arithmetic operations
- Test type errors (string + number, etc.)
- Test unary plus and minus
- Test operator precedence preservation

---

### Phase 4: Relational Operations

**Goal:** Evaluate comparisons to boolean values

**Strategy:**
- Evaluate operands to concrete values
- Type check before comparison
- Implement each comparison operator
- Handle string comparisons, LIKE, BETWEEN, IN, IS NULL

**Code Structure:**
```rust
/// Evaluate relational expression to boolean
fn eval_relational_expr(expr: &RelationalExpr) -> EvalResult<bool> {
    match expr {
        RelationalExpr::Equality { left, op, right } => {
            eval_equality(left, right, *op)
        }

        RelationalExpr::Comparison { left, op, right } => {
            eval_comparison(left, right, *op)
        }

        RelationalExpr::Like { expr, pattern, escape, negated } => {
            eval_like(expr, pattern, escape.as_ref(), *negated)
        }

        RelationalExpr::Between { expr, lower, upper, negated } => {
            eval_between(expr, lower, upper, *negated)
        }

        RelationalExpr::In { expr, values, negated } => {
            eval_in(expr, values, *negated)
        }

        RelationalExpr::IsNull { expr, negated } => {
            eval_is_null(expr, *negated)
        }
    }
}

/// Evaluate equality/inequality
fn eval_equality(
    left: &ValueExpr,
    right: &ValueExpr,
    op: EqualityOp
) -> EvalResult<bool> {
    let l_val = eval_value_expr(left)?;
    let r_val = eval_value_expr(right)?;

    // NULL handling
    if matches!(l_val, SubstitutedValue::Null) || matches!(r_val, SubstitutedValue::Null) {
        return Err(EvalError::NullError {
            operation: format!("{:?}", op),
            message: "cannot compare NULL values (use IS NULL instead)".to_string(),
        });
    }

    let equal = match (&l_val, &r_val) {
        // Numeric comparisons
        (SubstitutedValue::Integer(a), SubstitutedValue::Integer(b)) => a == b,
        (SubstitutedValue::Float(a), SubstitutedValue::Float(b)) => a == b,
        (SubstitutedValue::Integer(a), SubstitutedValue::Float(b)) => (*a as f64) == *b,
        (SubstitutedValue::Float(a), SubstitutedValue::Integer(b)) => *a == (*b as f64),

        // String comparisons
        (SubstitutedValue::String(a), SubstitutedValue::String(b)) => a == b,

        // Boolean comparisons (only for equality)
        (SubstitutedValue::Boolean(a), SubstitutedValue::Boolean(b)) => a == b,

        // Type mismatch
        _ => return Err(EvalError::TypeMismatch {
            operation: format!("{:?}", op),
            left_type: l_val.type_name().to_string(),
            right_type: r_val.type_name().to_string(),
        }),
    };

    Ok(match op {
        EqualityOp::Equal => equal,
        EqualityOp::NotEqual => !equal,
    })
}

/// Evaluate comparison operators (>, <, >=, <=)
fn eval_comparison(
    left: &ValueExpr,
    right: &ValueExpr,
    op: ComparisonOp
) -> EvalResult<bool> {
    let l_val = eval_value_expr(left)?;
    let r_val = eval_value_expr(right)?;

    // NULL handling
    if matches!(l_val, SubstitutedValue::Null) || matches!(r_val, SubstitutedValue::Null) {
        return Err(EvalError::NullError {
            operation: format!("{:?}", op),
            message: "cannot compare NULL values".to_string(),
        });
    }

    match (&l_val, &r_val) {
        // Numeric comparisons
        (SubstitutedValue::Integer(a), SubstitutedValue::Integer(b)) => {
            Ok(apply_comparison_op(*a, *b, op))
        }
        (SubstitutedValue::Float(a), SubstitutedValue::Float(b)) => {
            Ok(apply_comparison_op(*a, *b, op))
        }
        (SubstitutedValue::Integer(a), SubstitutedValue::Float(b)) => {
            Ok(apply_comparison_op(*a as f64, *b, op))
        }
        (SubstitutedValue::Float(a), SubstitutedValue::Integer(b)) => {
            Ok(apply_comparison_op(*a, *b as f64, op))
        }

        // String comparisons (lexicographic)
        (SubstitutedValue::String(a), SubstitutedValue::String(b)) => {
            Ok(apply_comparison_op(a, b, op))
        }

        // Boolean not allowed in comparisons
        (SubstitutedValue::Boolean(_), _) | (_, SubstitutedValue::Boolean(_)) => {
            Err(EvalError::TypeError {
                operation: format!("{:?}", op),
                expected: "numeric or string".to_string(),
                got: "boolean".to_string(),
                location: "comparison operand".to_string(),
            })
        }

        // Type mismatch
        _ => Err(EvalError::TypeMismatch {
            operation: format!("{:?}", op),
            left_type: l_val.type_name().to_string(),
            right_type: r_val.type_name().to_string(),
        }),
    }
}

fn apply_comparison_op<T: PartialOrd>(a: T, b: T, op: ComparisonOp) -> bool {
    match op {
        ComparisonOp::GreaterThan => a > b,
        ComparisonOp::GreaterOrEqual => a >= b,
        ComparisonOp::LessThan => a < b,
        ComparisonOp::LessOrEqual => a <= b,
    }
}

/// Evaluate LIKE operator with wildcards
fn eval_like(
    expr: &ValueExpr,
    pattern: &str,
    escape: Option<&String>,
    negated: bool
) -> EvalResult<bool> {
    let val = eval_value_expr(expr)?;

    let string_val = match val {
        SubstitutedValue::String(s) => s,
        SubstitutedValue::Null => {
            return Err(EvalError::NullError {
                operation: "LIKE".to_string(),
                message: "cannot apply LIKE to NULL".to_string(),
            });
        }
        _ => {
            return Err(EvalError::TypeError {
                operation: "LIKE".to_string(),
                expected: "string".to_string(),
                got: val.type_name().to_string(),
                location: "left operand".to_string(),
            });
        }
    };

    let matches = match_pattern(&string_val, pattern, escape)?;
    Ok(if negated { !matches } else { matches })
}

/// Pattern matching with SQL wildcards (% = any chars, _ = single char)
fn match_pattern(s: &str, pattern: &str, escape: Option<&String>) -> EvalResult<bool> {
    let escape_char = escape.and_then(|e| e.chars().next());

    // Convert SQL pattern to regex
    let mut regex_pattern = String::from("^");
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        if Some(ch) == escape_char {
            // Escaped character - treat next character literally
            if let Some(next) = chars.next() {
                regex_pattern.push_str(&regex::escape(&next.to_string()));
            }
        } else if ch == '%' {
            regex_pattern.push_str(".*");
        } else if ch == '_' {
            regex_pattern.push('.');
        } else {
            regex_pattern.push_str(&regex::escape(&ch.to_string()));
        }
    }
    regex_pattern.push('$');

    let re = regex::Regex::new(&regex_pattern)
        .map_err(|e| EvalError::ArithmeticError {
            operation: "LIKE pattern".to_string(),
            message: format!("invalid pattern: {}", e),
        })?;

    Ok(re.is_match(s))
}

/// Evaluate BETWEEN operator
fn eval_between(
    expr: &ValueExpr,
    lower: &ValueExpr,
    upper: &ValueExpr,
    negated: bool
) -> EvalResult<bool> {
    let val = eval_value_expr(expr)?;
    let low = eval_value_expr(lower)?;
    let high = eval_value_expr(upper)?;

    // Check for NULL
    if matches!(val, SubstitutedValue::Null) ||
       matches!(low, SubstitutedValue::Null) ||
       matches!(high, SubstitutedValue::Null) {
        return Err(EvalError::NullError {
            operation: "BETWEEN".to_string(),
            message: "cannot use NULL in BETWEEN".to_string(),
        });
    }

    // All must be same comparable type
    let in_range = match (&val, &low, &high) {
        (SubstitutedValue::Integer(v), SubstitutedValue::Integer(l), SubstitutedValue::Integer(h)) => {
            v >= l && v <= h
        }
        (SubstitutedValue::Float(v), SubstitutedValue::Float(l), SubstitutedValue::Float(h)) => {
            v >= l && v <= h
        }
        (SubstitutedValue::String(v), SubstitutedValue::String(l), SubstitutedValue::String(h)) => {
            v >= l && v <= h
        }
        // Mixed numeric types need coercion
        _ => {
            // Try numeric comparison with coercion
            let v_num = to_numeric(&val)?;
            let l_num = to_numeric(&low)?;
            let h_num = to_numeric(&high)?;
            v_num >= l_num && v_num <= h_num
        }
    };

    Ok(if negated { !in_range } else { in_range })
}

/// Evaluate IN operator
fn eval_in(
    expr: &ValueExpr,
    values: &[ValueLiteral],
    negated: bool
) -> EvalResult<bool> {
    let val = eval_value_expr(expr)?;

    if matches!(val, SubstitutedValue::Null) {
        return Err(EvalError::NullError {
            operation: "IN".to_string(),
            message: "cannot use NULL in IN".to_string(),
        });
    }

    let mut found = false;
    for lit_val in values {
        let list_val = SubstitutedValue::from_literal(lit_val);

        // Check if values match (with type compatibility)
        let matches = match (&val, &list_val) {
            (SubstitutedValue::Integer(a), SubstitutedValue::Integer(b)) => a == b,
            (SubstitutedValue::Float(a), SubstitutedValue::Float(b)) => a == b,
            (SubstitutedValue::Integer(a), SubstitutedValue::Float(b)) => (*a as f64) == *b,
            (SubstitutedValue::Float(a), SubstitutedValue::Integer(b)) => *a == (*b as f64),
            (SubstitutedValue::String(a), SubstitutedValue::String(b)) => a == b,
            (SubstitutedValue::Boolean(a), SubstitutedValue::Boolean(b)) => a == b,
            _ => false,  // Type mismatch means no match
        };

        if matches {
            found = true;
            break;
        }
    }

    Ok(if negated { !found } else { found })
}

/// Evaluate IS NULL operator
fn eval_is_null(expr: &ValueExpr, negated: bool) -> EvalResult<bool> {
    let val = eval_value_expr(expr)?;
    let is_null = matches!(val, SubstitutedValue::Null);
    Ok(if negated { !is_null } else { is_null })
}

fn to_numeric(val: &SubstitutedValue) -> EvalResult<f64> {
    match val {
        SubstitutedValue::Integer(i) => Ok(*i as f64),
        SubstitutedValue::Float(f) => Ok(*f),
        _ => Err(EvalError::TypeError {
            operation: "numeric comparison".to_string(),
            expected: "numeric".to_string(),
            got: val.type_name().to_string(),
            location: "operand".to_string(),
        }),
    }
}
```

**Tests:**
- Test all comparison operators with integers
- Test all comparison operators with floats
- Test all comparison operators with strings
- Test type mismatches
- Test NULL errors in comparisons
- Test LIKE with % wildcard
- Test LIKE with _ wildcard
- Test LIKE with ESCAPE
- Test BETWEEN with numbers and strings
- Test IN with various value lists
- Test IS NULL and IS NOT NULL

---

### Phase 5: Boolean Operations

**Goal:** Evaluate boolean expressions

**Strategy:**
- After relational/literal evaluation, combine with AND/OR/NOT
- Short-circuit evaluation for performance

**Code Structure:**
```rust
/// Evaluate boolean expression
fn eval_boolean_expr(expr: &BooleanExpr) -> EvalResult<bool> {
    match expr {
        BooleanExpr::Literal(b) => Ok(*b),

        BooleanExpr::Variable(_) => {
            // Should never happen after substitution
            panic!("Unsubstituted variable in evaluation phase")
        }

        BooleanExpr::And(left, right) => {
            let l = eval_boolean_expr(left)?;
            // Short-circuit: if left is false, don't evaluate right
            if !l {
                return Ok(false);
            }
            eval_boolean_expr(right)
        }

        BooleanExpr::Or(left, right) => {
            let l = eval_boolean_expr(left)?;
            // Short-circuit: if left is true, don't evaluate right
            if l {
                return Ok(true);
            }
            eval_boolean_expr(right)
        }

        BooleanExpr::Not(expr) => {
            Ok(!eval_boolean_expr(expr)?)
        }

        BooleanExpr::Relational(rel) => {
            eval_relational_expr(rel)
        }
    }
}
```

**Tests:**
- Test AND with all combinations
- Test OR with all combinations
- Test NOT
- Test complex nested boolean expressions
- Test short-circuit behavior (if possible to observe)
- Test operator precedence (OR < AND < NOT)

---

## 4. Testing Strategy

### 4.1 Test Organization

```
tests/
  evaluator_tests.rs          # Main test file
    - Unit tests for helpers
    - Integration tests for full evaluation
```

### 4.2 Test Categories

**1. Literal Tests**
```rust
#[test]
fn test_eval_boolean_literal_true() {
    let result = evaluate("TRUE", &HashMap::new());
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_eval_boolean_literal_false() {
    let result = evaluate("FALSE", &HashMap::new());
    assert_eq!(result.unwrap(), false);
}
```

**2. Variable Binding Tests**
```rust
#[test]
fn test_eval_boolean_variable() {
    let mut bindings = HashMap::new();
    bindings.insert("active".to_string(), RuntimeValue::Boolean(true));

    let result = evaluate("active", &bindings);
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_unbound_variable_error() {
    let result = evaluate("missing", &HashMap::new());
    assert!(result.is_err());
    match result.unwrap_err() {
        EvalError::UnboundVariable { variable, .. } => {
            assert_eq!(variable, "missing");
        }
        _ => panic!("Expected UnboundVariable error"),
    }
}

#[test]
fn test_wrong_type_boolean_variable() {
    let mut bindings = HashMap::new();
    bindings.insert("x".to_string(), RuntimeValue::Integer(42));

    let result = evaluate("x", &bindings);
    assert!(result.is_err());
    match result.unwrap_err() {
        EvalError::TypeError { expected, got, .. } => {
            assert_eq!(expected, "boolean");
            assert_eq!(got, "integer");
        }
        _ => panic!("Expected TypeError"),
    }
}
```

**3. Arithmetic Tests**
```rust
#[test]
fn test_arithmetic_addition_integers() {
    let mut bindings = HashMap::new();
    bindings.insert("a".to_string(), RuntimeValue::Integer(10));
    bindings.insert("b".to_string(), RuntimeValue::Integer(20));

    let result = evaluate("a + b = 30", &bindings);
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_arithmetic_mixed_types_coercion() {
    let mut bindings = HashMap::new();
    bindings.insert("a".to_string(), RuntimeValue::Integer(10));
    bindings.insert("b".to_string(), RuntimeValue::Float(5.5));

    let result = evaluate("a + b = 15.5", &bindings);
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_division_returns_float() {
    let mut bindings = HashMap::new();
    bindings.insert("a".to_string(), RuntimeValue::Integer(10));
    bindings.insert("b".to_string(), RuntimeValue::Integer(4));

    let result = evaluate("a / b = 2.5", &bindings);
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_division_by_zero_error() {
    let mut bindings = HashMap::new();
    bindings.insert("a".to_string(), RuntimeValue::Integer(10));

    let result = evaluate("a / 0 = 5", &bindings);
    assert!(result.is_err());
    match result.unwrap_err() {
        EvalError::ArithmeticError { message, .. } => {
            assert!(message.contains("division by zero"));
        }
        _ => panic!("Expected ArithmeticError"),
    }
}

#[test]
fn test_arithmetic_null_error() {
    let mut bindings = HashMap::new();
    bindings.insert("a".to_string(), RuntimeValue::Integer(10));
    bindings.insert("b".to_string(), RuntimeValue::Null);

    let result = evaluate("a + b = 10", &bindings);
    assert!(result.is_err());
    match result.unwrap_err() {
        EvalError::NullError { .. } => {},
        _ => panic!("Expected NullError"),
    }
}
```

**4. Comparison Tests**
```rust
#[test]
fn test_comparison_integers() {
    let mut bindings = HashMap::new();
    bindings.insert("age".to_string(), RuntimeValue::Integer(25));

    assert_eq!(evaluate("age > 18", &bindings).unwrap(), true);
    assert_eq!(evaluate("age < 30", &bindings).unwrap(), true);
    assert_eq!(evaluate("age >= 25", &bindings).unwrap(), true);
    assert_eq!(evaluate("age <= 25", &bindings).unwrap(), true);
}

#[test]
fn test_comparison_strings() {
    let mut bindings = HashMap::new();
    bindings.insert("name".to_string(), RuntimeValue::String("John".to_string()));

    assert_eq!(evaluate("name > 'Alice'", &bindings).unwrap(), true);
    assert_eq!(evaluate("name < 'Zoe'", &bindings).unwrap(), true);
}

#[test]
fn test_comparison_type_mismatch() {
    let mut bindings = HashMap::new();
    bindings.insert("age".to_string(), RuntimeValue::Integer(25));
    bindings.insert("name".to_string(), RuntimeValue::String("John".to_string()));

    let result = evaluate("age > name", &bindings);
    assert!(result.is_err());
    match result.unwrap_err() {
        EvalError::TypeMismatch { left_type, right_type, .. } => {
            assert_eq!(left_type, "integer");
            assert_eq!(right_type, "string");
        }
        _ => panic!("Expected TypeMismatch"),
    }
}
```

**5. LIKE Tests**
```rust
#[test]
fn test_like_wildcard_percent() {
    let mut bindings = HashMap::new();
    bindings.insert("email".to_string(),
        RuntimeValue::String("user@example.com".to_string()));

    assert_eq!(evaluate("email LIKE '%@example.com'", &bindings).unwrap(), true);
    assert_eq!(evaluate("email LIKE 'user%'", &bindings).unwrap(), true);
    assert_eq!(evaluate("email LIKE '%example%'", &bindings).unwrap(), true);
}

#[test]
fn test_like_wildcard_underscore() {
    let mut bindings = HashMap::new();
    bindings.insert("code".to_string(), RuntimeValue::String("A1B".to_string()));

    assert_eq!(evaluate("code LIKE 'A_B'", &bindings).unwrap(), true);
    assert_eq!(evaluate("code LIKE 'A__'", &bindings).unwrap(), false);
}

#[test]
fn test_like_with_escape() {
    let mut bindings = HashMap::new();
    bindings.insert("text".to_string(),
        RuntimeValue::String("50%".to_string()));

    let result = evaluate("text LIKE '50\\%' ESCAPE '\\'", &bindings);
    assert_eq!(result.unwrap(), true);
}
```

**6. BETWEEN Tests**
```rust
#[test]
fn test_between_integers() {
    let mut bindings = HashMap::new();
    bindings.insert("age".to_string(), RuntimeValue::Integer(25));

    assert_eq!(evaluate("age BETWEEN 18 AND 30", &bindings).unwrap(), true);
    assert_eq!(evaluate("age BETWEEN 30 AND 40", &bindings).unwrap(), false);
}

#[test]
fn test_between_inclusive() {
    let mut bindings = HashMap::new();
    bindings.insert("x".to_string(), RuntimeValue::Integer(5));

    assert_eq!(evaluate("x BETWEEN 5 AND 10", &bindings).unwrap(), true);
    assert_eq!(evaluate("x BETWEEN 1 AND 5", &bindings).unwrap(), true);
}

#[test]
fn test_not_between() {
    let mut bindings = HashMap::new();
    bindings.insert("age".to_string(), RuntimeValue::Integer(15));

    assert_eq!(evaluate("age NOT BETWEEN 18 AND 65", &bindings).unwrap(), true);
}
```

**7. IN Tests**
```rust
#[test]
fn test_in_integers() {
    let mut bindings = HashMap::new();
    bindings.insert("status".to_string(), RuntimeValue::Integer(2));

    assert_eq!(evaluate("status IN (1, 2, 3)", &bindings).unwrap(), true);
    assert_eq!(evaluate("status IN (4, 5, 6)", &bindings).unwrap(), false);
}

#[test]
fn test_in_strings() {
    let mut bindings = HashMap::new();
    bindings.insert("state".to_string(),
        RuntimeValue::String("active".to_string()));

    let result = evaluate("state IN ('active', 'pending')", &bindings);
    assert_eq!(result.unwrap(), true);
}

#[test]
fn test_not_in() {
    let mut bindings = HashMap::new();
    bindings.insert("x".to_string(), RuntimeValue::Integer(5));

    assert_eq!(evaluate("x NOT IN (1, 2, 3)", &bindings).unwrap(), true);
}
```

**8. IS NULL Tests**
```rust
#[test]
fn test_is_null() {
    let mut bindings = HashMap::new();
    bindings.insert("value".to_string(), RuntimeValue::Null);

    assert_eq!(evaluate("value IS NULL", &bindings).unwrap(), true);
}

#[test]
fn test_is_not_null() {
    let mut bindings = HashMap::new();
    bindings.insert("value".to_string(), RuntimeValue::Integer(42));

    assert_eq!(evaluate("value IS NOT NULL", &bindings).unwrap(), true);
}
```

**9. Boolean Logic Tests**
```rust
#[test]
fn test_and_operator() {
    let mut bindings = HashMap::new();
    bindings.insert("a".to_string(), RuntimeValue::Boolean(true));
    bindings.insert("b".to_string(), RuntimeValue::Boolean(true));

    assert_eq!(evaluate("a AND b", &bindings).unwrap(), true);
}

#[test]
fn test_or_operator() {
    let mut bindings = HashMap::new();
    bindings.insert("a".to_string(), RuntimeValue::Boolean(false));
    bindings.insert("b".to_string(), RuntimeValue::Boolean(true));

    assert_eq!(evaluate("a OR b", &bindings).unwrap(), true);
}

#[test]
fn test_not_operator() {
    let mut bindings = HashMap::new();
    bindings.insert("active".to_string(), RuntimeValue::Boolean(false));

    assert_eq!(evaluate("NOT active", &bindings).unwrap(), true);
}

#[test]
fn test_operator_precedence() {
    let mut bindings = HashMap::new();
    bindings.insert("a".to_string(), RuntimeValue::Boolean(true));
    bindings.insert("b".to_string(), RuntimeValue::Boolean(false));
    bindings.insert("c".to_string(), RuntimeValue::Boolean(true));

    // a OR b AND c should be a OR (b AND c)
    assert_eq!(evaluate("a OR b AND c", &bindings).unwrap(), true);
}
```

**10. Complex Integration Tests**
```rust
#[test]
fn test_complex_expression_real_world() {
    let mut bindings = HashMap::new();
    bindings.insert("age".to_string(), RuntimeValue::Integer(25));
    bindings.insert("status".to_string(),
        RuntimeValue::String("active".to_string()));
    bindings.insert("score".to_string(), RuntimeValue::Float(85.5));

    let expr = "age >= 18 AND status = 'active' AND score > 80.0";
    assert_eq!(evaluate(expr, &bindings).unwrap(), true);
}

#[test]
fn test_complex_expression_with_arithmetic() {
    let mut bindings = HashMap::new();
    bindings.insert("revenue".to_string(), RuntimeValue::Float(1000.0));
    bindings.insert("cost".to_string(), RuntimeValue::Float(700.0));

    let expr = "(revenue - cost) / revenue * 100 >= 20";
    assert_eq!(evaluate(expr, &bindings).unwrap(), true);
}
```

### 4.3 Negative Tests (Error Cases)

```rust
#[test]
fn test_error_null_in_arithmetic() {
    let mut bindings = HashMap::new();
    bindings.insert("x".to_string(), RuntimeValue::Null);

    let result = evaluate("x + 5 = 10", &bindings);
    assert!(matches!(result, Err(EvalError::NullError { .. })));
}

#[test]
fn test_error_type_mismatch_in_comparison() {
    let mut bindings = HashMap::new();
    bindings.insert("age".to_string(), RuntimeValue::Integer(25));
    bindings.insert("name".to_string(), RuntimeValue::String("John".to_string()));

    let result = evaluate("age = name", &bindings);
    assert!(matches!(result, Err(EvalError::TypeMismatch { .. })));
}

#[test]
fn test_error_boolean_in_arithmetic() {
    let mut bindings = HashMap::new();
    bindings.insert("flag".to_string(), RuntimeValue::Boolean(true));

    let result = evaluate("flag + 5 = 10", &bindings);
    assert!(matches!(result, Err(EvalError::TypeError { .. })));
}
```

### 4.4 Test Coverage Goals

- **100% of operators**: Every operator tested with valid inputs
- **All error paths**: Every error variant generated in tests
- **Type combinations**: All valid type combinations tested
- **Edge cases**: NULL, division by zero, empty strings, boundary values
- **Integration**: Complex nested expressions with multiple operators

---

## 5. Documentation Plan

### 5.1 Inline Documentation

**evaluator.rs:**
```rust
//! SQL Boolean Expression Evaluator
//!
//! This module provides runtime evaluation of SQL boolean expressions
//! that have been parsed into AST form.
//!
//! # Evaluation Process
//!
//! 1. **Parsing**: Input string is parsed to BooleanExpr AST
//! 2. **Substitution**: Variables are replaced with RuntimeValues from bindings
//! 3. **Validation**: All variables must be bound, type errors detected
//! 4. **Evaluation**: AST is evaluated to produce boolean result
//!
//! # Type System
//!
//! - Integer values (i64) are compatible with each other
//! - Float values (f64) are compatible with each other
//! - Integer and Float can be mixed in arithmetic (coerced to float)
//! - Division always produces float result
//! - Strings are compatible only with other strings
//! - Booleans are compatible only with other booleans
//! - NULL cannot be used in arithmetic or comparisons (use IS NULL)
//!
//! # Examples
//!
//! ```
//! use std::collections::HashMap;
//! use sqlexpr_rust::{evaluate, RuntimeValue};
//!
//! let mut bindings = HashMap::new();
//! bindings.insert("age".to_string(), RuntimeValue::Integer(25));
//! bindings.insert("status".to_string(),
//!     RuntimeValue::String("active".to_string()));
//!
//! let result = evaluate("age > 18 AND status = 'active'", &bindings);
//! assert_eq!(result.unwrap(), true);
//! ```

/// Main evaluation function
///
/// Evaluates a SQL boolean expression with variable bindings.
///
/// # Arguments
///
/// * `input` - SQL expression string to evaluate
/// * `bindings` - HashMap mapping variable names to runtime values
///
/// # Returns
///
/// `Ok(bool)` - The result of evaluating the expression
/// `Err(EvalError)` - If parsing fails, variables are unbound, types mismatch, etc.
///
/// # Examples
///
/// ```
/// # use std::collections::HashMap;
/// # use sqlexpr_rust::{evaluate, RuntimeValue};
/// let mut bindings = HashMap::new();
/// bindings.insert("x".to_string(), RuntimeValue::Integer(10));
///
/// assert_eq!(evaluate("x > 5", &bindings).unwrap(), true);
/// assert_eq!(evaluate("x < 5", &bindings).unwrap(), false);
/// ```
pub fn evaluate(input: &str, bindings: &HashMap<String, RuntimeValue>)
    -> EvalResult<bool>
{
    // ... implementation with inline comments
}
```

**Each helper function documented:**
```rust
/// Substitute all variables in a boolean expression with their bound values
///
/// # Arguments
///
/// * `expr` - The boolean expression to substitute
/// * `bindings` - Variable bindings
///
/// # Returns
///
/// A new BooleanExpr with all variables replaced by literals
///
/// # Errors
///
/// - `UnboundVariable` if any variable is not in bindings
/// - `TypeError` if a variable's type doesn't match its usage context
fn substitute_boolean_expr(
    expr: &BooleanExpr,
    bindings: &HashMap<String, RuntimeValue>
) -> EvalResult<BooleanExpr> {
    // ...
}
```

### 5.2 README.md

Create comprehensive README:

```markdown
# SQL Expression Parser and Evaluator

A Rust library for parsing and evaluating SQL boolean expressions.

## Features

- **Complete SQL Grammar**: Supports all SQL operators
  - Boolean: AND, OR, NOT
  - Comparison: =, <>, !=, >, >=, <, <=
  - Pattern: LIKE with % and _ wildcards
  - Range: BETWEEN
  - Membership: IN
  - NULL testing: IS NULL, IS NOT NULL
  - Arithmetic: +, -, *, /, %

- **Type Safe**: Strong typing with clear error messages
- **NULL Handling**: Proper SQL NULL semantics
- **Type Coercion**: Automatic int→float conversion where appropriate

## Quick Start

### Parsing Only

```rust
use sqlexpr_rust::parse;

let expr = parse("x > 5 AND name LIKE '%test%'").unwrap();
println!("{}", expr); // Display the AST
```

### Evaluation

```rust
use std::collections::HashMap;
use sqlexpr_rust::{evaluate, RuntimeValue};

let mut bindings = HashMap::new();
bindings.insert("age".to_string(), RuntimeValue::Integer(25));
bindings.insert("status".to_string(),
    RuntimeValue::String("active".to_string()));

let result = evaluate(
    "age >= 18 AND status = 'active'",
    &bindings
).unwrap();

assert_eq!(result, true);
```

## Type System

### Runtime Types

- `Integer(i64)` - 64-bit signed integers
- `Float(f64)` - 64-bit floating point
- `String(String)` - UTF-8 strings
- `Boolean(bool)` - true/false
- `Null` - SQL NULL value

### Type Compatibility

- Integers and floats can be mixed in arithmetic
- Division always returns float
- Comparisons require matching types (except int/float)
- NULL cannot be used in arithmetic or comparisons

## Examples

See `examples/` directory for complete examples:
- `basic_evaluation.rs` - Simple variable binding
- `complex_expressions.rs` - Nested boolean logic
- `type_coercion.rs` - Int/float arithmetic
- `error_handling.rs` - Handling evaluation errors

## Error Handling

All errors include detailed context:

```rust
let result = evaluate("x > y", &HashMap::new());
// Error: "Unbound variable 'x'. Available variables: []"

let mut bindings = HashMap::new();
bindings.insert("age".to_string(), RuntimeValue::Integer(25));
bindings.insert("name".to_string(),
    RuntimeValue::String("John".to_string()));

let result = evaluate("age > name", &bindings);
// Error: "Type mismatch in GreaterThan:
//         cannot operate on integer and string"
```

## Grammar

Based on W3C EBNF notation. See `SqlExprParser-EBNF-Final.ebnf`.

## License

[Your license here]
```

### 5.3 DESIGN.md

```markdown
# Evaluator Design

## Architecture

### Two-Phase Evaluation

1. **Substitution Phase**
   - Traverse AST
   - Replace Variable nodes with Literal nodes
   - Validate all variables are bound
   - Detect type mismatches early

2. **Evaluation Phase**
   - Compute arithmetic operations
   - Evaluate comparisons
   - Apply boolean logic
   - Return final boolean result

### Why Two Phases?

- **Separation of concerns**: Binding vs computation
- **Better errors**: Can report all unbound variables at once
- **Type safety**: Catch type errors before evaluation
- **Simpler evaluation**: No context needed during eval

## Type System

### Type Coercion Rules

1. **Arithmetic with mixed int/float**
   - Integer promoted to float
   - Result is float
   - Example: `5 + 2.5 = 7.5`

2. **Division always returns float**
   - Even `10 / 5 = 2.0`
   - Prevents integer division confusion

3. **No implicit string conversion**
   - `"123" = 123` is type error
   - Explicit conversion not supported

### NULL Handling

- NULL cannot appear in arithmetic
- NULL cannot appear in comparisons
- Use `IS NULL` or `IS NOT NULL` to test for NULL
- NULL in LIKE, BETWEEN, IN is an error

This differs from SQL's three-valued logic (TRUE/FALSE/NULL) but simplifies implementation.

## Operator Semantics

### LIKE Pattern Matching

- `%` matches zero or more characters
- `_` matches exactly one character
- `ESCAPE` clause allows literal % or _
- Converted to regex for evaluation

### BETWEEN Semantics

- Inclusive on both ends
- `x BETWEEN 5 AND 10` means `x >= 5 AND x <= 10`

### IN Semantics

- Checks for membership in list
- Type-aware: `5 IN (1.0, 2.0, 5.0)` works (int→float coercion)

## Performance Considerations

### Short-Circuit Evaluation

- `AND`: If left is false, right not evaluated
- `OR`: If left is true, right not evaluated

### Memory Usage

- Substitution creates new AST (memory overhead)
- Alternative: Pass context through eval (trade memory for complexity)
- Current choice: Favor simplicity

## Error Philosophy

### Fail Fast

- Unbound variables detected before any evaluation
- Type errors detected at operation time
- Clear error messages with context

### Error Message Quality

Every error includes:
- What went wrong
- Where in the expression
- What was expected vs what was found
- Suggestions when possible (e.g., available variables)

## Future Enhancements

1. **SQL Three-Valued Logic**: TRUE/FALSE/NULL
2. **Type Coercion**: String→number, bool→number
3. **More Functions**: UPPER, LOWER, SUBSTR, etc.
4. **Optimization**: Constant folding, common subexpression elimination
5. **Caching**: Memoize subexpression results
```

### 5.4 Examples

Create `examples/` directory with:

1. **basic_evaluation.rs**
2. **complex_expressions.rs**
3. **type_coercion.rs**
4. **error_handling.rs**
5. **pattern_matching.rs** (LIKE examples)
6. **all_operators.rs** (showcase every operator)

---

## 6. Implementation Checklist

### Phase 1: Core Infrastructure ☐
- [ ] Define RuntimeValue enum
- [ ] Define EvalError enum with Display
- [ ] Define public evaluate() function
- [ ] Add parse error conversion
- [ ] Write basic tests (literals only)

### Phase 2: Variable Substitution ☐
- [ ] Implement substitute_boolean_expr()
- [ ] Implement substitute_value_expr()
- [ ] Implement substitute_relational_expr()
- [ ] Handle all variable types
- [ ] Write substitution tests
- [ ] Test unbound variable errors
- [ ] Test type mismatch errors

### Phase 3: Value Evaluation ☐
- [ ] Implement eval_value_expr()
- [ ] Implement arithmetic operations (add, subtract, multiply)
- [ ] Implement division with float coercion
- [ ] Implement modulo
- [ ] Implement unary plus/minus
- [ ] Handle NULL in arithmetic
- [ ] Handle type mismatches
- [ ] Write arithmetic tests
- [ ] Test division by zero
- [ ] Test type coercion

### Phase 4: Relational Operations ☐
- [ ] Implement eval_relational_expr()
- [ ] Implement equality/inequality
- [ ] Implement comparison operators
- [ ] Implement LIKE with pattern matching
- [ ] Implement BETWEEN
- [ ] Implement IN
- [ ] Implement IS NULL
- [ ] Write comparison tests
- [ ] Write LIKE tests (with wildcards, escape)
- [ ] Write BETWEEN tests
- [ ] Write IN tests
- [ ] Write IS NULL tests

### Phase 5: Boolean Operations ☐
- [ ] Implement eval_boolean_expr()
- [ ] Implement AND with short-circuit
- [ ] Implement OR with short-circuit
- [ ] Implement NOT
- [ ] Write boolean logic tests
- [ ] Test operator precedence
- [ ] Test complex nested expressions

### Phase 6: Testing ☐
- [ ] Achieve 100% operator coverage
- [ ] Test all error paths
- [ ] Test all type combinations
- [ ] Add integration tests
- [ ] Add negative tests
- [ ] Test edge cases

### Phase 7: Documentation ☐
- [ ] Complete inline documentation
- [ ] Write README.md
- [ ] Write DESIGN.md
- [ ] Create examples/
- [ ] Add usage examples to docs
- [ ] Document all public APIs

### Phase 8: Polish ☐
- [ ] Run clippy and fix warnings
- [ ] Format code with rustfmt
- [ ] Verify all tests pass
- [ ] Check documentation builds
- [ ] Final review

---

## 7. Success Criteria

### Functionality
- ✓ All operators work correctly
- ✓ Type system enforced
- ✓ Clear error messages
- ✓ NULL handled properly
- ✓ Operator precedence preserved

### Quality
- ✓ 100% test coverage of operators
- ✓ All error paths tested
- ✓ Documentation complete
- ✓ Examples work
- ✓ No clippy warnings

### Usability
- ✓ Simple public API
- ✓ Clear error messages
- ✓ Good examples
- ✓ Well-documented

---

## 8. Timeline Estimate

- **Phase 1-2**: 2-3 hours (core + substitution)
- **Phase 3**: 2-3 hours (arithmetic evaluation)
- **Phase 4**: 3-4 hours (relational operations)
- **Phase 5**: 1 hour (boolean operations)
- **Phase 6**: 3-4 hours (comprehensive testing)
- **Phase 7**: 2-3 hours (documentation)
- **Phase 8**: 1 hour (polish)

**Total**: 14-19 hours

This can be done incrementally, with each phase delivering working, tested functionality.
