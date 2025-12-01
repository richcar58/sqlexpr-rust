use sqlexpr_rust::parse;

fn main() {
    let test = "balance NOT IN (-100, -50, -25, 0)";
    println!("Parsing: {}", test);
    match parse(test) {
        Ok(expr) => println!("✓ Success: {}", expr),
        Err(e) => println!("✗ Error: {}", e),
    }
}
