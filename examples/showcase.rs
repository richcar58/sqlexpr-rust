// Showcase example demonstrating all parser features
use sqlexpr_rust::parse;

fn main() {
    println!("SQL Expression Parser - Feature Showcase\n");
    println!("=========================================\n");

    let examples = vec![
        ("Simple comparison", "x > 5"),
        ("AND operator", "age >= 18 AND age <= 65"),
        ("OR operator", "status = 'active' OR status = 'pending'"),
        ("NOT operator", "NOT deleted"),
        ("LIKE operator", "email LIKE '%@example.com'"),
        ("LIKE with ESCAPE", "name LIKE '%\\%%' ESCAPE '\\'"),
        ("NOT LIKE", "filename NOT LIKE '%.tmp'"),
        ("BETWEEN", "price BETWEEN 10.00 AND 99.99"),
        ("NOT BETWEEN", "age NOT BETWEEN 13 AND 19"),
        ("IN operator", "country IN ('US', 'UK', 'CA')"),
        ("NOT IN", "status NOT IN ('deleted', 'archived')"),
        ("IS NULL", "middle_name IS NULL"),
        ("IS NOT NULL", "email IS NOT NULL"),
        ("Arithmetic", "(quantity * price) > 1000"),
        ("Complex arithmetic", "((a + b) * c - d) / e = 42"),
        ("Parentheses", "(x > 5 AND y < 10) OR (z = 20)"),
        ("Nested parentheses", "((x > 5) AND (y < 10))"),
        ("Line comment", "x > 5 -- this is a comment"),
        ("Block comment", "x /* comment */ > 5"),
        ("Hex literal", "flags = 0xFF"),
        ("Octal literal", "permissions = 0755"),
        ("Float literal", "temperature > 98.6"),
        ("Scientific notation", "value < 1.5e-10"),
        ("Complex real-world",
         "(customer_age >= 18 AND customer_age <= 65) AND \
          (account_status IN ('active', 'verified')) AND \
          (credit_score > 650 OR has_collateral = TRUE) AND \
          last_login IS NOT NULL"),
    ];

    let mut success_count = 0;
    let mut failure_count = 0;

    for (name, expr) in examples {
        print!("{:.<50} ", name);
        match parse(expr) {
            Ok(parsed) => {
                println!("✓");
                println!("  Input:  {}", expr);
                println!("  Output: {}\n", parsed);
                success_count += 1;
            }
            Err(e) => {
                println!("✗");
                println!("  Error: {}\n", e);
                failure_count += 1;
            }
        }
    }

    println!("=========================================");
    println!("Total: {} examples", success_count + failure_count);
    println!("✓ Success: {}", success_count);
    println!("✗ Failure: {}", failure_count);
}
