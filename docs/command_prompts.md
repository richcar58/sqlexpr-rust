# A Word from the Developer

The command prompts in this file capture the step-by-step refinement of parser and evaluator code. Anthropic's Claude Sonnet 4.5 was used to generate the code.  Some manual changes made directly to the code are not captured here.  Even without these manual modifications it's unlikely that running the same prompts in the future will yield the exact same results.  The prompts are given mostly for the historical record and to remind me what I did.   


# Generate Parser from EBNF Definition

1. brave_web_search w3c Extended Backus-Naur Form (EBNF) notation defined in Extensible Markup Language (XML) 1.0 (Fifth Edition) document, which can be found at https://www.w3.org/TR/xml/#sec-notation.
2. Use the w3c Extended Backus-Naur Form (EBNF) notation as defined in https://www.w3.org/TR/xml/#sec-notation to generate a rust parser for the language defined in local file SqlExprParser-EBNF-final.ebnf.  Please generate the complete parser specification, including LIKE and NOT LIKE operators with and without an ESCAPE clause; the BETWEEN operator; the IN and NOT IN operators; all forms of comments and all other language syntax.  All generated source code should reside in the SqlExpr-rust/src directory.  Also generate tests in the SqlExpr-rust/test diectory that exercise all language syntax and run those tests.  Show the plan and request permission before executing any changes.

## Parser Test Enhancements

1. Add BETWEEM operator support for string operands.  Add tests for BETWEEN and NOT BETWEEN with string operands.
2. Add more tests for Boolean Operators that have string operands.
3. Add more tests for Relational Operators that have string operands.
4. Add more tests for LIKE and NOT LIKE operators with embedded multi-character placeholders, with and without ESCAPE clauses.  
5. Add more tests for LIKE and NOT LIKE operators with zero or more leading, trailing and/or embedded single character placeholders.
6. Add more tests for IN and NOT IN operators with numeric value sets.

## Simplify Integer Tokeniation and Parsing

1. In lexer.rs, convert literal values assigned the LongLiteral(i64) lexeme to be assign the IntegerLiteral(i64) lexeme.
2. Replace all references to LongLiteral(i64) with references to IntegerLiteral(i64) parser.rs and ast.rs
3. Remove the LongLiteral(i64) lexeme from lexer.rs.

## Pretty Print

1. Let's enhance AST parsing with optional pretty printing.  Add a pretty_print field of type bool to the Parser struct in parser.rs.  Assign this field true if the SQLEXPR_PRETTY environment variable exists and is set to true (case insensitive).  Otherwise, set pretty_print to false.  Before the parse() function in parser.rs returns its AST result, if the pretty_print field is true, then print the original input string and the parsed AST to stdout.  Each node in the AST should have its type name and contents printed, with printing indented 3 spaces to the right of the printed output of its parent node.  The root node printing is not indented.

## Improve Error Messages

1. Append more debugging information to the errors returned using Err in lexer.rs  To each Err message append " near position {self.position} in:\n  {String::from_iter(&self.input)}".
2. Change the error handling logic in each test case in tests/parser_tests.rs to remove the panic calls.  Instead, right after the line that assigns the result variable, this code should complete the test function:

    if let Err(e) = &result {
        eprintln!("Parse error: {}", e);
    }
    assert!(result.is_ok());

3. Good, are are almost there.  In  each test case in tests/parser_tests.rs that has a call to panic!(panic_message), add the panic_message as a second parameter to the preceeding assert! statement:  

    assert!(result.is_ok());

    This assert! statement appears just before the match statement that contains the panic! call.  After copying the panic_message to the assert! call, remove the match containing the panic! call from the test function.

4. Almost there. The negative tests in tests/parser_tests print an error message when an error is expected.  These tests need to be updated:

    test_reject_standalone_literal
    test_reject_standalone_arithmetic
    test_reject_standalone_string
    test_reject_parenthesized_arithmetic
    test_reject_unterminated_string
    test_reject_unterminated_block_comment
    test_reject_invalid_operator
    reject_missing_operand

    In each of the above functions, this statement:

    if let Err(e) = &result {
        eprintln!("Parse error: {}", e);
    }

    should be replaced with this statement:

     if let Ok(r) = &result {
        eprintln!("Expected error but found success: {}", r);
    }
   
# SQL Boolean Expression Evaluation - Design Discussion

## Overview

The next enhancement will allow SQL boolean expressions to be evaluated when all variables are bound to actual values.  Building on the current implementation that already parses SQL expression strings into ASTs of type BooleanExpression, we want the following new capabilities:

1. Allow the user to specify a mapping of variable names to actual values, which we will call the *value mapping*.
2. Use the value mapping and the user's parsed expression's AST to bind all free variables in the AST to values.
3. Validate that all variables in the AST are bound.
4. Use a new Evaluator data type to execute the logical expression represented by the AST and its variable bindings.
5. The evaluator's result is true, false or an error, which is returned to the user.

