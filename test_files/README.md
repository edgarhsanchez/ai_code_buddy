# Test Files for AI Code Analysis

This directory contains test files designed to validate the AI-powered code analysis capabilities across different programming languages and issue types.

## Directory Structure

```
test_files/
├── javascript/
│   ├── security_vulnerabilities.js    # Security issues (XSS, injection, etc.)
│   └── performance_issues.js          # Performance problems (memory leaks, blocking)
├── python/
│   └── security_vulnerabilities.py    # Python security issues (SQLi, pickle, etc.)
└── rust/
    ├── security_vulnerabilities.rs    # Rust security issues (unsafe, credentials)
    └── performance_issues.rs          # Rust performance problems (allocations, complexity)
```

## Test Categories

### 🚨 Security Vulnerabilities
- **SQL Injection**: Direct query construction with user input
- **Command Injection**: Shell command execution with user data
- **XSS (Cross-Site Scripting)**: Unsafe DOM manipulation
- **Code Injection**: eval() and similar dangerous functions
- **Hardcoded Credentials**: API keys, passwords in source code
- **Path Traversal**: Unvalidated file path operations
- **Unsafe Deserialization**: pickle.loads(), yaml.load()
- **Unsafe Memory Operations**: Raw pointers, unsafe blocks

### ⚠️ Performance Issues
- **Memory Leaks**: Event listeners not cleaned up
- **Inefficient Algorithms**: O(n²) complexity where O(n) possible
- **Blocking Operations**: Synchronous I/O on main thread
- **Unnecessary Allocations**: String concatenation in loops
- **Poor Data Structure Choice**: Using Vec for frequent insertions
- **DOM Thrashing**: Multiple forced reflows

### 📝 Code Quality Issues
- **Unused Variables**: Declared but never used
- **Debug Logging**: console.log/println! in production code
- **Magic Numbers**: Hardcoded values without explanation
- **Deep Nesting**: Complex nested loops/conditions
- **Inconsistent Formatting**: Mixed spacing and styles

## Usage

These test files are automatically used when running the AI code analysis tool:

```bash
# Analyze specific test files
./ai-code-buddy test_files main feature-branch --cli

# Test against all languages
git add test_files/
git commit -m "Add test files"
./ai-code-buddy . main HEAD --cli
```

## Expected Analysis Results

The AI analysis should detect and report:

1. **Critical Issues**: Security vulnerabilities requiring immediate attention
2. **High Priority**: Performance problems affecting user experience  
3. **Medium Issues**: Code quality improvements for maintainability
4. **Low Priority**: Style and unused code cleanup

Each issue should include:
- **Severity Level**: 🚨 Critical, ⚠️ High, 📝 Medium/Low
- **Line Number**: Exact location of the issue
- **Description**: Clear explanation of the problem
- **Category**: Security, Performance, Code Quality

## Adding New Test Cases

When adding new test files:

1. **Follow naming convention**: `{category}_{language}.{ext}`
2. **Include line comments**: Mark expected issues with `// Line X:`
3. **Cover edge cases**: Test boundary conditions and common mistakes
4. **Document expected results**: Add comments explaining what should be detected

This test suite ensures the AI analysis maintains high accuracy and catches real-world security and performance issues.
