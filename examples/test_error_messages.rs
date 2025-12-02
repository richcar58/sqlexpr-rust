// Test improved error messages with position and input context
use sqlexpr_rust::parse;

fn main() {
    println!("Testing Improved Error Messages");
    println!("================================\n");

    let test_cases = vec![
        ("Unterminated string", "'hello world"),
        ("Unterminated block comment", "x > 5 /* comment without end"),
        ("Invalid hexadecimal", "x = 0x"),
        ("Unexpected character", "x @ 5"),
        ("Unexpected dot", "x . y"),
        ("Unexpected exclamation", "x ! y"),
    ];

    for (name, expr) in test_cases {
        println!("Test: {}", name);
        println!("{}", "-".repeat(60));
        match parse(expr) {
            Ok(_) => println!("✓ Unexpectedly succeeded\n"),
            Err(e) => {
                println!("✗ Error (as expected):");
                println!("{}\n", e);
            }
        }
    }

    println!("{}", "=".repeat(60));
    println!("All errors now include:");
    println!("  - Position information (character index)");
    println!("  - Full input clause for context");
}
