use crate::ast::*;
use crate::parser::{parse, ParseError};

use std::collections::HashMap;
use std::fmt;

// ============================================================================
// PUBLIC API
// ============================================================================

/// User-provided values for variable substitution
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

/// Comprehensive error type for evaluation failures
#[derive(Debug, Clone, PartialEq)]
pub enum EvalError {

    /// Parse error from the parser
    EvalParseError(String),

    /// Variable referenced but not found in value map
    UnboundVariable {
        name: String
    },

    /// Type mismatch in operation
    TypeError {
        operation: String,
        expected: String,
        actual: String,
        context: String,
    },

    /// NULL used in arithmetic or comparison operation (not IS NULL/IS NOT NULL)
    NullInOperation {
        operation: String,
        context: String,
    },

    /// Division by zero
    DivisionByZero {
        expression: String,
    },

    /// Invalid literal format
    InvalidLiteral {
        literal: String,
        literal_type: String,
        error: String,
    },
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalError::EvalParseError(msg) => write!(f, "Parse error: {}", msg),
            EvalError::UnboundVariable { name } => {
                write!(f, "Unbound variable '{}' - not found in value map", name)
            }
            EvalError::TypeError { operation, expected, actual, context } => {
                write!(f, "Type error in {}: expected {}, got {} (context: {})",
                    operation, expected, actual, context)
            }
            EvalError::NullInOperation { operation, context } => {
                write!(f, "NULL value in {} operation (context: {}). NULL is only allowed in IS NULL/IS NOT NULL",
                    operation, context)
            }
            EvalError::DivisionByZero { expression } => {
                write!(f, "Division by zero in expression: {}", expression)
            }
            EvalError::InvalidLiteral { literal, literal_type, error } => {
                write!(f, "Invalid {} literal '{}': {}", literal_type, literal, error)
            }
        }
    }
}

impl std::error::Error for EvalError {}

impl From<String> for EvalError {
    fn from(msg: String) -> Self {
        EvalError::EvalParseError(msg)
    }
}

impl From<&str> for EvalError {
    fn from(msg: &str) -> Self {
        EvalError::EvalParseError(msg.to_string())
    }
}

impl From<ParseError> for EvalError {
    fn from(msg: ParseError) -> Self {
        EvalError::EvalParseError(msg.to_string())
    }
}


/// Public evaluation function - evaluates a SQL boolean expression with variable bindings
///
/// # Arguments
/// * `input` - SQL expression string to evaluate (must be boolean-valued)
/// * `map` - Variable name to value bindings for substitution
///
/// # Returns
/// * `Ok(bool)` - The evaluated boolean result
/// * `Err(EvalError)` - Error during parsing, variable resolution, type checking, or evaluation
///
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use sqlexpr_rust::{evaluate, RuntimeValue};
///
/// let mut map = HashMap::new();
/// map.insert("x".to_string(), RuntimeValue::Integer(42));
///
/// let result = evaluate("x > 10", &map).unwrap();
/// assert_eq!(result, true);
/// ```
pub fn evaluate(input: &str, map: &HashMap<String, RuntimeValue>) -> Result<bool, EvalError> {
    let evaluator = Evaluator::new(input, map)?;
    evaluator.eval_boolean(&evaluator.ast)

}

// ============================================================================
// EVALUATOR
// ============================================================================

/// Private evaluator implementation
struct Evaluator<'a> {
    input: String,
    ast: BooleanExpr,
    value_map: &'a HashMap<String, RuntimeValue>,
}

impl<'a> Evaluator<'a> {
    /// Create new evaluator by parsing input
    fn new(input: &str, value_map: &'a HashMap<String, RuntimeValue>) -> Result<Self, EvalError> {
        let ast = parse(input)?;
        Ok(Evaluator {
            input: input.to_string(),
            ast,
            value_map,
        })
    }

    // ========================================================================
    // BOOLEAN EXPRESSION EVALUATION
    // ========================================================================

