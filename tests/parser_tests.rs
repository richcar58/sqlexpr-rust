// Comprehensive test suite for SQL Expression Parser
//
// This test suite exercises all language features defined in the EBNF grammar:
// - Boolean operators (AND, OR, NOT)
// - Relational operators (=, <>, !=, >, >=, <, <=)
// - LIKE and NOT LIKE (with and without ESCAPE)
// - BETWEEN and NOT BETWEEN
// - IN and NOT IN
// - IS NULL and IS NOT NULL
// - Arithmetic expressions (+, -, *, /, %)
// - Unary operators (+, -)
// - All literal types
// - Comments (line and block)
// - Variables/identifiers

use sqlexpr_rust::{parse, BooleanExpr, RelationalExpr, ValueExpr, ValueLiteral, EqualityOp, ComparisonOp};

// ============================================================================
// BOOLEAN OPERATORS
// ============================================================================

#[test]
fn test_boolean_literal_true() {
    let result = parse("TRUE");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Literal(true) => (),
        _ => panic!("Expected TRUE literal"),
    }
}

#[test]
fn test_boolean_literal_false() {
    let result = parse("FALSE");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Literal(false) => (),
        _ => panic!("Expected FALSE literal"),
    }
}

#[test]
fn test_boolean_variable() {
    let result = parse("is_active");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Variable(name) => assert_eq!(name, "is_active"),
        _ => panic!("Expected variable"),
    }
}

#[test]
fn test_and_operator() {
    let result = parse("x > 5 AND y < 10");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::And(_, _) => (),
        _ => panic!("Expected AND expression"),
    }
}

#[test]
fn test_or_operator() {
    let result = parse("x > 5 OR y < 10");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Or(_, _) => (),
        _ => panic!("Expected OR expression"),
    }
}

#[test]
fn test_not_operator() {
    let result = parse("NOT x > 5");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Not(_) => (),
        _ => panic!("Expected NOT expression"),
    }
}

#[test]
fn test_complex_boolean_expression() {
    let result = parse("(x > 5 AND y < 10) OR (z = 20 AND NOT w >= 100)");
    assert!(result.is_ok());
}

#[test]
fn test_and_or_precedence() {
    // AND should bind tighter than OR
    let result = parse("a = 1 OR b = 2 AND c = 3");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Or(_, right) => {
            // Right side should be AND
            match *right {
                BooleanExpr::And(_, _) => (),
                _ => panic!("Expected AND on right side of OR"),
            }
        }
        _ => panic!("Expected OR at top level"),
    }
}

#[test]
fn test_parenthesized_boolean() {
    let result = parse("(x > 5)");
    assert!(result.is_ok());
}

// Enhancement 2: Boolean operators with string operands
#[test]
fn test_and_with_string_comparisons() {
    let result = parse("name = 'John' AND city = 'Boston'");
    assert!(result.is_ok());
}

#[test]
fn test_or_with_string_comparisons() {
    let result = parse("status = 'active' OR status = 'pending' OR status = 'verified'");
    assert!(result.is_ok());
}

#[test]
fn test_not_with_string_comparison() {
    let result = parse("NOT (name = 'Admin')");
    assert!(result.is_ok());
}

#[test]
fn test_complex_boolean_with_strings() {
    let result = parse("(first_name = 'John' AND last_name = 'Doe') OR (email = 'john@example.com')");
    assert!(result.is_ok());
}

#[test]
fn test_boolean_with_string_inequality() {
    let result = parse("username <> 'guest' AND password <> ''");
    assert!(result.is_ok());
}

#[test]
fn test_and_or_not_with_strings() {
    let result = parse("(category = 'electronics' OR category = 'computers') AND NOT brand = 'unknown'");
    assert!(result.is_ok());
}

// ============================================================================
// RELATIONAL OPERATORS
// ============================================================================

#[test]
fn test_equal_operator() {
    let result = parse("x = 5");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Equality { op: EqualityOp::Equal, .. }) => (),
        _ => panic!("Expected equality expression"),
    }
}

#[test]
fn test_not_equal_operator_angle_brackets() {
    let result = parse("x <> 5");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Equality { op: EqualityOp::NotEqual, .. }) => (),
        _ => panic!("Expected not-equal expression"),
    }
}

