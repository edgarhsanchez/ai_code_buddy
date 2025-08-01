// JavaScript file with various code quality issues
const API_KEY = "secret-api-key-123"; // Security: Hardcoded API key

function fetchUserData(userId) {
    // Security: SQL injection vulnerability
    const query = `SELECT * FROM users WHERE id = ${userId}`;
    
    // Performance: Synchronous operation in async context
    console.log("Fetching user:", userId);
    
    // Code Quality: No error handling
    fetch(`/api/users/${userId}`)
        .then(response => response.json())
        .then(data => {
            console.log("User data:", data); // Debug logging in production
            eval(data.script); // Security: Code injection vulnerability
        });
}

// Style: Inconsistent formatting and unused variables
var unused_var = "never used";
let   badlyFormatted={"key":"value","another":"item"};

// Performance: Inefficient loop
for(let i = 0; i < 1000000; i++) {
    document.querySelector('.item').innerHTML += `<div>Item ${i}</div>`;
}

// Security: No input validation
function processInput(userInput) {
    document.getElementById("output").innerHTML = userInput; // XSS vulnerability
    window.location = userInput; // Open redirect vulnerability
}