    /// Evaluate a boolean expression
    fn eval_boolean(&self, expr: &BooleanExpr) -> Result<bool, EvalError> {
        match expr {
            BooleanExpr::Literal(b) => Ok(*b),

            BooleanExpr::Variable(name) => {
                match self.value_map.get(name) {
                    Some(RuntimeValue::Boolean(b)) => Ok(*b),
                    Some(other) => Err(EvalError::TypeError {
                        operation: "boolean variable".to_string(),
                        expected: "boolean".to_string(),
                        actual: Self::runtime_type_name(other),
                        context: format!("variable '{}'", name),
                    }),
                    None => Err(EvalError::UnboundVariable {
                        name: name.clone(),
                    }),
                }
            }

            BooleanExpr::And(left, right) => {
                let l = self.eval_boolean(left)?;
                // Short-circuit: if left is false, don't evaluate right
                if !l {
                    return Ok(false);
                }
                self.eval_boolean(right)
            }

            BooleanExpr::Or(left, right) => {
                let l = self.eval_boolean(left)?;
                // Short-circuit: if left is true, don't evaluate right
                if l {
                    return Ok(true);
                }
                self.eval_boolean(right)
            }

            BooleanExpr::Not(expr) => {
                Ok(!self.eval_boolean(expr)?)
            }

            BooleanExpr::Relational(rel) => {
                self.eval_relational(rel)
            }
        }
    }

    // ========================================================================
    // RELATIONAL EXPRESSION EVALUATION
    // ========================================================================

    /// Evaluate a relational expression to boolean
    fn eval_relational(&self, expr: &RelationalExpr) -> Result<bool, EvalError> {
        match expr {
            RelationalExpr::Equality { left, op, right } => {
                self.eval_equality(left, right, *op)
            }

            RelationalExpr::Comparison { left, op, right } => {
                self.eval_comparison(left, right, *op)
            }

            RelationalExpr::Like { expr, pattern, escape, negated } => {
                self.eval_like(expr, pattern, escape.as_ref(), *negated)
            }

            RelationalExpr::Between { expr, lower, upper, negated } => {
                self.eval_between(expr, lower, upper, *negated)
            }

            RelationalExpr::In { expr, values, negated } => {
                self.eval_in(expr, values, *negated)
            }

            RelationalExpr::IsNull { expr, negated } => {
                self.eval_is_null(expr, *negated)
            }
        }
    }

    /// Evaluate equality/inequality operators
    fn eval_equality(&self, left: &ValueExpr, right: &ValueExpr, op: EqualityOp)
        -> Result<bool, EvalError>
    {
        let l_val = self.eval_value(left)?;
        let r_val = self.eval_value(right)?;

        // NULL handling
        if l_val.is_null() || r_val.is_null() {
            return Err(EvalError::NullInOperation {
                operation: format!("{:?}", op),
                context: "cannot compare NULL values (use IS NULL instead)".to_string(),
            });
        }

        let equal = match (&l_val, &r_val) {
            // Numeric comparisons
            (SubValue::Integer(a), SubValue::Integer(b)) => a == b,
            (SubValue::Float(a), SubValue::Float(b)) => a == b,
            (SubValue::Integer(a), SubValue::Float(b)) => (*a as f64) == *b,
            (SubValue::Float(a), SubValue::Integer(b)) => *a == (*b as f64),

            // String comparisons
            (SubValue::String(a), SubValue::String(b)) => a == b,

            // Boolean comparisons (only for equality)
            (SubValue::Boolean(a), SubValue::Boolean(b)) => a == b,

            // Type mismatch
            _ => return Err(EvalError::TypeError {
                operation: format!("{:?}", op),
                expected: format!("matching types"),
                actual: format!("{} vs {}", l_val.type_name(), r_val.type_name()),
                context: "equality comparison".to_string(),
            }),
        };

        Ok(match op {
            EqualityOp::Equal => equal,
            EqualityOp::NotEqual => !equal,
        })
    }

