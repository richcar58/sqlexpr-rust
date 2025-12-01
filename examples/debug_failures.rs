use sqlexpr_rust::parse;

fn main() {
    let tests = vec![
        ("parenthesized_boolean", "(x > 5)"),
        ("nested_parentheses", "((x > 5))"),
        ("not_like", "name NOT LIKE '%test%'"),
        ("not_between", "age NOT BETWEEN 18 AND 65"),
        ("not_in", "status NOT IN ('inactive', 'deleted')"),
        ("double_unary_minus", "(--x) = 5"),
        ("example_9", "NOT (x = 5 OR y = 10)"),
    ];
    
    for (name, test) in tests {
        println!("\n[{}] Parsing: {}", name, test);
        match parse(test) {
            Ok(expr) => println!("✓ Success: {}", expr),
            Err(e) => println!("✗ Error: {}", e),
        }
    }
}
