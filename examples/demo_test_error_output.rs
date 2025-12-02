// Demonstration of error output in tests
// This simulates what happens when a test fails

use sqlexpr_rust::parse;

fn main() {
    println!("Demonstration of Test Error Output");
    println!("===================================\n");

    println!("When a test fails with the new error handling pattern:");
    println!("1. The error is printed to stderr via eprintln!");
    println!("2. The assertion then fails\n");

    println!("Example test code:");
    println!("  let result = parse(\"'unterminated\");");
    println!("  if let Err(e) = &result {{");
    println!("      eprintln!(\"Parse error: {{}}\", e);");
    println!("  }}");
    println!("  assert!(result.is_ok());\n");

    println!("Output when this test runs:\n");

    // Simulate the test
    let result = parse("'unterminated");
    if let Err(e) = &result {
        eprintln!("Parse error: {}", e);
    }

    // Show what would happen
    if result.is_err() {
        println!("\nSTDERR: Parse error: ...");
        println!("STDOUT: assertion `left == right` failed");
        println!("  left: false");
        println!("  right: true");
    }
}
