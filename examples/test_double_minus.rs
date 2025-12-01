use sqlexpr_rust::parse;

fn main() {
    let tests = vec![
        ("with space", "(- -x) = 5"),
        ("with parens", "(-(- x)) = 5"),
        ("without space (comment)", "(--x) = 5"),
    ];
    
    for (name, test) in tests {
        println!("\n[{}] Parsing: {}", name, test);
        match parse(test) {
            Ok(expr) => println!("✓ Success: {}", expr),
            Err(e) => println!("✗ Error: {}", e),
        }
    }
}