#[test]
fn test_not_equal_operator_exclamation() {
    let result = parse("x != 5");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Equality { op: EqualityOp::NotEqual, .. }) => (),
        _ => panic!("Expected not-equal expression"),
    }
}

#[test]
fn test_greater_than() {
    let result = parse("x > 5");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Comparison { op: ComparisonOp::GreaterThan, .. }) => (),
        _ => panic!("Expected greater-than expression"),
    }
}

#[test]
fn test_greater_or_equal() {
    let result = parse("x >= 5");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Comparison { op: ComparisonOp::GreaterOrEqual, .. }) => (),
        _ => panic!("Expected greater-or-equal expression"),
    }
}

#[test]
fn test_less_than() {
    let result = parse("x < 5");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Comparison { op: ComparisonOp::LessThan, .. }) => (),
        _ => panic!("Expected less-than expression"),
    }
}

#[test]
fn test_less_or_equal() {
    let result = parse("x <= 5");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Comparison { op: ComparisonOp::LessOrEqual, .. }) => (),
        _ => panic!("Expected less-or-equal expression"),
    }
}

// Enhancement 3: Relational operators with string operands
#[test]
fn test_string_equality() {
    let result = parse("name = 'Alice'");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Equality { op: EqualityOp::Equal, .. }) => (),
        _ => panic!("Expected equality expression"),
    }
}

#[test]
fn test_string_inequality_not_equal() {
    let result = parse("status <> 'deleted'");
    assert!(result.is_ok());
}

#[test]
fn test_string_inequality_exclamation() {
    let result = parse("username != 'admin'");
    assert!(result.is_ok());
}

#[test]
fn test_string_greater_than() {
    let result = parse("name > 'M'");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Comparison { op: ComparisonOp::GreaterThan, .. }) => (),
        _ => panic!("Expected greater-than expression"),
    }
}

#[test]
fn test_string_greater_or_equal() {
    let result = parse("city >= 'Boston'");
    assert!(result.is_ok());
}

#[test]
fn test_string_less_than() {
    let result = parse("code < 'ZZZ'");
    assert!(result.is_ok());
}

#[test]
fn test_string_less_or_equal() {
    let result = parse("country <= 'USA'");
    assert!(result.is_ok());
}

#[test]
fn test_empty_string_comparison() {
    let result = parse("description <> ''");
    assert!(result.is_ok());
}

#[test]
fn test_string_with_special_chars() {
    let result = parse("email = 'user@example.com'");
    assert!(result.is_ok());
}

#[test]
fn test_string_comparison_with_spaces() {
    let result = parse("full_name = 'John Smith'");
    assert!(result.is_ok());
}

#[test]
fn test_string_comparison_with_numbers() {
    let result = parse("code >= '12345'");
    assert!(result.is_ok());
}

// ============================================================================
// LIKE OPERATOR
// ============================================================================

#[test]
fn test_like_operator() {
    let result = parse("name LIKE '%test%'");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Like { pattern, negated: false, escape: None, .. }) => {
            assert_eq!(pattern, "%test%");
        }
        _ => panic!("Expected LIKE expression"),
    }
}

#[test]
fn test_like_with_escape() {
    let result = parse("name LIKE '%test\\%%' ESCAPE '\\'");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Like { pattern, escape: Some(esc), negated: false, .. }) => {
            assert_eq!(pattern, "%test\\%%");
            assert_eq!(esc, "\\");
        }
        _ => panic!("Expected LIKE with ESCAPE"),
    }
}

#[test]
fn test_not_like_operator() {
    let result = parse("name NOT LIKE '%test%'");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Like { pattern, negated: true, escape: None, .. }) => {
            assert_eq!(pattern, "%test%");
        }
        _ => panic!("Expected NOT LIKE expression"),
    }
}

#[test]
fn test_not_like_with_escape() {
    let result = parse("name NOT LIKE '%test\\%%' ESCAPE '\\'");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Like { pattern, escape: Some(esc), negated: true, .. }) => {
            assert_eq!(pattern, "%test\\%%");
            assert_eq!(esc, "\\");
        }
        _ => panic!("Expected NOT LIKE with ESCAPE"),
    }
}

