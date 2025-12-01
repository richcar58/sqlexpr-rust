use sqlexpr_rust::parse;

fn main() {
    let tests = vec![
        ("parenthesized_boolean", "(x > 5)"),
        ("nested_parentheses", "((x > 5))"),
        ("parenthesized_comparison", "(x > 5)"),
        ("example_9", "NOT (x = 5 OR y = 10)"),
        ("complex_boolean", "(x > 5 AND y < 10) OR (z = 20 AND NOT w >= 100)"),
    ];
    
    for (name, test) in tests {
        println!("\n[{}] Parsing: {}", name, test);
        match parse(test) {
            Ok(expr) => println!("✓ Success: {}", expr),
            Err(e) => println!("✗ Error: {}", e),
        }
    }
}
