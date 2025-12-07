# Parser Type Checking Enhancement Plan

## Overview

This plan details the implementation of enhanced type checking for BETWEEN and IN operators at parse time. The goal is to move type validation from runtime (evaluator) to compile/parse time for better error detection and simplified semantics.

## Current State Analysis

### BETWEEN Operator (Lines 479-489 and 453-463 in parser.rs)
- Currently accepts any `ValueExpr` for lower and upper bounds
- No type checking at parse time
- Allows variables, arithmetic expressions, and all literal types
- Type compatibility checked at evaluation time

### IN Operator (Lines 491-498 and 465-473 in parser.rs)
- Uses `parse_string_list()` which calls `expect_value_literal()` (lines 536-610)
- Currently accepts all `ValueLiteral` types: Integer, Float, String, Null, Boolean
- Type consistency checked at evaluation time in `Evaluator::are_same_type()` (evaluator.rs:827-836)
- Evaluator performs a separate pass over all values to check type consistency (evaluator.rs:479-492)

## Requirements

### 1. BETWEEN/NOT BETWEEN Type Checking

**Restrictions:**
- Lower and upper bounds must be **literals only** (not variables or expressions)
- Both bounds must be the same type category:
  - **Numeric**: Integer or Float (mixing Integer and Float is allowed)
  - **String**: String literals only
- No other literal types allowed (reject Null, Boolean)

**Error Messages:**
- When non-literal used: "BETWEEN bounds must be literal values, not variables or expressions"
- When Null used: "NULL is not allowed in BETWEEN bounds"
- When Boolean used: "Boolean literals are not allowed in BETWEEN bounds"
- When types don't match: "BETWEEN bounds must be both numeric or both string, found {lower_type} and {upper_type}"

### 2. IN/NOT IN Type Checking