#[test]
fn test_like_case_insensitive_keyword() {
    let result = parse("name LiKe '%test%'");
    assert!(result.is_ok());
}

// Enhancement 4: LIKE/NOT LIKE with multi-character wildcards (%)
#[test]
fn test_like_leading_multichar_wildcard() {
    let result = parse("filename LIKE '%@gmail.com'");
    assert!(result.is_ok());
}

#[test]
fn test_like_trailing_multichar_wildcard() {
    let result = parse("path LIKE '/home/user/%'");
    assert!(result.is_ok());
}

#[test]
fn test_like_embedded_multichar_wildcard() {
    let result = parse("description LIKE 'Product%Details'");
    assert!(result.is_ok());
}

#[test]
fn test_like_multiple_multichar_wildcards() {
    let result = parse("url LIKE 'http://%example.com/%/index.html'");
    assert!(result.is_ok());
}

#[test]
fn test_not_like_multichar_wildcard() {
    let result = parse("email NOT LIKE '%@spam.com'");
    assert!(result.is_ok());
}

#[test]
fn test_like_multichar_with_escape() {
    let result = parse("text LIKE '%50\\%%' ESCAPE '\\'");
    assert!(result.is_ok());
}

#[test]
fn test_not_like_multichar_with_escape() {
    let result = parse("code NOT LIKE 'TEST\\%USER%' ESCAPE '\\'");
    assert!(result.is_ok());
}

#[test]
fn test_like_only_multichar_wildcard() {
    let result = parse("anything LIKE '%'");
    assert!(result.is_ok());
}

#[test]
fn test_like_double_multichar_wildcards() {
    let result = parse("pattern LIKE '%%text%%'");
    assert!(result.is_ok());
}

// Enhancement 5: LIKE/NOT LIKE with single-character wildcards (_)
#[test]
fn test_like_single_char_wildcard() {
    let result = parse("code LIKE 'A_C'");
    assert!(result.is_ok());
}

#[test]
fn test_like_multiple_single_char_wildcards() {
    let result = parse("phone LIKE '___-___-____'");
    assert!(result.is_ok());
}

#[test]
fn test_like_leading_single_char_wildcard() {
    let result = parse("name LIKE '_ohn'");
    assert!(result.is_ok());
}

#[test]
fn test_like_trailing_single_char_wildcard() {
    let result = parse("word LIKE 'tes_'");
    assert!(result.is_ok());
}

#[test]
fn test_like_embedded_single_char_wildcards() {
    let result = parse("license LIKE 'AB_-_EF_-____'");
    assert!(result.is_ok());
}

#[test]
fn test_like_mixed_wildcards() {
    let result = parse("identifier LIKE '%_ID_%'");
    assert!(result.is_ok());
}

#[test]
fn test_not_like_single_char_wildcard() {
    let result = parse("status NOT LIKE '_nknown'");
    assert!(result.is_ok());
}

#[test]
fn test_like_single_char_with_escape() {
    let result = parse("text LIKE 'test\\_data' ESCAPE '\\'");
    assert!(result.is_ok());
}

#[test]
fn test_not_like_mixed_with_escape() {
    let result = parse("pattern NOT LIKE '%\\_%test%' ESCAPE '\\'");
    assert!(result.is_ok());
}

#[test]
fn test_like_complex_pattern() {
    let result = parse("filepath LIKE '/usr/__/bin/%/app_'");
    assert!(result.is_ok());
}

#[test]
fn test_like_zero_or_more_leading() {
    let result = parse("text LIKE '%suffix'");
    assert!(result.is_ok());
}

#[test]
fn test_like_zero_or_more_trailing() {
    let result = parse("text LIKE 'prefix%'");
    assert!(result.is_ok());
}

#[test]
fn test_like_zero_or_more_embedded() {
    let result = parse("text LIKE 'start%middle%end'");
    assert!(result.is_ok());
}

#[test]
fn test_not_like_complex_escape() {
    let result = parse("data NOT LIKE 'test\\%\\_value%' ESCAPE '\\'");
    assert!(result.is_ok());
}

// ============================================================================
// BETWEEN OPERATOR
// ============================================================================

