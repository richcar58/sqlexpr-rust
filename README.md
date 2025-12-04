# SQL Expression Parser & Evaluator

A Rust library for parsing and evaluating SQL-like boolean expressions with full support for comparisons, arithmetic, pattern matching, and logical operators.

There are two Java SqlExpr parser/evaluator implementations that accept basically the same language as this Rust parser.  See [sqlexpr-javacc](https://github.com/richcar58/sqlexpr-javacc) for a parser built using the [JavaCC](https://javacc.github.io/javacc/) parser generator; see [sqlexpr-congocc](https://github.com/richcar58/sqlexpr-congocc) for a parser built using the [CongoCC](https://parsers.org/) parser generator.

## Features

### Parser
- **Grammar-enforced type safety**: All top-level expressions must be boolean-valued
- **Comprehensive operators**:
  - Logical: `AND`, `OR`, `NOT`
  - Comparison: `>`, `>=`, `<`, `<=`, `=`, `<>`, `!=`
  - Pattern matching: `LIKE`, `NOT LIKE` (with `%`, `_` wildcards and `ESCAPE`)
  - Range: `BETWEEN`, `NOT BETWEEN`
  - Membership: `IN`, `NOT IN`
  - Null testing: `IS NULL`, `IS NOT NULL`
  - Arithmetic: `+`, `-`, `*`, `/`, `%` (modulo)
  - Unary: `+`, `-`
- **Rich literals**:
  - Integers: decimal (`42`), hexadecimal (`0xFF`), octal (`0755`)
  - Floats: standard (`3.14`), scientific notation (`1.5e-10`)
  - Strings: single-quoted with escape sequences (`'hello\'world'`)
  - Booleans: `TRUE`, `FALSE`
  - Null: `NULL`
- **Comments**: Line comments (`--`) and block comments (`/* */`)
- **Case-insensitive keywords**: `AND`, `and`, `And` all work
- **Detailed error messages**: Parse errors include position and context

### Evaluator
- **Variable substitution**: Bind runtime values to variables
- **Type system**: Integer, Float, String, Boolean, Null
- **Automatic type coercion**: Mixed int/float arithmetic automatically promotes to float
- **Division semantics**: Always returns float (e.g., `7/2 = 3.5`)
- **Null handling**: NULL disallowed in arithmetic/comparisons, only allowed with `IS NULL`
- **Short-circuit evaluation**: `AND` and `OR` operators evaluate efficiently
- **Pattern matching**: Full LIKE implementation with wildcards and escape sequences
- **Comprehensive error reporting**: Type errors, null violations, division by zero, etc.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
sqlexpr-rust = "0.1.0"
```

### Parsing Expressions

```rust
use sqlexpr_rust::parse;

fn main() {
    // Parse a simple comparison
    let ast = parse("age >= 18 AND status = 'active'").unwrap();
    println!("Parsed: {}", ast);

    // Parse complex expressions
    let expr = parse("(price * quantity) > 1000 AND customer_type IN ('gold', 'platinum')").unwrap();

    // Parse with LIKE pattern matching
    let pattern = parse("email LIKE '%@example.com' AND NOT deleted").unwrap();
}
```

### Evaluating Expressions

```rust
use std::collections::HashMap;
use sqlexpr_rust::{evaluate, RuntimeValue};

fn main() {
    // Create variable bindings
    let mut bindings = HashMap::new();
    bindings.insert("age".to_string(), RuntimeValue::Integer(25));
    bindings.insert("status".to_string(), RuntimeValue::String("active".to_string()));
    bindings.insert("premium".to_string(), RuntimeValue::Boolean(true));

    // Evaluate the expression
    let result = evaluate("age >= 18 AND status = 'active' AND premium", &bindings).unwrap();
    assert_eq!(result, true);

    // Arithmetic evaluation
    bindings.insert("price".to_string(), RuntimeValue::Float(99.99));
    bindings.insert("quantity".to_string(), RuntimeValue::Integer(10));

    let result = evaluate("(price * quantity) > 500", &bindings).unwrap();
    assert_eq!(result, true);

    // Pattern matching
    bindings.insert("email".to_string(), RuntimeValue::String("user@example.com".to_string()));

    let result = evaluate("email LIKE '%@example.com'", &bindings).unwrap();
    assert_eq!(result, true);
}
```

### Error Handling

```rust
use sqlexpr_rust::{evaluate, RuntimeValue, EvalError};
use std::collections::HashMap;

fn main() {
    let mut bindings = HashMap::new();
    bindings.insert("x".to_string(), RuntimeValue::String("hello".to_string()));

    // Type error: string in arithmetic
    let result = evaluate("(x + 10) > 0", &bindings);
    match result {
        Err(EvalError::TypeError { operation, expected, actual, context }) => {
            println!("Type error in {}: expected {}, got {} ({})",
                     operation, expected, actual, context);
        }
        _ => {}
    }

    // Type error: incompatible IN list
    bindings.insert("y".to_string(), RuntimeValue::Integer(42));
    let result = evaluate("y IN ('a', 'b', 'c')", &bindings);
    assert!(matches!(result, Err(EvalError::TypeError { .. })));
}
```

## Project Layout

```
sqlexpr-rust/
├── src/
│   ├── lib.rs           # Public API and re-exports
│   ├── lexer.rs         # Tokenization
│   ├── parser.rs        # Recursive descent parser
│   ├── ast.rs           # Abstract Syntax Tree definitions
│   └── evaluator.rs     # Expression evaluation engine
├── tests/
│   ├── parser_tests.rs  # Parser test suite (155 tests)
│   └── evaluator_tests.rs # Evaluator test suite (111 tests)
├── examples/
│   ├── showcase.rs      # Feature demonstration
│   └── ...              # Additional examples
├── docs/
│   ├── EVALUATION_DESIGN.md            # Design alternatives
│   ├── EVALUATOR_IMPLEMENTATION_PLAN.md # Implementation roadmap
│   └── command_prompts.md              # Development notes
├── SqlExprParser-EBNF-Final.ebnf  # Formal grammar specification
├── Cargo.toml
└── README.md
```

## Core Components

### Lexer (`src/lexer.rs`)
Tokenizes input strings into a stream of tokens. Handles:
- Keywords (case-insensitive)
- Identifiers and variables
- Numeric literals (int, float, hex, octal, scientific)
- String literals with escapes
- Operators and punctuation
- Comments (line and block)

### Parser (`src/parser.rs`)
Recursive descent parser implementing the EBNF grammar. Features:
- Operator precedence handling
- Type safety at grammar level
- Lookahead for disambiguation
- Detailed error messages with position info

### AST (`src/ast.rs`)
Hierarchical AST structure:
- `BooleanExpr`: AND, OR, NOT, literals, variables, relational expressions
- `RelationalExpr`: Comparisons, LIKE, BETWEEN, IN, IS NULL
- `ValueExpr`: Arithmetic operations, literals, variables

### Evaluator (`src/evaluator.rs`)
Evaluation engine with:
- Variable binding resolution
- Type checking and coercion
- Short-circuit boolean logic
- Pattern matching for LIKE
- Comprehensive error handling

## Grammar Overview

The grammar enforces type safety at parse time:

```ebnf
BooleanExpression = BooleanOrExpression ;
BooleanOrExpression = BooleanAndExpression { "OR" BooleanAndExpression } ;
BooleanAndExpression = BooleanTerm { "AND" BooleanTerm } ;
BooleanTerm = "NOT" BooleanTerm
            | "(" BooleanExpression ")"
            | BooleanLiteral
            | Variable
            | RelationalExpression ;

RelationalExpression = ValueExpression ComparisonOp ValueExpression
                     | ValueExpression "LIKE" Pattern
                     | ValueExpression "BETWEEN" ValueExpression "AND" ValueExpression
                     | ValueExpression "IN" "(" ValueList ")"
                     | ValueExpression "IS" ["NOT"] "NULL" ;

ValueExpression = AdditiveExpression ;
AdditiveExpression = MultiplicativeExpression { ("+" | "-") MultiplicativeExpression } ;
MultiplicativeExpression = UnaryExpression { ("*" | "/" | "%") UnaryExpression } ;
UnaryExpression = ["+" | "-"] PrimaryExpression ;
PrimaryExpression = Literal | Variable | "(" ValueExpression ")" ;
```

See `SqlExprParser-EBNF-Final.ebnf` for the complete formal grammar.

## Type System

### RuntimeValue Types
- `Integer(i64)`: 64-bit signed integers
- `Float(f64)`: 64-bit floating point
- `String(String)`: UTF-8 strings
- `Boolean(bool)`: true/false
- `Null`: SQL NULL value

### Type Coercion Rules
1. **Arithmetic**: Int + Int → Int, Float + Float → Float
2. **Mixed arithmetic**: Int + Float → Float (automatic promotion)
3. **Division**: Always returns Float (e.g., `7 / 2 = 3.5`)
4. **Comparisons**: Same types compared directly; Int/Float mixing allowed
5. **NULL handling**: NULL in arithmetic/comparisons raises error; use `IS NULL`

## Examples

### Boolean Logic
```sql
TRUE AND FALSE                          -- false
age >= 18 AND status = 'active'        -- depends on bindings
(x > 10 OR y > 10) AND NOT deleted     -- compound condition
```

### Arithmetic
```sql
(price * quantity) > 1000              -- arithmetic in comparison
(revenue - cost) / revenue >= 0.2      -- percentage calculation
amount % 100 = 0                       -- check divisibility
```

### Pattern Matching
```sql
email LIKE '%@example.com'             -- domain match
name LIKE 'J%n'                        -- starts with J, ends with n
code LIKE 'A___B'                      -- A + 3 chars + B
text LIKE '50\%' ESCAPE '\'            -- literal % character
```

### Range and Membership
```sql
age BETWEEN 18 AND 65                  -- inclusive range
status IN ('active', 'pending')        -- membership test
score NOT BETWEEN 0 AND 59             -- exclusion
role NOT IN ('admin', 'moderator')     -- negative membership
```

### Null Handling
```sql
middle_name IS NULL                    -- null check
email IS NOT NULL                      -- non-null check
-- x + NULL  would raise NullInOperation error
-- x > NULL  would raise NullInOperation error
```

## Running Examples

```bash
# Run the feature showcase
cargo run --example showcase

# Enable pretty-printing of AST
SQLEXPR_PRETTY=true cargo run --example showcase

# Run all tests
cargo test

# Run specific test suite
cargo test --test parser_tests
cargo test --test evaluator_tests

# Build documentation
cargo doc --open
```

## Testing

The project includes comprehensive test coverage:

- **Parser tests** (`tests/parser_tests.rs`): 155 tests covering all grammar features
- **Evaluator tests** (`tests/evaluator_tests.rs`): 111 tests covering all operations
- **Unit tests** (`src/lib.rs`, modules): 13 embedded tests
- **Doc tests**: 1 documentation example test

Total: **280 tests**

Run tests with:
```bash
cargo test                  # All tests
cargo test --verbose        # With output
cargo test <pattern>        # Specific tests
```

### Viewing Abstract Syntax Trees (ASTs)

Tell the parser to pretty print ASTs of parsed expressions using the *SQLEXPR_PRETTY* environment variable. For example, the following commands can be used to dump the ASTs generated by the *parser_tests* and *evaluator_tests* programs.  These commands should be run from the top-level project directory.  For easy reference, the output files from these test programs are shipped with the source code.

```bash
SQLEXPR_PRETTY=true cargo test --test parser_tests -- --nocapture --test-threads=1 > examples/output/parser_tests.out
SQLEXPR_PRETTY=true cargo test --test evaluator_tests -- --nocapture --test-threads=1 > examples/output/evaluator_tests.out
```

## Error Messages

The library provides detailed error messages:

### Parse Errors
```
Parse error: Unexpected token ')' near position 15 in:
  (x > 5 AND y < )
```

### Evaluation Errors
```
Type error in addition: expected numeric types, got string and integer
(context: arithmetic operation)

NULL value in GreaterThan operation (context: cannot compare NULL).
NULL is only allowed in IS NULL/IS NOT NULL

Division by zero in expression: x / 0 > 5
```

## Performance Considerations

- **Parser**: Single-pass recursive descent, O(n) complexity
- **Lexer**: Single-pass tokenization, O(n) complexity
- **Evaluator**: Direct evaluation without intermediate representation
- **Short-circuit**: AND/OR operators short-circuit for efficiency
- **Pattern matching**: Regex-based LIKE uses Rust's `regex` crate

## Limitations

1. **No subqueries**: Only standalone boolean expressions
2. **No aggregate functions**: No `SUM`, `COUNT`, etc.
3. **No date/time types**: Only basic types (int, float, string, bool, null)
4. **Case-sensitive strings**: String comparisons are case-sensitive
5. **No COLLATE**: String ordering uses Rust's string comparison

## License

See `LICENSE` file for details.

## Contributing

Contributions are welcome! Please ensure:
1. All tests pass: `cargo test`
2. Code follows Rust conventions: `cargo fmt`
3. No warnings: `cargo clippy`
4. Add tests for new features

## Documentation

- **Grammar**: See `SqlExprParser-EBNF-Final.ebnf`
- **API docs**: Run `cargo doc --open`
- **Design docs**: See `docs/` directory
- **Examples**: See `examples/` directory

## Acknowledgments

This parser implements a clean separation between boolean and value expressions at the grammar level, ensuring type safety during parsing rather than evaluation.
