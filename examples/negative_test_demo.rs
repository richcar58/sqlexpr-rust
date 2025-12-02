// Demonstration of negative test error handling
use sqlexpr_rust::parse;

fn main() {
    println!("Negative Test Error Handling");
    println!("=============================\n");

    println!("Negative tests verify that invalid input is correctly rejected.\n");

    println!("Pattern for POSITIVE tests (should succeed):");
    println!("---------------------------------------------");
    println!("if let Err(e) = &result {{");
    println!("    eprintln!(\"Parse error: {{}}\", e);");
    println!("}}");
    println!("assert!(result.is_ok());\n");
    println!("→ Prints error when parsing FAILS (unexpected)\n\n");

    println!("Pattern for NEGATIVE tests (should fail):");
    println!("------------------------------------------");
    println!("if let Ok(r) = &result {{");
    println!("    eprintln!(\"Expected error but found success: {{}}\", r);");
    println!("}}");
    println!("assert!(result.is_err());\n");
    println!("→ Prints AST when parsing SUCCEEDS (unexpected)\n\n");

    println!("Examples:");
    println!("=========\n");

    println!("Test 1: Standalone literal (should be rejected)");
    println!("------------------------------------------------");
    let result1 = parse("42");
    if let Ok(r) = &result1 {
        eprintln!("Expected error but found success: {}", r);
    }
    println!("Result: {} ✓", if result1.is_err() { "Rejected" } else { "Accepted" });
    println!();

    println!("Test 2: Standalone arithmetic (should be rejected)");
    println!("---------------------------------------------------");
    let result2 = parse("1 + 2");
    if let Ok(r) = &result2 {
        eprintln!("Expected error but found success: {}", r);
    }
    println!("Result: {} ✓", if result2.is_err() { "Rejected" } else { "Accepted" });
    println!();

    println!("Test 3: Valid comparison (should be accepted)");
    println!("----------------------------------------------");
    let result3 = parse("x > 5");
    if let Err(e) = &result3 {
        eprintln!("Parse error: {}", e);
    }
    println!("Result: {} ✓", if result3.is_ok() { "Accepted" } else { "Rejected" });
    println!();

    println!("Benefits:");
    println!("---------");
    println!("✓ Negative tests print AST if they unexpectedly pass");
    println!("✓ Positive tests print error if they unexpectedly fail");
    println!("✓ Makes debugging test failures much easier");
}
