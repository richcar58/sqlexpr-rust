#!/bin/bash
# Showcase the pretty print feature

echo "=========================================="
echo "SQL Expression Parser - Pretty Print Demo"
echo "=========================================="
echo ""
echo "1. Without SQLEXPR_PRETTY (normal mode):"
echo "------------------------------------------"
cargo run --example test_pretty_print 2>&1 | grep -A 2 "Expression: x > 5 AND"

echo ""
echo "2. With SQLEXPR_PRETTY=true (debug mode):"
echo "------------------------------------------"
SQLEXPR_PRETTY=true cargo run --example test_pretty_print 2>&1 | grep -A 15 "Expression: x > 5 AND"