Please suggest different design approaches for the above new capabilities.  Play special attention to capability 4, which focuses on the operational semantics that define how each AST is evaluated, how actual values are substituted for unbound variables, and how the final result is attained.  

___

# Please elaborate on the Approach 1: Visitor Pattern with Mutable Context, but with these requirements:

1. Use the content of evaluation.rs as the starting point for the public interface of the Evaluator.
    1, The **Evaluator** struct obviates the need for ValueMapping and EvalContext types as described in Approach 1.  
    1. The **RuntimeValue** enum provides the mechanism by which callers specify variable names and the values to which they should be bound. 
    2. The caller calls *evaluate(input: &str, map: &HashMap<String, RuntimeValue>)* to evaluate a boolean expression (input) using a value mapping (map).
    3. The **EvalError** type can be enhanced or redefined to improve error reporting.
2. Before an AST clause can be evaluated, all variable names in the clause need to be replaced with their value defined in the value mapping.
    1. Type checking can only be applied after substituting values for variable names.
    2. Evaluation fails with an error if a variable name cannot be mapped to an runtime value.
3. If possible, all code needed to evaluate an expression should be contained in evaluator.rs.
4. Careful attention must be paid to how literal values are mapped to Rust types for evaluation.  
    1. The Literal values that appear in the AST are defined in ast.rs.  The ValueLiteral::Integer values map to i64 in Rust; ValueLiteral::Float map to f64 in Rust; ValueLiteral::String map to String in Rust; BooleanTerm::BooleanLiteral map to bool in Rust.
    2. The literal values in RuntimeValue in evaluator.rs appear in the value mapping on the evaluate() call.  The RuntimeValue::Integer map to i64 in Rust; the RuntimeValue::Float map to f64 in Rust; RuntimeValue::String map to String in Rust; RuntimeValue::Boolean map to bool in Rust.
    3. ValueLiteral::Null and RuntimeValue::Null are case insensitive inputs that both map to the String "NULL" in Rust.
5. Type checking can be performed on operands on-demand before executing an operation.
    1. The ValueLiteral and RuntimeValue types that map to i64 in Rust are compatible.
    2. The ValueLiteral and RuntimeValue types that map to f64 in Rust are compatible.
    3. When mapping numeric types to Rust type, implement type coercion of integer (i64) to float (f64) only (1) on division operations or (2) on arimethic operations that contain both types.
    4. The ValueLiteral and RuntimeValue types that map to String in Rust are compatible.
    5. The ValueLiteral and RuntimeValue types that map to bool in Rust are compatible.
    6. The ValueLiteral and RuntimeValue Null types that map to "NULL" in Rust are compatible.
    7. It is an error for NULL to appear in comparison or arimethic operations.
6. Perform type checking on operands before executing the operation.
    1. Comparison and Equality operators (Gt, Lt, Gte, Lte, Like, Between, Eq, NotEq, etc.) can have numeric or string operand types, but not a mixture of these types in the same clause.
    2. Equality operators (Eq, NotEq) can also have boolean operand types in addition to numeric or string.
    3. Generate precise, human readable error messages when type errors are detected.
7. Correctness requires preserving operator precedence as described in the SqlExprParser-EBNF-Final.ebnf documentation.  The parser encodes these precedence rules into the AST structure it produces, so as long as the evaluator respects AST semantics, operator precedence will be preserved. 

## Testing

Develop a plan for comprehensive testing of evaluation syntax and semantics.  All operators should be exercised with both positive and negative tests.  Positive tests provide value bindings to an expression and evaluate to the expected boolean value.  Negative tests provide value bindings that result in illegal type assignment or missing value assignments errors.

## Documentation

The algorithms and strategies used in the final implementation should be documented.  This includes inline documentation in evaluator code and in test code.  Documentation also includes generating a README.md file that describes what the parser and evaluator do and details how both should be used.  A separate Design.md file describes the design choices made and key aspects of the implementation.

*Please generate a plan for the implementation of the Evaluator and how it will be tested and documented.*

## Tweaks

The fn test_error_in_operator_with_non_string and test_error_in_operator_with_non_numeric tests in evaluator.rs fail without a proper error message.  In both cases, the lefthand argument to the IN clause is a type that doesn't match the type of the vector elements.  A clear error message using EvalError::TypeError should be generated.  The problem may be in the Evaluator::eval_in() function's match statement, which does not distinguish between no match because the lefthand value is not in the vector and a type mismatch between the lefthand value and the vector elements.

Since vectors and arrays in Rust must 

## Create README.md

Create a README.md file to: 

1. Explain the main features of the project.
2. Guide users on how to quickly use the parser and evaluator public functions.
3. Explain the layout and content of the project.
4. Introduce any other information that user would find helpful.


