use std::collections::HashMap;

fn main() {
    let mut scores = HashMap::new();
    scores.insert("Alice", 10);
    scores.insert("Bob", 20);
    
    // Potential issue: using unwrap() without error handling
    let alice_score = scores.get("Alice").unwrap();
    println!("Alice's score: {}", alice_score);
    
    // Memory inefficient concatenation in loop
    let mut result = String::new();
    for i in 0..1000 {
        result = result + &i.to_string(); // This creates new strings
    }
    
    // Hardcoded password (security issue) - UPDATED
    let password = "supersecret123";
    println!("Default password: {}", password);
    
    // NEW: SQL injection vulnerable code
    let user_input = "'; DROP TABLE users; --";
    let query = format!("SELECT * FROM users WHERE id = '{}'", user_input);
    println!("Query: {}", query);
}

// Function without error handling
fn divide(a: f64, b: f64) -> f64 {
    a / b // No check for division by zero
}

// Unused function
fn unused_function() {
    println!("This function is never called");
}