**Restrictions:**
- All values must be **literals only** (not variables or expressions)
- Only Integer, Float, or String literals allowed
- Null and Boolean literals are **rejected**
- All values must be **exactly** the same ValueLiteral variant:
  - All Integer, OR
  - All Float, OR
  - All String
  - **No mixing** of Integer and Float (stricter than evaluator's current behavior)

**Error Messages:**
- When Null encountered: "NULL is not allowed in IN list"
- When Boolean encountered: "Boolean literals are not allowed in IN list"
- When types mixed: "IN list values must all be the same type, found {first_type} and {current_type}"

**Implementation Strategy:**
- Type check as values are being parsed and added to the vector
- Fail immediately upon detecting type mismatch
- No separate validation pass needed

### 3. Remove Evaluator Type Checking

**Changes to evaluator.rs:**
- Remove or simplify `Evaluator::are_same_type()` (lines 827-836)
- Remove the type consistency loop in IN evaluation (lines 479-492)
- Add documentation comments explaining that type checking is now done at parse time
- The evaluator can assume all IN lists are type-consistent

## Implementation Plan

### Phase 1: Parser Enhancements (src/parser.rs)

#### Step 1.1: Add Helper Functions

Add new helper methods to the `Parser` struct:

```rust
/// Extract literal from ValueExpr, returning error if not a literal
fn extract_literal(expr: &ValueExpr) -> ParseResult<&ValueLiteral> {
    match expr {
        ValueExpr::Literal(lit) => Ok(lit),
        ValueExpr::Variable(_) => Err(ParseError {
            message: "Variables are not allowed here, only literal values".to_string(),
        }),
        _ => Err(ParseError {
            message: "Complex expressions are not allowed here, only literal values".to_string(),
        }),
    }
}

/// Check if literal is numeric (Integer or Float)
fn is_numeric_literal(lit: &ValueLiteral) -> bool {
    matches!(lit, ValueLiteral::Integer(_) | ValueLiteral::Float(_))
}

/// Get literal type name for error messages
fn literal_type_name(lit: &ValueLiteral) -> &'static str {
    match lit {
        ValueLiteral::Integer(_) => "integer",
        ValueLiteral::Float(_) => "float",
        ValueLiteral::String(_) => "string",
        ValueLiteral::Null => "NULL",
        ValueLiteral::Boolean(_) => "boolean",
    }
}

/// Check if two literals are type-compatible for BETWEEN
fn are_between_compatible(lower: &ValueLiteral, upper: &ValueLiteral) -> bool {
    match (lower, upper) {
        // Both numeric
        (ValueLiteral::Integer(_), ValueLiteral::Integer(_)) => true,
        (ValueLiteral::Integer(_), ValueLiteral::Float(_)) => true,
        (ValueLiteral::Float(_), ValueLiteral::Integer(_)) => true,
        (ValueLiteral::Float(_), ValueLiteral::Float(_)) => true,
        // Both string
        (ValueLiteral::String(_), ValueLiteral::String(_)) => true,
        // Everything else incompatible
        _ => false,
    }
}

/// Validate literal for IN list (reject Null and Boolean)
fn validate_in_literal(lit: &ValueLiteral, position: usize, input: &str) -> ParseResult<()> {
    match lit {
        ValueLiteral::Null => Err(ParseError {
            message: format!(
                "NULL is not allowed in IN list near position {} in:\n  {}",
                position, input
            ),
        }),
        ValueLiteral::Boolean(_) => Err(ParseError {
            message: format!(
                "Boolean literals are not allowed in IN list near position {} in:\n  {}",
                position, input
            ),
        }),
        ValueLiteral::Integer(_) | ValueLiteral::Float(_) | ValueLiteral::String(_) => Ok(()),
    }
}

/// Check if two literals are exactly the same type (for IN list)
fn are_exact_same_type(a: &ValueLiteral, b: &ValueLiteral) -> bool {
    match (a, b) {
        (ValueLiteral::Integer(_), ValueLiteral::Integer(_)) => true,
        (ValueLiteral::Float(_), ValueLiteral::Float(_)) => true,
        (ValueLiteral::String(_), ValueLiteral::String(_)) => true,
        _ => false,
    }
}
```

#### Step 1.2: Update BETWEEN Parsing

Modify the BETWEEN parsing logic (lines 479-489 and 453-463):

**For regular BETWEEN (lines 479-489):**

```rust
Token::Between => {
    self.advance();
    let lower_expr = self.parse_value_expression()?;
    self.expect(Token::And)?;
    let upper_expr = self.parse_value_expression()?;

    // Extract literals from expressions
    let lower_lit = Self::extract_literal(&lower_expr)?;
    let upper_lit = Self::extract_literal(&upper_expr)?;

    // Reject NULL
    if matches!(lower_lit, ValueLiteral::Null) {
        return Err(ParseError {
            message: format!(
                "NULL is not allowed as lower bound in BETWEEN near position {} in:\n  {}",
                self.position, self.input
            ),
        });
    }
    if matches!(upper_lit, ValueLiteral::Null) {
        return Err(ParseError {
            message: format!(
                "NULL is not allowed as upper bound in BETWEEN near position {} in:\n  {}",
                self.position, self.input
            ),
        });
    }

    // Reject Boolean
    if matches!(lower_lit, ValueLiteral::Boolean(_)) {
        return Err(ParseError {
            message: format!(
                "Boolean literals are not allowed as lower bound in BETWEEN near position {} in:\n  {}",
                self.position, self.input
            ),
        });
    }
    if matches!(upper_lit, ValueLiteral::Boolean(_)) {
        return Err(ParseError {
            message: format!(
                "Boolean literals are not allowed as upper bound in BETWEEN near position {} in:\n  {}",
                self.position, self.input
            ),
        });
    }

    // Check type compatibility
    if !Self::are_between_compatible(lower_lit, upper_lit) {
        return Err(ParseError {
            message: format!(
                "BETWEEN bounds must be both numeric or both string, found {} and {} near position {} in:\n  {}",
                Self::literal_type_name(lower_lit),
                Self::literal_type_name(upper_lit),
                self.position,
                self.input
            ),
        });
    }

    Ok(RelationalExpr::Between {
        expr: left,
        lower: lower_expr,
        upper: upper_expr,
        negated: false,
    })
}
```

**Similar changes for NOT BETWEEN (lines 453-463).**

#### Step 1.3: Update IN Parsing

Modify `parse_string_list()` to perform incremental type checking (lines 536-548):

```rust
/// Parse value literal list for IN operator with strict type checking
/// All values must be the same exact type (Integer, Float, or String)
/// NULL and Boolean are rejected
fn parse_string_list(&mut self) -> ParseResult<Vec<ValueLiteral>> {
    self.expect(Token::LeftParen)?;

    let first = self.expect_value_literal()?;

    // Validate first literal
    self.validate_in_literal(&first, self.position, &self.input)?;

    let mut values = vec![first.clone()];

    while self.current_token() == &Token::Comma {
        self.advance();
        let next = self.expect_value_literal()?;

        // Validate this literal
        self.validate_in_literal(&next, self.position, &self.input)?;

        // Check type consistency with first value
        if !Self::are_exact_same_type(&first, &next) {
            return Err(ParseError {
                message: format!(
                    "IN list values must all be the same type, found {} and {} near position {} in:\n  {}",
                    Self::literal_type_name(&first),
                    Self::literal_type_name(&next),
                    self.position,
                    self.input
                ),
            });
        }

        values.push(next);
    }

    self.expect(Token::RightParen)?;
    Ok(values)
}
```

### Phase 2: Update Evaluator (src/evaluator.rs)

#### Step 2.1: Simplify IN Evaluation

Update the `eval_in()` method (lines 460-518):

Remove the type consistency check loop (lines 478-492) since the parser now guarantees type consistency.

Add a comment explaining the change:

```rust
fn eval_in(&self, expr: &ValueExpr, values: &[ValueLiteral], negated: bool) -> Result<bool, EvalError> {
    let val = self.eval_value(expr)?;

    // Type consistency of the values list is now guaranteed by the parser,
    // so we only need to check if the left operand is type-compatible
    if !values.is_empty() {
        let first_list_val = SubValue::from_literal(&values[0]);

        if !Self::are_compatible(&val, &first_list_val) {
            return Err(EvalError::TypeError {
                operation: "IN".to_string(),
                expected: first_list_val.type_name(),
                actual: val.type_name(),
                context: "left operand type doesn't match list element types".to_string(),
            });
        }

        // Note: Type consistency check removed - now done at parse time
    }

    // ... rest of the method unchanged
}
```

#### Step 2.2: Document Changes

Add documentation to `are_same_type()` explaining its reduced role:

```rust
/// Check if two values have the same type
/// Note: This is now primarily used for debugging. The parser enforces
/// type consistency for IN lists at parse time.
fn are_same_type(a: &SubValue, b: &SubValue) -> bool {
    // ... existing implementation
}
```

### Phase 3: Comprehensive Test Suite

Create new test file: `tests/parser_type_checking_tests.rs`

#### Test Categories

**A. BETWEEN - Positive Tests (Should Pass)**

1. `test_between_both_integers` - `x BETWEEN 1 AND 10`
2. `test_between_both_floats` - `x BETWEEN 1.5 AND 10.5`
3. `test_between_mixed_numeric_int_float` - `x BETWEEN 1 AND 10.5`
4. `test_between_mixed_numeric_float_int` - `x BETWEEN 1.5 AND 10`
5. `test_between_both_strings` - `name BETWEEN 'Alice' AND 'Zeus'`
6. `test_between_negative_numbers` - `temp BETWEEN -10 AND -5`
7. `test_between_negative_and_positive` - `balance BETWEEN -100 AND 100`
8. `test_between_negative_floats` - `value BETWEEN -3.14 AND -1.5`
9. `test_between_hex_literals` - `flags BETWEEN 0x00 AND 0xFF`
10. `test_between_octal_literals` - `perms BETWEEN 0644 AND 0777`
11. `test_between_scientific_notation` - `measure BETWEEN 1.5e-10 AND 2.5e-10`
12. `test_between_long_literals` - `id BETWEEN 1000000L AND 9999999L`
13. `test_not_between_integers` - `x NOT BETWEEN 1 AND 10`
14. `test_not_between_floats` - `score NOT BETWEEN 0.0 AND 59.9`
15. `test_not_between_strings` - `code NOT BETWEEN 'A' AND 'M'`

**B. BETWEEN - Negative Tests (Should Fail)**

16. `test_between_null_lower` - `x BETWEEN NULL AND 10` ❌
17. `test_between_null_upper` - `x BETWEEN 1 AND NULL` ❌
18. `test_between_both_null` - `x BETWEEN NULL AND NULL` ❌
19. `test_between_boolean_lower` - `x BETWEEN TRUE AND 10` ❌
20. `test_between_boolean_upper` - `x BETWEEN 1 AND FALSE` ❌
21. `test_between_both_boolean` - `x BETWEEN TRUE AND FALSE` ❌
22. `test_between_string_and_int` - `x BETWEEN 'hello' AND 10` ❌
23. `test_between_int_and_string` - `x BETWEEN 1 AND 'world'` ❌
24. `test_between_string_and_float` - `x BETWEEN 'test' AND 3.14` ❌
25. `test_between_float_and_string` - `x BETWEEN 2.71 AND 'value'` ❌
26. `test_between_variable_lower` - `x BETWEEN y AND 10` ❌
27. `test_between_variable_upper` - `x BETWEEN 1 AND y` ❌
28. `test_between_expression_lower` - `x BETWEEN (y + 5) AND 10` ❌
29. `test_between_expression_upper` - `x BETWEEN 1 AND (y * 2)` ❌
30. `test_not_between_null_lower` - `x NOT BETWEEN NULL AND 10` ❌
31. `test_not_between_type_mismatch` - `x NOT BETWEEN 'A' AND 100` ❌

**C. IN - Positive Tests (Should Pass)**

32. `test_in_all_integers` - `x IN (1, 2, 3, 4, 5)`
33. `test_in_all_floats` - `score IN (1.5, 2.5, 3.5)`
34. `test_in_all_strings` - `status IN ('active', 'pending', 'completed')`
35. `test_in_single_integer` - `code IN (42)`
36. `test_in_single_float` - `value IN (3.14)`
37. `test_in_single_string` - `state IN ('running')`
38. `test_in_negative_integers` - `temp IN (-10, -5, 0, 5, 10)`
39. `test_in_negative_floats` - `balance IN (-100.5, -50.25, 0.0)`
40. `test_in_hex_integers` - `flags IN (0x00, 0x0F, 0xFF)`
41. `test_in_octal_integers` - `perms IN (0644, 0755, 0777)`
42. `test_in_long_integers` - `id IN (1000000L, 2000000L, 3000000L)`
43. `test_in_scientific_floats` - `measure IN (1.5e-10, 2.5e-10, 3.5e-10)`
44. `test_in_many_values` - `x IN (1, 2, 3, 4, 5, 6, 7, 8, 9, 10)` (test with 10+ values)
45. `test_not_in_integers` - `x NOT IN (1, 2, 3)`
46. `test_not_in_floats` - `score NOT IN (0.0, 0.5, 1.0)`
47. `test_not_in_strings` - `role NOT IN ('admin', 'root')`

**D. IN - Negative Tests (Should Fail)**

48. `test_in_with_null` - `x IN (1, 2, NULL, 3)` ❌
49. `test_in_with_null_first` - `x IN (NULL, 1, 2)` ❌
50. `test_in_with_null_only` - `x IN (NULL)` ❌
51. `test_in_with_boolean` - `x IN (1, 2, TRUE, 3)` ❌
52. `test_in_with_boolean_first` - `x IN (FALSE, 1, 2)` ❌
53. `test_in_with_boolean_only` - `x IN (TRUE)` ❌
54. `test_in_mixed_int_float` - `x IN (1, 2.5, 3)` ❌ (stricter than before!)
55. `test_in_mixed_float_int` - `x IN (1.5, 2, 3.5)` ❌
56. `test_in_mixed_int_string` - `x IN (1, 'hello', 3)` ❌
57. `test_in_mixed_string_int` - `x IN ('a', 1, 'b')` ❌
58. `test_in_mixed_float_string` - `x IN (1.5, 'test', 2.5)` ❌
59. `test_in_mixed_string_float` - `x IN ('x', 1.5, 'y')` ❌
60. `test_in_mixed_hex_decimal` - `x IN (0xFF, 255, 0x10)` ❌ (both Integer but different syntax)
   - **Note:** This should actually PASS since both 0xFF and 255 are Integer literals
61. `test_in_mixed_long_int` - `x IN (1000000L, 1000000, 2000000L)` ❌
   - **Correction:** This should PASS - both are Integer type
62. `test_not_in_with_null` - `x NOT IN (1, NULL, 3)` ❌
63. `test_not_in_mixed_types` - `x NOT IN (1, 2.5, 'hello')` ❌

**Note on tests 60-61:** After review, both hex/decimal and long/regular integers are the same ValueLiteral::Integer type, so they should be allowed to mix. Updated test expectations.

**Corrected D. IN - Negative Tests:**

60. `test_in_mixed_types_all_three` - `x IN (1, 2.5, 'hello')` ❌
61. `test_in_alternating_types` - `x IN ('a', 1, 'b', 2)` ❌

**E. Edge Cases and Complex Scenarios**

62. `test_between_empty_strings` - `code BETWEEN '' AND 'Z'`
63. `test_between_same_value` - `x BETWEEN 5 AND 5`
64. `test_between_reverse_order` - `x BETWEEN 10 AND 1` (parser allows, evaluator handles)
65. `test_in_empty_list_error` - `x IN ()` ❌ (should fail - empty list)
66. `test_in_duplicate_values` - `x IN (1, 2, 1, 3)` (allowed)
67. `test_complex_between_in_expression` - `(x BETWEEN 1 AND 10) AND (status IN ('a', 'b'))`
68. `test_nested_not_between_not_in` - `NOT (x BETWEEN 1 AND 5) OR NOT (y IN (10, 20))`

**F. Error Message Quality Tests**

These tests verify that error messages are clear and informative:

69. `test_between_null_error_message` - Verify error mentions "NULL not allowed"
70. `test_between_type_mismatch_error_message` - Verify error shows both types
71. `test_in_null_error_message` - Verify error mentions "NULL not allowed in IN list"
72. `test_in_boolean_error_message` - Verify error mentions "Boolean not allowed in IN list"
73. `test_in_type_mismatch_error_message` - Verify error shows first type and mismatched type

### Phase 4: Update Existing Tests

Review and update existing tests in `tests/parser_tests.rs` and `tests/evaluator_tests.rs`:

1. **Parser tests** - Should continue to pass (no breaking changes for valid syntax)
2. **Evaluator tests** - May need updates:
   - Tests using `IN` with mixed Integer/Float may now fail at parse time instead of eval time
   - Update these tests or move them to parser tests as negative cases

### Phase 5: Documentation Updates

Update the following files:

1. **CLAUDE.md** - Add section on type checking rules for BETWEEN and IN
2. **README.md** (if exists) - Document the stricter type checking
3. **SqlExprParser-EBNF-Final.ebnf** - Add comments about type restrictions
4. **docs/command_prompts.md** - Mark this task as completed

## Test Implementation Details

### Positive Test Pattern

```rust
#[test]
fn test_between_both_integers() {
    let result = parse("x BETWEEN 1 AND 10");
    if let Err(e) = &result {
        eprintln!("Parse error: {}", e);
    }
    assert!(result.is_ok(), "Expected BETWEEN with integer bounds");
}
```

### Negative Test Pattern

```rust
#[test]
fn test_between_null_lower() {
    let result = parse("x BETWEEN NULL AND 10");
    if let Ok(r) = &result {
        eprintln!("Expected error but found success: {}", r);
    }
    assert!(result.is_err(), "Expected error for NULL in BETWEEN lower bound");
    if let Err(e) = result {
        let msg = e.message;
        assert!(
            msg.contains("NULL") && msg.contains("not allowed"),
            "Error message should mention NULL is not allowed, got: {}",
            msg
        );
    }
}
```

### Error Message Verification Pattern

```rust
#[test]
fn test_in_type_mismatch_error_message() {
    let result = parse("x IN (1, 2.5, 3)");
    assert!(result.is_err());
    if let Err(e) = result {
        let msg = e.message;
        assert!(
            msg.contains("integer") && msg.contains("float"),
            "Error should mention both types: {}",
            msg
        );
        assert!(
            msg.contains("must all be the same type"),
            "Error should explain requirement: {}",
            msg
        );
    }
}
```

## Implementation Checklist

- [ ] Phase 1.1: Add helper functions to Parser
- [ ] Phase 1.2: Update BETWEEN parsing (regular and NOT)
- [ ] Phase 1.3: Update IN parsing with incremental type checking
- [ ] Phase 2.1: Simplify evaluator IN type checking
- [ ] Phase 2.2: Add documentation to evaluator
- [ ] Phase 3: Implement comprehensive test suite (73 tests)
  - [ ] BETWEEN positive tests (15)
  - [ ] BETWEEN negative tests (16)
  - [ ] IN positive tests (16)
  - [ ] IN negative tests (13)
  - [ ] Edge cases (7)
  - [ ] Error message tests (6)
- [ ] Phase 4: Review and update existing tests
- [ ] Phase 5: Update documentation
- [ ] Final: Run all tests and verify no regressions

## Expected Test Count

- **New tests:** 73
- **Existing tests:** 155 (parser) + 114 (evaluator) = 269
- **Total after implementation:** 342 tests

## Breaking Changes

⚠️ **Important:** This is a **breaking change** for code that currently relies on:

1. **IN with mixed Integer/Float:** Previously allowed at parse time, checked at eval time
   - `x IN (1, 2.5, 3)` - **Now rejected at parse time**
2. **BETWEEN with variables/expressions:** Previously allowed
   - `x BETWEEN y AND z` - **Now rejected at parse time**
   - `x BETWEEN (a + 5) AND 10` - **Now rejected at parse time**

These changes improve parse-time error detection but may break existing code.

## Success Criteria

1. ✅ All 73 new tests pass
2. ✅ All existing tests continue to pass (with documented exceptions)
3. ✅ Error messages are clear and informative
4. ✅ No performance regression
5. ✅ Documentation is complete and accurate
6. ✅ Code is well-commented explaining the type checking rules

## Timeline

This is a moderate-sized enhancement that should take:
- Implementation: Focus on correctness and thorough testing
- Testing: Most time will be spent on the comprehensive test suite
- Documentation: Update all relevant docs

The work breaks down into clear phases that can be completed incrementally.
