use std::process::Command;
use std::fs;

// Potentially insecure code for testing AI analysis
pub fn analyze_security_issues() {
    // Security Issue 1: Command injection vulnerability
    let user_input = std::env::args().nth(1).unwrap_or_default();
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("echo {}", user_input)) // Dangerous: no input sanitization
        .output()
        .expect("Failed to execute command");
    
    println!("Output: {:?}", output);
    
    // Security Issue 2: Hardcoded credentials
    let api_key = "sk-1234567890abcdef"; // This should be in environment variables
    let database_password = "admin123"; // Never hardcode passwords
    
    // Security Issue 3: Unsafe block without justification
    unsafe {
        let ptr = std::ptr::null_mut::<i32>();
        *ptr = 42; // This will cause segfault
    }
    
    // Security Issue 4: Path traversal vulnerability
    let filename = "../../../etc/passwd";
    let _content = fs::read_to_string(filename); // No path validation
    
    println!("API Key: {}, Password: {}", api_key, database_password);
}

pub fn inefficient_code() {
    // Performance Issue: Inefficient string concatenation
    let mut result = String::new();
    for i in 0..10000 {
        result = result + &format!("Item {}\n", i); // Creates new string each time
    }
    
    // Code Quality Issue: Unused variables
    let unused_var = "This variable is never used";
    let _another_unused = 42;
    
    // Style Issue: Inconsistent formatting
    let badly_formatted=vec![1,2,3,4,5];
    let   extra_spaces    =    "too many spaces";
    
    println!("Result length: {}", result.len());
}
