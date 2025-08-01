// Rust Performance Issues Test File
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

// Performance Issue 1: Inefficient string operations
pub fn inefficient_string_operations() {
    let mut result = String::new();
    
    // MEDIUM: Inefficient string concatenation in loop
    for i in 0..10000 {
        result = result + &format!("Item {}\n", i); // Line 11: Creates new string each time
    }
    
    // Better approach would be:
    // let mut result = String::with_capacity(estimated_size);
    // for i in 0..10000 { result.push_str(&format!("Item {}\n", i)); }
    
    println!("Result length: {}", result.len());
}

// Performance Issue 2: Unnecessary allocations
pub fn unnecessary_allocations() {
    let numbers: Vec<i32> = (0..1000000).collect();
    
    // MEDIUM: Creating unnecessary intermediate collections
    let processed: Vec<String> = numbers
        .iter()
        .map(|n| n.to_string()) // Line 26: Could be done lazily
        .collect::<Vec<String>>() // Line 27: Unnecessary collection
        .iter()
        .filter(|s| s.len() > 2) // Line 29: Could be done before map
        .map(|s| format!("Number: {}", s)) // Line 30: Another allocation
        .collect();
    
    println!("Processed {} items", processed.len());
}

// Performance Issue 3: Inefficient data structures
pub fn inefficient_data_structures() {
    let mut data = Vec::new();
    
    // MEDIUM: Using Vec for frequent insertions at beginning
    for i in 0..10000 {
        data.insert(0, i); // Line 40: O(n) operation, should use VecDeque
    }
    
    // MEDIUM: Linear search when HashMap would be better
    let mut found_items = Vec::new();
    for target in 0..1000 {
        for (index, &item) in data.iter().enumerate() { // Line 45: O(n²) complexity
            if item == target {
                found_items.push(index);
                break;
            }
        }
    }
    
    println!("Found {} items", found_items.len());
}

// Performance Issue 4: Blocking operations on main thread
pub fn blocking_operations() {
    println!("Starting long operation...");
    
    // HIGH: Blocking the main thread
    thread::sleep(Duration::from_secs(5)); // Line 58: Should be async
    
    // MEDIUM: Synchronous file operations
    let _contents = std::fs::read_to_string("/etc/hosts") // Line 61: Should be async
        .unwrap_or_else(|_| "default".to_string());
    
    println!("Operation completed");
}

// Performance Issue 5: Memory inefficient operations
pub fn memory_inefficient() {
    // MEDIUM: Loading entire file into memory
    let large_data = vec![0u8; 100_000_000]; // Line 69: 100MB allocation
    
    // MEDIUM: Cloning large data unnecessarily
    let cloned_data = large_data.clone(); // Line 72: Unnecessary clone
    
    // MEDIUM: Not using iterators efficiently
    let mut processed = Vec::new();
    for i in 0..cloned_data.len() {
        processed.push(cloned_data[i] * 2); // Line 76: Could use map
    }
    
    println!("Processed {} bytes", processed.len());
}

// Performance Issue 6: Inefficient error handling
pub fn inefficient_error_handling() -> Result<String, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    
    for i in 0..1000 {
        // MEDIUM: Using Result in hot path
        let result = risky_operation(i)?; // Line 87: Should batch or use different approach
        results.push(result);
    }
    
    Ok(results.join(","))
}

fn risky_operation(n: i32) -> Result<String, &'static str> {
    if n % 100 == 0 {
        Err("Divisible by 100") // Frequent errors in hot path
    } else {
        Ok(format!("Value: {}", n))
    }
}

// Code Quality Issues
pub fn code_quality_issues() {
    // LOW: Unused variables
    let unused_variable = "This is never used"; // Line 103
    let _another_unused = 42; // Line 104
    
    // MEDIUM: Complex nested loops
    for i in 0..100 {
        for j in 0..100 {
            for k in 0..100 { // Line 108: Deep nesting, could be refactored
                if i * j * k > 50000 {
                    println!("Found: {} {} {}", i, j, k);
                    break;
                }
            }
        }
    }
    
    // LOW: Magic numbers
    let buffer_size = 4096; // Line 116: Should be a named constant
    let timeout = 30000; // Line 117: Should be a named constant
    
    println!("Buffer: {}, Timeout: {}", buffer_size, timeout);
}
