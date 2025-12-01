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