// Test pretty printing functionality
use sqlexpr_rust::parse;

fn main() {
    println!("Testing Pretty Print Feature");
    println!("============================\n");

    let examples = vec![
        "x > 5",
        "x > 5 AND y < 10",
        "(a + b) * (c - d) = 100",
        "name LIKE '%test%' AND age BETWEEN 18 AND 65",
        "status IN ('active', 'pending') OR priority > 5",
        "NOT (x = 10 OR y = 20) AND z IS NOT NULL",
    ];

    println!("Note: Set SQLEXPR_PRETTY=true environment variable to enable pretty printing\n");
    
    for expr in examples {
        println!("Expression: {}", expr);
        match parse(expr) {
            Ok(_) => println!("✓ Parsed successfully\n"),
            Err(e) => println!("✗ Error: {}\n", e),
        }
    }
}
