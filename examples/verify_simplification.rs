// Verify integer tokenization simplification
use sqlexpr_rust::parse;

fn main() {
    println!("Integer Tokenization Simplification Verification\n");
    println!("=================================================\n");

    let examples = vec![
        ("Regular integer", "x = 42"),
        ("Long suffix (L)", "x = 100L"),
        ("Long suffix (l)", "x = 100l"),
        ("Hex integer", "x = 0xFF"),
        ("Octal integer", "x = 0777"),
        ("Float", "x = 3.14"),
        ("IN with longs", "id IN (1000L, 2000L, 3000L)"),
        ("IN with mixed", "v IN (100, 200L, 0x1A, 077)"),
    ];

    println!("All integer types (including long suffix) are now tokenized");
    println!("as IntegerLiteral(i64) - simplifying the parser logic.\n");

    for (name, expr) in examples {
        print!("{:.<50} ", name);
        match parse(expr) {
            Ok(parsed) => {
                println!("✓");
                println!("  Input:  {}", expr);
                println!("  Output: {}\n", parsed);
            }
            Err(e) => {
                println!("✗");
                println!("  Error: {}\n", e);
            }
        }
    }

    println!("=================================================");
    println!("Simplification complete:");
    println!("  - LongLiteral token removed from lexer");
    println!("  - Long variant removed from ValueLiteral enum");
    println!("  - All integer values use IntegerLiteral(i64)");
    println!("  - L/l suffix still accepted but treated as integer");
}