#[test]
fn test_between_operator() {
    let result = parse("age BETWEEN 18 AND 65");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Between { negated: false, .. }) => (),
        _ => panic!("Expected BETWEEN expression"),
    }
}

#[test]
fn test_not_between_operator() {
    let result = parse("age NOT BETWEEN 18 AND 65");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Between { negated: true, .. }) => (),
        _ => panic!("Expected NOT BETWEEN expression"),
    }
}

#[test]
fn test_between_with_expressions() {
    let result = parse("(x + y) BETWEEN (a - 5) AND (b * 2)");
    assert!(result.is_ok());
}

#[test]
fn test_between_case_insensitive() {
    let result = parse("age BeTwEeN 18 aNd 65");
    assert!(result.is_ok());
}

// Enhancement 1: BETWEEN with string operands
#[test]
fn test_between_with_string_operands() {
    let result = parse("name BETWEEN 'Alice' AND 'Zeus'");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Between { negated: false, .. }) => (),
        _ => panic!("Expected BETWEEN expression"),
    }
}

#[test]
fn test_not_between_with_string_operands() {
    let result = parse("username NOT BETWEEN 'aaa' AND 'zzz'");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Between { negated: true, .. }) => (),
        _ => panic!("Expected NOT BETWEEN expression"),
    }
}

#[test]
fn test_between_with_mixed_string_cases() {
    let result = parse("city BETWEEN 'Boston' AND 'Seattle'");
    assert!(result.is_ok());
}

#[test]
fn test_between_with_date_strings() {
    let result = parse("date_str BETWEEN '2024-01-01' AND '2024-12-31'");
    assert!(result.is_ok());
}

#[test]
fn test_not_between_with_empty_strings() {
    let result = parse("code NOT BETWEEN '' AND 'A'");
    assert!(result.is_ok());
}

// ============================================================================
// IN OPERATOR
// ============================================================================

#[test]
fn test_in_operator_single_value() {
    let result = parse("status IN ('active')");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::In { values, negated: false, .. }) => {
            assert_eq!(values.len(), 1);
            match &values[0] {
                ValueLiteral::String(s) => assert_eq!(s, "active"),
                _ => panic!("Expected string literal"),
            }
        }
        _ => panic!("Expected IN expression"),
    }
}

#[test]
fn test_in_operator_multiple_values() {
    let result = parse("status IN ('active', 'pending', 'completed')");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::In { values, negated: false, .. }) => {
            assert_eq!(values.len(), 3);
            match (&values[0], &values[1], &values[2]) {
                (ValueLiteral::String(s1), ValueLiteral::String(s2), ValueLiteral::String(s3)) => {
                    assert_eq!(s1, "active");
                    assert_eq!(s2, "pending");
                    assert_eq!(s3, "completed");
                }
                _ => panic!("Expected string literals"),
            }
        }
        _ => panic!("Expected IN expression"),
    }
}

#[test]
fn test_not_in_operator() {
    let result = parse("status NOT IN ('inactive', 'deleted')");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::In { values, negated: true, .. }) => {
            assert_eq!(values.len(), 2);
        }
        _ => panic!("Expected NOT IN expression"),
    }
}

#[test]
fn test_in_case_insensitive() {
    let result = parse("status iN ('active')");
    assert!(result.is_ok());
}

// Enhancement 6: IN/NOT IN with numeric value sets
#[test]
fn test_in_with_integer_values() {
    let result = parse("age IN (18, 21, 25, 30)");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::In { values, negated: false, .. }) => {
            assert_eq!(values.len(), 4);
            match &values[0] {
                ValueLiteral::Integer(n) => assert_eq!(*n, 18),
                _ => panic!("Expected integer literal"),
            }
        }
        _ => panic!("Expected IN expression"),
    }
}

#[test]
fn test_not_in_with_integer_values() {
    let result = parse("error_code NOT IN (404, 500, 503)");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::In { values, negated: true, .. }) => {
            assert_eq!(values.len(), 3);
        }
        _ => panic!("Expected NOT IN expression"),
    }
}

#[test]
fn test_in_with_float_values() {
    let result = parse("temperature IN (98.6, 99.0, 100.4)");
    assert!(result.is_ok());
}

