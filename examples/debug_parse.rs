use sqlexpr_rust::parse;

fn main() {
    let tests = vec![
        "(a + b) > (c - d)",
        "(a + b) > 10 AND name LIKE '%test%'",
    ];
    
    for test in tests {
        println!("\nParsing: {}", test);
        match parse(test) {
            Ok(expr) => println!("✓ Success: {}", expr),
            Err(e) => println!("✗ Error: {}", e),
        }
    }
}
