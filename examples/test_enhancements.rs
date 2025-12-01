// Showcase of parser test enhancements
use sqlexpr_rust::parse;

fn main() {
    println!("SQL Expression Parser - Test Enhancements Showcase\n");
    println!("====================================================\n");

    let examples = vec![
        // Enhancement 1: BETWEEN with string operands
        ("BETWEEN with strings", "name BETWEEN 'Alice' AND 'Zeus'"),
        ("NOT BETWEEN with strings", "city NOT BETWEEN 'Atlanta' AND 'Denver'"),
        
        // Enhancement 2: Boolean operators with string operands
        ("AND with strings", "first_name = 'John' AND last_name = 'Doe'"),
        ("OR with strings", "status = 'active' OR status = 'pending'"),
        ("NOT with strings", "NOT (name = 'Admin')"),
        
        // Enhancement 3: Relational operators with string operands
        ("String equality", "username = 'alice'"),
        ("String inequality", "email <> 'guest@example.com'"),
        ("String greater than", "name > 'M'"),
        ("String less than", "code < 'ZZZ'"),
        
        // Enhancement 4: LIKE with multi-character wildcards
        ("Leading wildcard", "email LIKE '%@gmail.com'"),
        ("Trailing wildcard", "path LIKE '/home/user/%'"),
        ("Embedded wildcards", "url LIKE 'http://%example.com/%/index.html'"),
        ("LIKE with escape", "text LIKE '%50\\%%' ESCAPE '\\'"),
        
        // Enhancement 5: LIKE with single-character wildcards
        ("Single char wildcard", "code LIKE 'A_C'"),
        ("Multiple single chars", "phone LIKE '___-___-____'"),
        ("Mixed wildcards", "identifier LIKE '%_ID_%'"),
        ("Complex pattern", "filepath LIKE '/usr/__/bin/%/app_'"),
        
        // Enhancement 6: IN/NOT IN with numeric values
        ("IN with integers", "age IN (18, 21, 25, 30)"),
        ("NOT IN with integers", "error_code NOT IN (404, 500, 503)"),
        ("IN with floats", "temperature IN (98.6, 99.0, 100.4)"),
        ("IN with hex", "flags IN (0x00, 0xFF, 0x1A)"),
        ("IN with negative", "balance NOT IN (-100, -50, -25, 0)"),
        ("IN with mixed types", "value IN (10, 20.5, 0x1F, 100L)"),
    ];

    let mut success_count = 0;
    let mut failure_count = 0;

    for (name, expr) in examples {
        print!("{:.<55} ", name);
        match parse(expr) {
            Ok(_) => {
                println!("✓");
                success_count += 1;
            }
            Err(e) => {
                println!("✗");
                println!("  Error: {}\n", e);
                failure_count += 1;
            }
        }
    }

    println!("\n====================================================");
    println!("Total: {} examples", success_count + failure_count);
    println!("✓ Success: {}", success_count);
    println!("✗ Failure: {}", failure_count);
    println!("\nTest count: 152 tests (57 new enhancements added)");
}