#[test]
fn test_in_with_hex_values() {
    let result = parse("flags IN (0x00, 0xFF, 0x1A)");
    assert!(result.is_ok());
}

#[test]
fn test_in_with_mixed_numeric_types() {
    let result = parse("value IN (10, 20.5, 0x1F, 100L)");
    assert!(result.is_ok());
}

#[test]
fn test_not_in_with_negative_integers() {
    let result = parse("balance NOT IN (-100, -50, -25, 0)");
    assert!(result.is_ok());
}

#[test]
fn test_in_with_scientific_notation() {
    let result = parse("measurement IN (1.5e-10, 2.5e-10, 3.5e-10)");
    assert!(result.is_ok());
}

#[test]
fn test_in_single_integer() {
    let result = parse("status_code IN (200)");
    assert!(result.is_ok());
}

#[test]
fn test_not_in_with_long_literals() {
    let result = parse("big_number NOT IN (1000000L, 2000000L, 3000000L)");
    assert!(result.is_ok());
}

#[test]
fn test_in_with_octal_values() {
    let result = parse("permissions IN (0644, 0755, 0777)");
    assert!(result.is_ok());
}

#[test]
fn test_in_mixed_strings_and_numbers_strings_only() {
    // Note: Since we enforce type consistency at parse level,
    // IN lists should contain same type values
    let result = parse("code IN ('A001', 'B002', 'C003')");
    assert!(result.is_ok());
}

#[test]
fn test_in_zero_values() {
    let result = parse("count IN (0, 0.0)");
    assert!(result.is_ok());
}

// ============================================================================
// IS NULL OPERATOR
// ============================================================================

#[test]
fn test_is_null() {
    let result = parse("value IS NULL");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::IsNull { negated: false, .. }) => (),
        _ => panic!("Expected IS NULL expression"),
    }
}

#[test]
fn test_is_not_null() {
    let result = parse("value IS NOT NULL");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::IsNull { negated: true, .. }) => (),
        _ => panic!("Expected IS NOT NULL expression"),
    }
}

#[test]
fn test_is_null_case_insensitive() {
    let result = parse("value Is NuLl");
    assert!(result.is_ok());
}

// ============================================================================
// ARITHMETIC EXPRESSIONS
// ============================================================================

#[test]
fn test_addition_in_comparison() {
    let result = parse("(a + b) > 10");
    assert!(result.is_ok());
    match result.unwrap() {
        BooleanExpr::Relational(RelationalExpr::Comparison { left, .. }) => {
            match left {
                ValueExpr::Add(_, _) => (),
                _ => panic!("Expected addition expression"),
            }
        }
        _ => panic!("Expected comparison"),
    }
}

#[test]
fn test_subtraction_in_comparison() {
    let result = parse("(a - b) > 10");
    assert!(result.is_ok());
}

#[test]
fn test_multiplication_in_comparison() {
    let result = parse("(a * b) > 10");
    assert!(result.is_ok());
}

#[test]
fn test_division_in_comparison() {
    let result = parse("(a / b) > 10");
    assert!(result.is_ok());
}

#[test]
fn test_modulo_in_comparison() {
    let result = parse("(a % b) = 0");
    assert!(result.is_ok());
}

#[test]
fn test_arithmetic_precedence() {
    // Multiplication should bind tighter than addition
    let result = parse("(a + b * c) = 10");
    assert!(result.is_ok());
}

#[test]
fn test_complex_arithmetic() {
    let result = parse("((a + b) * (c - d) / e) > 100");
    assert!(result.is_ok());
}

// ============================================================================
// UNARY OPERATORS
// ============================================================================

#[test]
fn test_unary_plus() {
    let result = parse("(+x) > 0");
    assert!(result.is_ok());
}

#[test]
fn test_unary_minus() {
    let result = parse("(-x) < 0");
    assert!(result.is_ok());
}

#[test]
fn test_double_unary_minus() {
    // Note: (--x) would be treated as a comment in SQL
    // So we use spaces: (- -x) to get double unary minus
    let result = parse("(- -x) = 5");
    assert!(result.is_ok());
}

