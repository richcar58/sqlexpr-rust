# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`sqlexpr-rust` is a Rust library that provides a complete parser for SQL boolean expressions. The parser is based on the W3C Extended Backus-Naur Form (EBNF) grammar defined in `SqlExprParser-EBNF-Final.ebnf`.

The parser enforces type safety at the grammar level: all top-level expressions must evaluate to boolean values, while arithmetic and value expressions can only appear as operands to relational operators.

## Build and Test Commands

```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run all tests (95 comprehensive tests)
cargo test

# Run library tests only
cargo test --lib

# Run integration tests only
cargo test --test parser_tests

# Run a specific test
cargo test test_like_operator

# Run tests with output
cargo test -- --nocapture

# Check code without building
cargo check

# Format code
cargo fmt

# Run clippy linter
cargo clippy

# Run example programs
cargo run --example debug_parse
```

## Project Structure

- `src/lib.rs` - Public API and re-exports
- `src/ast.rs` - Abstract Syntax Tree definitions (BooleanExpr, RelationalExpr, ValueExpr)
- `src/lexer.rs` - Tokenizer with support for keywords, literals, comments
- `src/parser.rs` - Recursive descent parser implementation
- `tests/parser_tests.rs` - Comprehensive test suite (95 tests covering all language features)
- `SqlExprParser-EBNF-Final.ebnf` - Complete EBNF grammar specification
- Edition: 2024

## Architecture

### Three-Layer AST Hierarchy

The parser implements a strict type hierarchy enforced at the grammar level:

1. **Boolean Expression Layer** (Top Level - always boolean)
   - OR expressions (lowest precedence)
   - AND expressions
   - NOT expressions
   - Boolean literals (TRUE, FALSE)
   - Variables (type checked at runtime)
   - Relational expressions

2. **Relational Expression Layer** (Bridge - produces boolean from values)
   - Equality: `=`, `<>`, `!=`
   - Comparison: `>`, `>=`, `<`, `<=`
   - LIKE / NOT LIKE (with optional ESCAPE clause)
   - BETWEEN / NOT BETWEEN
   - IN / NOT IN
   - IS NULL / IS NOT NULL

3. **Value Expression Layer** (Operands - numeric/string values)
   - Arithmetic: `+`, `-`, `*`, `/`, `%`
   - Unary operators: `+`, `-`
   - Literals: integers, longs, hex, octal, floats, strings, NULL, booleans
   - Variables
   - Parenthesized expressions

### Lexer Features

- Case-insensitive keywords (AND, OR, NOT, LIKE, BETWEEN, etc.)
- String literals with SQL-style escaping (`''` for apostrophe)
- Numeric literals: decimal, hexadecimal (0x), octal (0), floating-point
- Line comments: `--` to end of line
- Block comments: `/* ... */`
- Comprehensive error messages

### Parser Strategy

- **Recursive descent parsing** with proper operator precedence
- **Backtracking** for disambiguating parenthesized expressions
- **Lookahead** to distinguish variables from relational expressions
- **Type safety** enforced at parse time (rejects standalone arithmetic/literals)

### Operator Precedence (Highest to Lowest)

1. Primary expressions (literals, variables, parentheses)
2. Unary operators: `+`, `-` (value), `NOT` (boolean)
3. Multiplicative: `*`, `/`, `%`
4. Additive: `+`, `-`
5. Relational: `>`, `>=`, `<`, `<=`, LIKE, BETWEEN, IN, IS
6. Equality: `=`, `<>`, `!=`
7. Logical AND
8. Logical OR

## Key Design Decisions

1. **Grammar-level type safety**: Non-boolean expressions are rejected at parse time, not runtime
2. **SQL comment compatibility**: `--` always starts a line comment (use `- -x` with space for double negation)
3. **Parenthesized expressions**: Parser uses backtracking to distinguish `(x > 5)` from `(x + y) > 5`
4. **NOT operator context**: `NOT` can be boolean negation OR part of NOT LIKE/BETWEEN/IN operators

## Common Patterns

### Parsing an expression
```rust
use sqlexpr_rust::parse;

let expr = parse("x > 5 AND name LIKE '%test%'")?;
println!("{}", expr); // Displays the parsed AST
```

### Working with AST
```rust
use sqlexpr_rust::{BooleanExpr, RelationalExpr};

match expr {
    BooleanExpr::And(left, right) => { /* ... */ }
    BooleanExpr::Relational(rel) => { /* ... */ }
    _ => { /* ... */ }
}
```

## Test Coverage

The test suite (`tests/parser_tests.rs`) includes 95 comprehensive tests covering:
- All boolean operators (AND, OR, NOT)
- All comparison operators
- LIKE/NOT LIKE with and without ESCAPE
- BETWEEN/NOT BETWEEN
- IN/NOT IN with multiple values
- IS NULL/IS NOT NULL
- All arithmetic operators and precedence
- All literal types (decimal, hex, octal, float, string, NULL, TRUE, FALSE)
- Comments (line and block)
- Complex nested expressions
- Error cases (rejecting standalone arithmetic, etc.)
