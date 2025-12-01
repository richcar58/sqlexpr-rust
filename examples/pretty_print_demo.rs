// Comprehensive pretty print demonstration
use sqlexpr_rust::parse;

fn main() {
    println!("SQL Expression Parser - Pretty Print Feature Demo");
    println!("=================================================\n");

    let test_cases = vec![
        ("Simple comparison", "x > 5"),
        ("Boolean AND", "x > 5 AND y < 10"),
        ("Boolean OR", "status = 'active' OR status = 'pending'"),
        ("Boolean NOT", "NOT deleted"),
        ("Arithmetic", "(a + b) * (c - d) = 100"),
        ("LIKE operator", "email LIKE '%@example.com'"),
        ("LIKE with ESCAPE", "text LIKE '%50\\%%' ESCAPE '\\'"),
        ("BETWEEN", "age BETWEEN 18 AND 65"),
        ("IN operator", "status IN ('active', 'pending', 'verified')"),
        ("IS NULL", "value IS NOT NULL"),
        ("Complex nested", "(x > 5 AND y < 10) OR (z = 20 AND NOT w >= 100)"),
    ];

    println!("Environment: SQLEXPR_PRETTY = {:?}\n",
        std::env::var("SQLEXPR_PRETTY").unwrap_or_else(|_| "not set".to_string()));

    for (i, (name, expr)) in test_cases.iter().enumerate() {
        println!("Test Case {}: {}", i + 1, name);
        println!("{}", "=".repeat(60));
        match parse(expr) {
            Ok(_) => {
                if std::env::var("SQLEXPR_PRETTY").is_err() {
                    println!("Expression: {}", expr);
                    println!("Status: ✓ Parsed successfully");
                    println!("(Set SQLEXPR_PRETTY=true to see AST details)\n");
                }
            }
            Err(e) => println!("✗ Error: {}\n", e),
        }
    }
}
