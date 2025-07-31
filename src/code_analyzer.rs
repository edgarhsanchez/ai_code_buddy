use crate::review::{CodeIssue, IssueCategory, Severity};
use regex::Regex;
use std::collections::HashMap;

pub struct CodeAnalyzer {
    rust_patterns: HashMap<String, (IssueCategory, Severity, String)>,
    general_patterns: HashMap<String, (IssueCategory, Severity, String)>,
}

impl CodeAnalyzer {
    pub fn new() -> Self {
        let mut analyzer = Self {
            rust_patterns: HashMap::new(),
            general_patterns: HashMap::new(),
        };

        analyzer.initialize_rust_patterns();
        analyzer.initialize_general_patterns();
        analyzer
    }

    fn initialize_rust_patterns(&mut self) {
        // Security patterns
        self.rust_patterns.insert(
            r"\.unwrap\(\)".to_string(),
            (
                IssueCategory::PotentialBugs,
                Severity::Medium,
                "Consider using proper error handling instead of unwrap()".to_string(),
            ),
        );

        self.rust_patterns.insert(
            r"\.expect\([^)]*\)".to_string(),
            (
                IssueCategory::PotentialBugs,
                Severity::Low,
                "Consider using ? operator or match for better error handling".to_string(),
            ),
        );

        // Performance patterns
        self.rust_patterns.insert(
            r"\.clone\(\)".to_string(),
            (
                IssueCategory::Performance,
                Severity::Low,
                "Consider if clone is necessary, use references when possible".to_string(),
            ),
        );

        self.rust_patterns.insert(
            r"String::from\([^)]*\)\.as_str\(\)".to_string(),
            (
                IssueCategory::Performance,
                Severity::Medium,
                "Unnecessary String allocation, use string literal directly".to_string(),
            ),
        );

        // Style patterns
        self.rust_patterns.insert(
            r"fn\s+\w+\([^)]*\)\s*\{[^}]*println!\([^)]*\);[^}]*\}".to_string(),
            (
                IssueCategory::Style,
                Severity::Low,
                "Consider using logging instead of println! in production code".to_string(),
            ),
        );

        // Maintainability patterns
        self.rust_patterns.insert(
            r"(?:TODO|FIXME|XXX|HACK)".to_string(),
            (
                IssueCategory::Maintainability,
                Severity::Low,
                "Address TODO/FIXME comments before production".to_string(),
            ),
        );
    }

    fn initialize_general_patterns(&mut self) {
        // Security patterns
        self.general_patterns.insert(
            r"(?i)password|secret|token|api_key".to_string(),
            (
                IssueCategory::Security,
                Severity::High,
                "Potential hardcoded credentials found".to_string(),
            ),
        );

        // Documentation patterns
        self.general_patterns.insert(
            r"pub\s+fn\s+\w+\([^)]*\)\s*(?:->.*?)?\s*\{".to_string(),
            (
                IssueCategory::Documentation,
                Severity::Low,
                "Public function missing documentation".to_string(),
            ),
        );

        // Testing patterns
        self.general_patterns.insert(
            r"#\[test\]".to_string(),
            (
                IssueCategory::Testing,
                Severity::Info,
                "Test function found".to_string(),
            ),
        );
    }

    pub fn analyze_code(&self, content: &str, file_path: &str, language: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();

        // Choose patterns based on language
        let patterns = match language.to_lowercase().as_str() {
            "rust" => &self.rust_patterns,
            _ => &self.general_patterns,
        };

        for (pattern, (category, severity, suggestion)) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for (line_num, line) in content.lines().enumerate() {
                    if regex.is_match(line) {
                        issues.push(CodeIssue {
                            category: category.clone(),
                            severity: severity.clone(),
                            description: format!("Pattern '{}' found", pattern),
                            file_path: file_path.to_string(),
                            line_number: Some((line_num + 1) as u32),
                            suggestion: suggestion.clone(),
                            code_snippet: Some(line.to_string()),
                        });
                    }
                }
            }
        }

        // Additional analysis based on file structure
        issues.extend(self.analyze_file_structure(content, file_path));

        issues
    }

    fn analyze_file_structure(&self, content: &str, file_path: &str) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Check for long functions
        let mut function_start = None;
        let mut brace_count = 0;

        for (line_num, line) in lines.iter().enumerate() {
            if line.contains("fn ") && !line.contains("//") {
                function_start = Some(line_num);
                brace_count = 0;
            }

            brace_count += line.matches('{').count() as i32;
            brace_count -= line.matches('}').count() as i32;

            if let Some(start) = function_start {
                if brace_count == 0 && line_num > start {
                    let function_length = line_num - start;
                    if function_length > 50 {
                        issues.push(CodeIssue {
                            category: IssueCategory::Maintainability,
                            severity: Severity::Medium,
                            description: "Function is too long".to_string(),
                            file_path: file_path.to_string(),
                            line_number: Some((start + 1) as u32),
                            suggestion: format!(
                                "Consider breaking this {}-line function into smaller functions",
                                function_length
                            ),
                            code_snippet: None,
                        });
                    }
                    function_start = None;
                }
            }
        }

        // Check for deeply nested code
        for (line_num, line) in lines.iter().enumerate() {
            let indent_level = line.len() - line.trim_start().len();
            if indent_level > 24 {
                // More than 6 levels of indentation (4 spaces each)
                issues.push(CodeIssue {
                    category: IssueCategory::Readability,
                    severity: Severity::Medium,
                    description: "Deeply nested code detected".to_string(),
                    file_path: file_path.to_string(),
                    line_number: Some((line_num + 1) as u32),
                    suggestion: "Consider extracting nested logic into separate functions"
                        .to_string(),
                    code_snippet: Some(line.to_string()),
                });
            }
        }

        issues
    }

    pub fn analyze_complexity(&self, content: &str) -> f32 {
        // Simple cyclomatic complexity calculation
        let complexity_keywords = [
            "if", "else", "while", "for", "match", "loop", "&&", "||", "?", "catch", "except",
        ];

        let mut complexity = 1.0; // Base complexity

        for keyword in &complexity_keywords {
            complexity += content.matches(keyword).count() as f32;
        }

        complexity
    }
}