    /// Evaluate comparison operators (>, <, >=, <=)
    fn eval_comparison(&self, left: &ValueExpr, right: &ValueExpr, op: ComparisonOp)
        -> Result<bool, EvalError>
    {
        let l_val = self.eval_value(left)?;
        let r_val = self.eval_value(right)?;

        // NULL handling
        if l_val.is_null() || r_val.is_null() {
            return Err(EvalError::NullInOperation {
                operation: format!("{:?}", op),
                context: "cannot compare NULL values".to_string(),
            });
        }

        match (&l_val, &r_val) {
            // Numeric comparisons
            (SubValue::Integer(a), SubValue::Integer(b)) => {
                Ok(Self::apply_comparison_op(*a, *b, op))
            }
            (SubValue::Float(a), SubValue::Float(b)) => {
                Ok(Self::apply_comparison_op(*a, *b, op))
            }
            (SubValue::Integer(a), SubValue::Float(b)) => {
                Ok(Self::apply_comparison_op(*a as f64, *b, op))
            }
            (SubValue::Float(a), SubValue::Integer(b)) => {
                Ok(Self::apply_comparison_op(*a, *b as f64, op))
            }

            // String comparisons (lexicographic)
            (SubValue::String(a), SubValue::String(b)) => {
                Ok(Self::apply_comparison_op(a, b, op))
            }

            // Boolean not allowed in comparisons
            (SubValue::Boolean(_), _) | (_, SubValue::Boolean(_)) => {
                Err(EvalError::TypeError {
                    operation: format!("{:?}", op),
                    expected: "numeric or string".to_string(),
                    actual: "boolean".to_string(),
                    context: "comparison operand".to_string(),
                })
            }

            // Type mismatch
            _ => Err(EvalError::TypeError {
                operation: format!("{:?}", op),
                expected: "matching types".to_string(),
                actual: format!("{} vs {}", l_val.type_name(), r_val.type_name()),
                context: "comparison".to_string(),
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
    fn eval_like(&self, expr: &ValueExpr, pattern: &str, escape: Option<&String>, negated: bool)
        -> Result<bool, EvalError>
    {
        let val = self.eval_value(expr)?;

        let string_val = match val {
            SubValue::String(s) => s,
            SubValue::Null => {
                return Err(EvalError::NullInOperation {
                    operation: "LIKE".to_string(),
                    context: "cannot apply LIKE to NULL".to_string(),
                });
            }
            _ => {
                return Err(EvalError::TypeError {
                    operation: "LIKE".to_string(),
                    expected: "string".to_string(),
                    actual: val.type_name(),
                    context: "left operand".to_string(),
                });
            }
        };

        let matches = Self::match_pattern(&string_val, pattern, escape)?;
        Ok(if negated { !matches } else { matches })
    }

    /// Pattern matching with SQL wildcards (% = any chars, _ = single char)
    fn match_pattern(s: &str, pattern: &str, escape: Option<&String>) -> Result<bool, EvalError> {
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
            .map_err(|e| EvalError::InvalidLiteral {
                literal: pattern.to_string(),
                literal_type: "LIKE pattern".to_string(),
                error: format!("{}", e),
            })?;

        Ok(re.is_match(s))
    }

    /// Evaluate BETWEEN operator
    fn eval_between(&self, expr: &ValueExpr, lower: &ValueExpr, upper: &ValueExpr, negated: bool)
        -> Result<bool, EvalError>
    {
        let val = self.eval_value(expr)?;
        let low = self.eval_value(lower)?;
        let high = self.eval_value(upper)?;

        // Check for NULL
        if val.is_null() || low.is_null() || high.is_null() {
            return Err(EvalError::NullInOperation {
                operation: "BETWEEN".to_string(),
                context: "cannot use NULL in BETWEEN".to_string(),
            });
        }

        // All must be same comparable type
        let in_range = match (&val, &low, &high) {
            (SubValue::Integer(v), SubValue::Integer(l), SubValue::Integer(h)) => {
                v >= l && v <= h
            }
            (SubValue::Float(v), SubValue::Float(l), SubValue::Float(h)) => {
                v >= l && v <= h
            }
            (SubValue::String(v), SubValue::String(l), SubValue::String(h)) => {
                v >= l && v <= h
            }
            // Mixed numeric types need coercion
            _ => {
                // Try numeric comparison with coercion
                let v_num = Self::to_numeric(&val)?;
                let l_num = Self::to_numeric(&low)?;
                let h_num = Self::to_numeric(&high)?;
                v_num >= l_num && v_num <= h_num
            }
        };

        Ok(if negated { !in_range } else { in_range })
    }

    /// Evaluate IN operator
    fn eval_in(&self, expr: &ValueExpr, values: &[ValueLiteral], negated: bool)
        -> Result<bool, EvalError>
    {
        let val = self.eval_value(expr)?;

        if val.is_null() {
            return Err(EvalError::NullInOperation {
                operation: "IN".to_string(),
                context: "cannot use NULL in IN".to_string(),
            });
        }

        // Type consistency of the values list is now guaranteed by the parser,
        // so we only need to check if the left operand is type-compatible with the list
        if !values.is_empty() {
            let first_list_val = SubValue::from_literal(&values[0]);
            let is_compatible = Self::are_types_compatible_for_in(&val, &first_list_val);

            // If list is incompatible with left value, it's a type error
            if !is_compatible {
                return Err(EvalError::TypeError {
                    operation: "IN".to_string(),
                    expected: first_list_val.type_name(),
                    actual: val.type_name(),
                    context: "left operand type doesn't match list element types".to_string(),
                });
            }

            // Note: Type consistency check for list elements removed - now done at parse time
        }

        let mut found = false;
        for lit_val in values {
            let list_val = SubValue::from_literal(lit_val);

            // Check if values match (with type compatibility)
            let matches = match (&val, &list_val) {
                (SubValue::Integer(a), SubValue::Integer(b)) => a == b,
                (SubValue::Float(a), SubValue::Float(b)) => a == b,
                (SubValue::Integer(a), SubValue::Float(b)) => (*a as f64) == *b,
                (SubValue::Float(a), SubValue::Integer(b)) => *a == (*b as f64),
                (SubValue::String(a), SubValue::String(b)) => a == b,
                (SubValue::Boolean(a), SubValue::Boolean(b)) => a == b,
                (SubValue::Null, SubValue::Null) => true,
                _ => false,  // Means no match since type mismatches are caused above
            };

            if matches {
                found = true;
                break;
            }
        }

        Ok(if negated { !found } else { found })
    }

    /// Evaluate IS NULL operator
    fn eval_is_null(&self, expr: &ValueExpr, negated: bool) -> Result<bool, EvalError> {
        let val = self.eval_value(expr)?;
        let is_null = val.is_null();
        Ok(if negated { !is_null } else { is_null })
    }

    // ========================================================================
    // VALUE EXPRESSION EVALUATION
    // ========================================================================

    /// Evaluate a value expression to a concrete value
    fn eval_value(&self, expr: &ValueExpr) -> Result<SubValue, EvalError> {
        match expr {
            ValueExpr::Literal(lit) => Ok(SubValue::from_literal(lit)),

            ValueExpr::Variable(name) => {
                match self.value_map.get(name) {
                    Some(rv) => Ok(SubValue::from_runtime(rv)),
                    None => Err(EvalError::UnboundVariable {
                        name: name.clone(),
                    }),
                }
            }

            ValueExpr::Add(l, r) => self.eval_arithmetic_add(l, r),
            ValueExpr::Subtract(l, r) => self.eval_arithmetic_subtract(l, r),
            ValueExpr::Multiply(l, r) => self.eval_arithmetic_multiply(l, r),
            ValueExpr::Divide(l, r) => self.eval_arithmetic_divide(l, r),
            ValueExpr::Modulo(l, r) => self.eval_arithmetic_modulo(l, r),

            ValueExpr::UnaryPlus(e) => {
                let val = self.eval_value(e)?;
                match val {
                    SubValue::Integer(i) => Ok(SubValue::Integer(i)),
                    SubValue::Float(f) => Ok(SubValue::Float(f)),
                    SubValue::Null => Err(EvalError::NullInOperation {
                        operation: "unary plus".to_string(),
                        context: "cannot apply unary plus to NULL".to_string(),
                    }),
                    _ => Err(EvalError::TypeError {
                        operation: "unary plus".to_string(),
                        expected: "numeric".to_string(),
                        actual: val.type_name(),
                        context: "operand".to_string(),
                    }),
                }
            }

            ValueExpr::UnaryMinus(e) => {
                let val = self.eval_value(e)?;
                match val {
                    SubValue::Integer(i) => Ok(SubValue::Integer(-i)),
                    SubValue::Float(f) => Ok(SubValue::Float(-f)),
                    SubValue::Null => Err(EvalError::NullInOperation {
                        operation: "unary minus".to_string(),
                        context: "cannot apply unary minus to NULL".to_string(),
                    }),
                    _ => Err(EvalError::TypeError {
                        operation: "unary minus".to_string(),
                        expected: "numeric".to_string(),
                        actual: val.type_name(),
                        context: "operand".to_string(),
                    }),
                }
            }
        }
    }

    /// Arithmetic addition with type checking and coercion
    fn eval_arithmetic_add(&self, l: &ValueExpr, r: &ValueExpr) -> Result<SubValue, EvalError> {
        let left = self.eval_value(l)?;
        let right = self.eval_value(r)?;

        // Check for NULL
        if left.is_null() || right.is_null() {
            return Err(EvalError::NullInOperation {
                operation: "addition".to_string(),
                context: "cannot add NULL values".to_string(),
            });
        }

        match (&left, &right) {
            (SubValue::Integer(a), SubValue::Integer(b)) => {
                Ok(SubValue::Integer(a + b))
            }
            (SubValue::Float(a), SubValue::Float(b)) => {
                Ok(SubValue::Float(a + b))
            }
            // Type coercion: int + float = float
            (SubValue::Integer(a), SubValue::Float(b)) => {
                Ok(SubValue::Float(*a as f64 + b))
            }
            (SubValue::Float(a), SubValue::Integer(b)) => {
                Ok(SubValue::Float(a + *b as f64))
            }
            _ => Err(EvalError::TypeError {
                operation: "addition".to_string(),
                expected: "numeric types".to_string(),
                actual: format!("{} and {}", left.type_name(), right.type_name()),
                context: "arithmetic operation".to_string(),
            }),
        }
    }

    /// Arithmetic subtraction
    fn eval_arithmetic_subtract(&self, l: &ValueExpr, r: &ValueExpr) -> Result<SubValue, EvalError> {
        let left = self.eval_value(l)?;
        let right = self.eval_value(r)?;

        if left.is_null() || right.is_null() {
            return Err(EvalError::NullInOperation {
                operation: "subtraction".to_string(),
                context: "cannot subtract NULL values".to_string(),
            });
        }

        match (&left, &right) {
            (SubValue::Integer(a), SubValue::Integer(b)) => {
                Ok(SubValue::Integer(a - b))
            }
            (SubValue::Float(a), SubValue::Float(b)) => {
                Ok(SubValue::Float(a - b))
            }
            (SubValue::Integer(a), SubValue::Float(b)) => {
                Ok(SubValue::Float(*a as f64 - b))
            }
            (SubValue::Float(a), SubValue::Integer(b)) => {
                Ok(SubValue::Float(a - *b as f64))
            }
            _ => Err(EvalError::TypeError {
                operation: "subtraction".to_string(),
                expected: "numeric types".to_string(),
                actual: format!("{} and {}", left.type_name(), right.type_name()),
                context: "arithmetic operation".to_string(),
            }),
        }
    }

    /// Arithmetic multiplication
    fn eval_arithmetic_multiply(&self, l: &ValueExpr, r: &ValueExpr) -> Result<SubValue, EvalError> {
        let left = self.eval_value(l)?;
        let right = self.eval_value(r)?;

        if left.is_null() || right.is_null() {
            return Err(EvalError::NullInOperation {
                operation: "multiplication".to_string(),
                context: "cannot multiply NULL values".to_string(),
            });
        }

        match (&left, &right) {
            (SubValue::Integer(a), SubValue::Integer(b)) => {
                Ok(SubValue::Integer(a * b))
            }
            (SubValue::Float(a), SubValue::Float(b)) => {
                Ok(SubValue::Float(a * b))
            }
            (SubValue::Integer(a), SubValue::Float(b)) => {
                Ok(SubValue::Float(*a as f64 * b))
            }
            (SubValue::Float(a), SubValue::Integer(b)) => {
                Ok(SubValue::Float(a * *b as f64))
            }
            _ => Err(EvalError::TypeError {
                operation: "multiplication".to_string(),
                expected: "numeric types".to_string(),
                actual: format!("{} and {}", left.type_name(), right.type_name()),
                context: "arithmetic operation".to_string(),
            }),
        }
    }

    /// Division with mandatory float coercion
    fn eval_arithmetic_divide(&self, l: &ValueExpr, r: &ValueExpr) -> Result<SubValue, EvalError> {
        let left = self.eval_value(l)?;
        let right = self.eval_value(r)?;

        if left.is_null() || right.is_null() {
            return Err(EvalError::NullInOperation {
                operation: "division".to_string(),
                context: "cannot divide NULL values".to_string(),
            });
        }

        // Convert both to float for division
        let left_float = match left {
            SubValue::Integer(i) => i as f64,
            SubValue::Float(f) => f,
            _ => return Err(EvalError::TypeError {
                operation: "division".to_string(),
                expected: "numeric".to_string(),
                actual: left.type_name(),
                context: "left operand".to_string(),
            }),
        };

        let right_float = match right {
            SubValue::Integer(i) => i as f64,
            SubValue::Float(f) => f,
            _ => return Err(EvalError::TypeError {
                operation: "division".to_string(),
                expected: "numeric".to_string(),
                actual: right.type_name(),
                context: "right operand".to_string(),
            }),
        };

        if right_float == 0.0 {
            return Err(EvalError::DivisionByZero {
                expression: self.input.clone(),
            });
        }

        Ok(SubValue::Float(left_float / right_float))
    }

    /// Arithmetic modulo
    fn eval_arithmetic_modulo(&self, l: &ValueExpr, r: &ValueExpr) -> Result<SubValue, EvalError> {
        let left = self.eval_value(l)?;
        let right = self.eval_value(r)?;

        if left.is_null() || right.is_null() {
            return Err(EvalError::NullInOperation {
                operation: "modulo".to_string(),
                context: "cannot modulo NULL values".to_string(),
            });
        }

        match (&left, &right) {
            (SubValue::Integer(a), SubValue::Integer(b)) => {
                if *b == 0 {
                    return Err(EvalError::DivisionByZero {
                        expression: self.input.clone(),
                    });
                }
                Ok(SubValue::Integer(a % b))
            }
            (SubValue::Float(a), SubValue::Float(b)) => {
                if *b == 0.0 {
                    return Err(EvalError::DivisionByZero {
                        expression: self.input.clone(),
                    });
                }
                Ok(SubValue::Float(a % b))
            }
            (SubValue::Integer(a), SubValue::Float(b)) => {
                if *b == 0.0 {
                    return Err(EvalError::DivisionByZero {
                        expression: self.input.clone(),
                    });
                }
                Ok(SubValue::Float((*a as f64) % b))
            }
            (SubValue::Float(a), SubValue::Integer(b)) => {
                if *b == 0 {
                    return Err(EvalError::DivisionByZero {
                        expression: self.input.clone(),
                    });
                }
                Ok(SubValue::Float(a % (*b as f64)))
            }
            _ => Err(EvalError::TypeError {
                operation: "modulo".to_string(),
                expected: "numeric types".to_string(),
                actual: format!("{} and {}", left.type_name(), right.type_name()),
                context: "arithmetic operation".to_string(),
            }),
        }
    }

    // ========================================================================
    // HELPER FUNCTIONS
    // ========================================================================

    fn to_numeric(val: &SubValue) -> Result<f64, EvalError> {
        match val {
            SubValue::Integer(i) => Ok(*i as f64),
            SubValue::Float(f) => Ok(*f),
            _ => Err(EvalError::TypeError {
                operation: "numeric comparison".to_string(),
                expected: "numeric".to_string(),
                actual: val.type_name(),
                context: "operand".to_string(),
            }),
        }
    }

    /// Check if two types are compatible for IN operator
    /// (allows int/float mixing, but not string/numeric, etc.)
    fn are_types_compatible_for_in(left: &SubValue, right: &SubValue) -> bool {
        match (left, right) {
            // Exact matches
            (SubValue::Integer(_), SubValue::Integer(_)) => true,
            (SubValue::Float(_), SubValue::Float(_)) => true,
            (SubValue::String(_), SubValue::String(_)) => true,
            (SubValue::Boolean(_), SubValue::Boolean(_)) => true,
            (SubValue::Null, SubValue::Null) => true,
            // Numeric type mixing is allowed
            (SubValue::Integer(_), SubValue::Float(_)) => true,
            (SubValue::Float(_), SubValue::Integer(_)) => true,
            // Everything else is incompatible
            _ => false,
        }
    }

    fn runtime_type_name(rv: &RuntimeValue) -> String {
        match rv {
            RuntimeValue::Integer(_) => "integer".to_string(),
            RuntimeValue::Float(_) => "float".to_string(),
            RuntimeValue::String(_) => "string".to_string(),
            RuntimeValue::Boolean(_) => "boolean".to_string(),
            RuntimeValue::Null => "NULL".to_string(),
        }
    }
}

// ============================================================================
// INTERNAL TYPES
// ============================================================================

/// Substituted values - what AST nodes become after variable substitution
#[derive(Debug, Clone, PartialEq)]
enum SubValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl SubValue {
    /// Convert from RuntimeValue
    fn from_runtime(rv: &RuntimeValue) -> Self {
        match rv {
            RuntimeValue::Integer(i) => SubValue::Integer(*i),
            RuntimeValue::Float(f) => SubValue::Float(*f),
            RuntimeValue::String(s) => SubValue::String(s.clone()),
            RuntimeValue::Boolean(b) => SubValue::Boolean(*b),
            RuntimeValue::Null => SubValue::Null,
        }
    }

    /// Convert from ValueLiteral
    fn from_literal(lit: &ValueLiteral) -> Self {
        match lit {
            ValueLiteral::Integer(i) => SubValue::Integer(*i),
            ValueLiteral::Float(f) => SubValue::Float(*f),
            ValueLiteral::String(s) => SubValue::String(s.clone()),
            ValueLiteral::Boolean(b) => SubValue::Boolean(*b),
            ValueLiteral::Null => SubValue::Null,
        }
    }

    fn type_name(&self) -> String {
        match self {
            SubValue::Integer(_) => "integer".to_string(),
            SubValue::Float(_) => "float".to_string(),
            SubValue::String(_) => "string".to_string(),
            SubValue::Boolean(_) => "boolean".to_string(),
            SubValue::Null => "NULL".to_string(),
        }
    }

    fn is_null(&self) -> bool {
        matches!(self, SubValue::Null)
    }
}