// ============================================================================
// LITERALS
// ============================================================================

#[test]
fn test_integer_literal() {
    let result = parse("x = 42");
    assert!(result.is_ok());
}

#[test]
fn test_long_literal() {
    let result = parse("x = 42L");
    assert!(result.is_ok());
}

#[test]
fn test_long_literal_lowercase() {
    let result = parse("x = 42l");
    assert!(result.is_ok());
}

#[test]
fn test_hex_literal() {
    let result = parse("x = 0x1A");
    assert!(result.is_ok());
}

#[test]
fn test_hex_literal_lowercase() {
    let result = parse("x = 0x1a");
    assert!(result.is_ok());
}

#[test]
fn test_octal_literal() {
    let result = parse("x = 077");
    assert!(result.is_ok());
}

#[test]
fn test_float_literal_with_decimal() {
    let result = parse("x = 3.14");
    assert!(result.is_ok());
}

#[test]
fn test_float_literal_with_exponent() {
    let result = parse("x = 1e5");
    assert!(result.is_ok());
}

#[test]
fn test_float_literal_with_negative_exponent() {
    let result = parse("x = 1e-5");
    assert!(result.is_ok());
}

#[test]
fn test_float_literal_starting_with_dot() {
    let result = parse("x = .5");
    assert!(result.is_ok());
}

#[test]
fn test_float_literal_full() {
    let result = parse("x = 3.14e-2");
    assert!(result.is_ok());
}

#[test]
fn test_string_literal() {
    let result = parse("name = 'John'");
    assert!(result.is_ok());
}

#[test]
fn test_string_literal_with_escaped_quote() {
    let result = parse("name = 'It''s John'");
    assert!(result.is_ok());
}

#[test]
fn test_null_literal() {
    let result = parse("x = NULL");
    assert!(result.is_ok());
}

#[test]
fn test_boolean_literal_as_value() {
    let result = parse("x = TRUE");
    assert!(result.is_ok());
}

// ============================================================================
// IDENTIFIERS / VARIABLES
// ============================================================================

#[test]
fn test_identifier_with_underscore() {
    let result = parse("my_variable > 5");
    assert!(result.is_ok());
}

#[test]
fn test_identifier_with_dollar() {
    let result = parse("$variable > 5");
    assert!(result.is_ok());
}

#[test]
fn test_identifier_starting_with_underscore() {
    let result = parse("_variable > 5");
    assert!(result.is_ok());
}

#[test]
fn test_identifier_with_numbers() {
    let result = parse("var123 > 5");
    assert!(result.is_ok());
}

// ============================================================================
// COMMENTS
// ============================================================================

#[test]
fn test_line_comment() {
    let result = parse("x > 5 -- this is a comment\n");
    assert!(result.is_ok());
}

#[test]
fn test_line_comment_at_end() {
    let result = parse("x > 5 -- comment");
    assert!(result.is_ok());
}

#[test]
fn test_block_comment() {
    let result = parse("x /* comment */ > 5");
    assert!(result.is_ok());
}

#[test]
fn test_multiline_block_comment() {
    let result = parse("x /* this is\n a multiline\n comment */ > 5");
    assert!(result.is_ok());
}

#[test]
fn test_multiple_comments() {
    let result = parse("x > 5 /* c1 */ AND /* c2 */ y < 10 -- c3");
    assert!(result.is_ok());
}

// ============================================================================
// WHITESPACE
// ============================================================================

#[test]
fn test_whitespace_handling() {
    let result = parse("  x   >   5  ");
    assert!(result.is_ok());
}

#[test]
fn test_newlines_in_expression() {
    let result = parse("x > 5\nAND\ny < 10");
    assert!(result.is_ok());
}

#[test]
fn test_tabs_in_expression() {
    let result = parse("x\t>\t5");
    assert!(result.is_ok());
}

// ============================================================================
// COMPLEX REAL-WORLD EXAMPLES
// ============================================================================

#[test]
fn test_example_from_grammar_1() {
    let result = parse("x > 5");
    assert!(result.is_ok());
}

#[test]
fn test_example_from_grammar_2() {
    let result = parse("name = 'John' AND age >= 18");
    assert!(result.is_ok());
}

#[test]
fn test_example_from_grammar_3() {
    let result = parse("price BETWEEN 10 AND 100");
    assert!(result.is_ok());
}

#[test]
fn test_example_from_grammar_4() {
    let result = parse("status IN ('active', 'pending')");
    assert!(result.is_ok());
}

#[test]
fn test_example_from_grammar_5() {
    let result = parse("email LIKE '%@example.com'");
    assert!(result.is_ok());
}

#[test]
fn test_example_from_grammar_6() {
    let result = parse("value IS NOT NULL");
    assert!(result.is_ok());
}

#[test]
fn test_example_from_grammar_7() {
    let result = parse("(a + b) > (c - d)");
    assert!(result.is_ok());
}

#[test]
fn test_example_from_grammar_8() {
    let result = parse("TRUE");
    assert!(result.is_ok());
}

#[test]
fn test_example_from_grammar_9() {
    let result = parse("NOT (x = 5 OR y = 10)");
    assert!(result.is_ok());
}

#[test]
fn test_complex_real_world_1() {
    let result = parse(
        "(customer_age >= 18 AND customer_age <= 65) AND \
         (account_status IN ('active', 'pending', 'verified')) AND \
         (credit_score > 650 OR has_collateral = TRUE) AND \
         last_login IS NOT NULL"
    );
    assert!(result.is_ok());
}

#[test]
fn test_complex_real_world_2() {
    let result = parse(
        "(product_name LIKE '%laptop%' OR product_name LIKE '%computer%') AND \
         (price BETWEEN 500 AND 2000) AND \
         (stock_quantity > 0) AND \
         NOT (category IN ('refurbished', 'damaged'))"
    );
    assert!(result.is_ok());
}

#[test]
fn test_complex_real_world_3() {
    let result = parse(
        "((revenue - cost) / revenue * 100) >= 20 AND \
         (sales_count > 1000 OR premium_customer = TRUE) AND \
         region NOT IN ('excluded1', 'excluded2')"
    );
    assert!(result.is_ok());
}

// ============================================================================
// ERROR CASES (Should fail)
// ============================================================================

#[test]
fn test_reject_standalone_literal() {
    // Grammar should reject non-boolean top-level expressions
    let result = parse("42");
    assert!(result.is_err());
}

#[test]
fn test_reject_standalone_arithmetic() {
    let result = parse("1 + 2");
    assert!(result.is_err());
}

#[test]
fn test_reject_standalone_string() {
    let result = parse("'hello'");
    assert!(result.is_err());
}

#[test]
fn test_reject_parenthesized_arithmetic() {
    let result = parse("(a * b)");
    assert!(result.is_err());
}

#[test]
fn test_reject_unterminated_string() {
    let result = parse("x = 'unterminated");
    assert!(result.is_err());
}

#[test]
fn test_reject_unterminated_block_comment() {
    let result = parse("x > 5 /* unterminated comment");
    assert!(result.is_err());
}

#[test]
fn test_reject_invalid_operator() {
    let result = parse("x === 5");
    assert!(result.is_err());
}

#[test]
fn test_reject_missing_operand() {
    let result = parse("x >");
    assert!(result.is_err());
}

// ============================================================================
// PARENTHESIZATION
// ============================================================================

#[test]
fn test_parenthesized_comparison() {
    let result = parse("(x > 5)");
    assert!(result.is_ok());
}

#[test]
fn test_nested_parentheses() {
    let result = parse("((x > 5))");
    assert!(result.is_ok());
}

#[test]
fn test_parenthesized_value_expressions() {
    let result = parse("((a + b) * (c - d)) > ((e / f) + (g % h))");
    assert!(result.is_ok());
}

// ============================================================================
// CASE INSENSITIVITY
// ============================================================================

#[test]
fn test_keywords_case_insensitive() {
    let result = parse("x > 5 aNd y < 10 oR z = 20");
    assert!(result.is_ok());
}

#[test]
fn test_null_case_insensitive() {
    let result = parse("x = nULl");
    assert!(result.is_ok());
}

#[test]
fn test_true_false_case_insensitive() {
    let result = parse("tRuE aNd FaLsE");
    assert!(result.is_ok());
}
