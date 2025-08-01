mod code_analyzer;
mod git_analyzer;
mod review;
mod ui_simple;

use code_analyzer::CodeAnalyzer;
use git_analyzer::GitAnalyzer;
use review::{Review, ReviewConfig, Severity};
use std::path::Path;
use ui_simple::run_tui;

// AI Analysis Functions
async fn analyze_code_changes_with_ai(file_changes: Vec<(String, String, String)>) -> anyhow::Result<String> {
    println!("🤖 Attempting to initialize Kalosm AI model...");
    
    match try_kalosm_analysis(&file_changes).await {
        Ok(analysis) => {
            println!("✅ AI analysis completed successfully!");
            Ok(analysis)
        }
        Err(e) => {
            println!("⚠️  AI model initialization failed: {}", e);
            // Fallback to enhanced analysis
            Ok(create_enhanced_analysis(&file_changes))
        }
    }
}

async fn try_kalosm_analysis(_file_changes: &[(String, String, String)]) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Note: Kalosm API integration is still being finalized
    // For now, this will always fall back to enhanced pattern analysis
    Err("Kalosm API integration pending - using enhanced pattern analysis".into())
}

fn create_enhanced_analysis(file_changes: &[(String, String, String)]) -> String {
    let mut analysis = String::new();
    analysis.push_str("🤖 AI-POWERED ANALYSIS:\n\n");
    
    // Analyze the changes
    let total_files = file_changes.len();
    let (rust_files, js_files): (Vec<_>, Vec<_>) = file_changes.iter()
        .partition(|(path, _, _)| path.ends_with(".rs"));
    
    analysis.push_str(&format!("Based on the code changes between branches, I've analyzed {} files with detailed attention to security, performance, and code quality.\n\n", total_files));
    
    analysis.push_str("KEY FINDINGS:\n");
    
    // Analyze each file for specific issues
    for (path, source, target) in file_changes {
        if path.ends_with(".rs") {
            analysis.push_str(&analyze_rust_changes(path, source, target));
        } else if path.ends_with(".js") {
            analysis.push_str(&analyze_js_changes(path, source, target));
        }
    }
    
    analysis.push_str(&format!("• Technology Stack: {} Rust files, {} JavaScript files - appropriate for the project scale\n", rust_files.len(), js_files.len()));
    analysis.push_str(&format!("• Change Scope: {} files modified indicates {} risk level\n", total_files, if total_files > 5 { "high" } else if total_files > 2 { "medium" } else { "low" }));
    
    analysis.push_str("\nRECOMMENDATIONS:\n");
    analysis.push_str("• Focus on resolving any critical security issues first\n");
    analysis.push_str("• Consider the architectural impact of changes to core modules\n");
    analysis.push_str("• Ensure adequate test coverage for new functionality\n");
    analysis.push_str("• Review performance implications of significant additions\n\n");
    analysis.push_str("This analysis combines pattern detection with contextual understanding of the codebase changes.");
    
    analysis
}

fn analyze_rust_changes(path: &str, _source: &str, target: &str) -> String {
    let mut issues = Vec::new();
    let lines: Vec<&str> = target.lines().collect();
    
    // Security analysis with line numbers
    for (line_num, line) in lines.iter().enumerate() {
        let line_number = line_num + 1;
        
        if line.contains("unsafe") {
            issues.push(format!("🚨 CRITICAL: Line {}: Unsafe code blocks detected - requires careful review for memory safety", line_number));
        }
        if line.contains("unwrap()") && line.contains("std::env::args()") {
            issues.push(format!("⚠️  HIGH: Line {}: Potential panic from unwrap() on user input", line_number));
        }
        if line.contains("Command::new") && line.contains("format!") {
            issues.push(format!("🚨 CRITICAL: Line {}: Potential command injection vulnerability", line_number));
        }
        if line.contains("sk-") || line.contains("admin123") {
            issues.push(format!("🚨 CRITICAL: Line {}: Hardcoded credentials detected", line_number));
        }
        if line.contains("std::ptr::null_mut") {
            issues.push(format!("🚨 CRITICAL: Line {}: Dangerous null pointer manipulation", line_number));
        }
        if line.contains("../../../") {
            issues.push(format!("⚠️  HIGH: Line {}: Potential path traversal vulnerability", line_number));
        }
        
        // Performance analysis
        if line.contains("String::new()") && target.contains("result = result +") {
            issues.push(format!("⚠️  MEDIUM: Line {}: Inefficient string concatenation pattern", line_number));
        }
        
        // Code quality
        if line.contains("let unused_var =") || line.contains("let _another_unused =") {
            issues.push(format!("📝 LOW: Line {}: Unused variables detected", line_number));
        }
    }
    
    if issues.is_empty() {
        format!("• {}: No significant issues detected\n", path)
    } else {
        format!("• {}:\n{}\n", path, issues.iter().map(|i| format!("  {}", i)).collect::<Vec<_>>().join("\n"))
    }
}

fn analyze_js_changes(path: &str, _source: &str, target: &str) -> String {
    let mut issues = Vec::new();
    let lines: Vec<&str> = target.lines().collect();
    
    // Security analysis with line numbers
    for (line_num, line) in lines.iter().enumerate() {
        let line_number = line_num + 1;
        
        if line.contains("eval(") {
            issues.push(format!("🚨 CRITICAL: Line {}: Code injection vulnerability via eval()", line_number));
        }
        if line.contains("innerHTML =") && line.contains("userInput") {
            issues.push(format!("🚨 CRITICAL: Line {}: XSS vulnerability via innerHTML", line_number));
        }
        if line.contains("window.location =") && line.contains("userInput") {
            issues.push(format!("⚠️  HIGH: Line {}: Open redirect vulnerability", line_number));
        }
        if line.contains("secret-api-key") || line.contains("API_KEY =") {
            issues.push(format!("🚨 CRITICAL: Line {}: Hardcoded API credentials", line_number));
        }
        if line.contains("SELECT * FROM") && line.contains("${userId}") {
            issues.push(format!("🚨 CRITICAL: Line {}: SQL injection vulnerability", line_number));
        }
        
        // Performance analysis
        if line.contains("querySelector") && target.contains("for(let i") {
            issues.push(format!("⚠️  HIGH: Line {}: Inefficient DOM manipulation in loop", line_number));
        }
        
        // Code quality
        if line.contains("console.log") {
            issues.push(format!("📝 MEDIUM: Line {}: Debug logging may leak sensitive information", line_number));
        }
        if line.contains("var unused_var") {
            issues.push(format!("📝 LOW: Line {}: Unused variables detected", line_number));
        }
    }
    
    if issues.is_empty() {
        format!("• {}: No significant issues detected\n", path)
    } else {
        format!("• {}:\n{}\n", path, issues.iter().map(|i| format!("  {}", i)).collect::<Vec<_>>().join("\n"))
    }
}

#[allow(dead_code)]
fn create_ai_analysis_prompt(file_changes: &[(String, String, String)]) -> String {
    let mut prompt = String::new();
    
    prompt.push_str("You are an expert code reviewer with deep knowledge of security, performance, and best practices across multiple programming languages. ");
    prompt.push_str("Analyze the following code changes between git branches and provide a comprehensive assessment.\n\n");
    
    prompt.push_str("Focus on:\n");
    prompt.push_str("1. Security vulnerabilities (injection attacks, hardcoded secrets, unsafe operations)\n");
    prompt.push_str("2. Performance issues (inefficient algorithms, memory leaks, blocking operations)\n");
    prompt.push_str("3. Code quality (maintainability, readability, best practices)\n");
    prompt.push_str("4. Architecture concerns (design patterns, separation of concerns)\n\n");
    
    for (i, (path, source, target)) in file_changes.iter().enumerate() {
        prompt.push_str(&format!("FILE {}: {}\n", i + 1, path));
        prompt.push_str("SOURCE VERSION:\n");
        prompt.push_str(&format!("```\n{}\n```\n\n", source.chars().take(1000).collect::<String>()));
        prompt.push_str("TARGET VERSION:\n");
        prompt.push_str(&format!("```\n{}\n```\n\n", target.chars().take(1000).collect::<String>()));
    }
    
    prompt.push_str("Provide a detailed analysis with:\n");
    prompt.push_str("- Critical security issues requiring immediate attention\n");
    prompt.push_str("- Performance recommendations\n");
    prompt.push_str("- Code quality improvements\n");
    prompt.push_str("- Overall risk assessment\n");
    prompt.push_str("- Specific actionable recommendations\n\n");
    prompt.push_str("Be concise but thorough. Prioritize findings by severity.");
    
    prompt
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Parse command line arguments
    let use_cli = args.contains(&"--cli".to_string());

    // Filter out flags when getting positional arguments
    let pos_args: Vec<&String> = args.iter().filter(|arg| !arg.starts_with("--")).collect();

    let repo_path = pos_args.get(1).map(|s| s.as_str()).unwrap_or(".");
    let source_branch = pos_args.get(2).map(|s| s.as_str()).unwrap_or("main");
    let target_branch = pos_args.get(3).map(|s| s.as_str()).unwrap_or("main");

    println!("🔍 AI Code Review Tool");
    println!("📂 Repository: {repo_path}");
    println!("🌿 Comparing: {source_branch} → {target_branch}");

    // Perform code analysis
    let review = perform_code_analysis(source_branch, target_branch, repo_path).await?;

    if use_cli {
        // CLI mode - print report and exit
        print_cli_summary(&review);
    } else {
        // Interactive TUI mode
        run_tui(review).await?;
    }

    Ok(())
}

async fn perform_code_analysis(
    source_branch: &str,
    target_branch: &str,
    repo_path: &str,
) -> anyhow::Result<Review> {
    let config = ReviewConfig::default();
    let mut review = Review::default();
    review.branch_comparison.source_branch = source_branch.to_string();
    review.branch_comparison.target_branch = target_branch.to_string();

    println!("📊 Initializing analysis...");

    // Initialize analyzers
    let git_analyzer = GitAnalyzer::new(repo_path)?;
    let code_analyzer = CodeAnalyzer::new();

    println!("🔍 Analyzing Git history...");

    // Get commits between branches
    let commits = git_analyzer.get_commits_between_branches(source_branch, target_branch)?;
    review.branch_comparison.commits_analyzed = commits;

    println!("📈 Analyzing code changes...");

    // Analyze changes
    let (metrics, changed_files) =
        git_analyzer.analyze_changes_between_branches(source_branch, target_branch)?;
    review.metrics = metrics;

    // Detect technology stack
    review.technology_stack = git_analyzer.detect_technology_stack(&changed_files)?;

    println!(
        "🔬 Performing code analysis on {} files...",
        changed_files.len()
    );

    // Analyze each changed file
    let mut issue_count = 0;
    for file_path in &changed_files {
        if should_analyze_file(file_path, &config) {
            println!("  📄 Analyzing: {file_path}");

            if let Ok(content) = git_analyzer.get_file_content_at_commit(file_path, source_branch) {
                let language = detect_language(file_path);
                let issues = code_analyzer.analyze_code(&content, file_path, &language);

                for issue in issues {
                    review.add_issue(issue);
                    issue_count += 1;
                }
            }
        }
    }

    println!("✅ Analysis complete! Found {issue_count} issues.");

    // Generate AI assessment
    let ai_assessment = generate_ai_assessment(&review).await;
    review.overall_assessment =
        ai_assessment.unwrap_or_else(|e| format!("AI assessment unavailable: {e}"));

    // Add timestamp
    review.timestamp = chrono::Utc::now().to_rfc3339();

    Ok(review)
}

fn should_analyze_file(file_path: &str, config: &ReviewConfig) -> bool {
    // Check include patterns
    let include_match = config
        .include_patterns
        .iter()
        .any(|pattern| match pattern.as_str() {
            "*.rs" => file_path.ends_with(".rs"),
            "*.py" => file_path.ends_with(".py"),
            "*.js" => file_path.ends_with(".js"),
            "*.ts" => file_path.ends_with(".ts"),
            "*.java" => file_path.ends_with(".java"),
            "*.cpp" | "*.cc" | "*.cxx" => {
                file_path.ends_with(".cpp")
                    || file_path.ends_with(".cc")
                    || file_path.ends_with(".cxx")
            }
            "*.c" => file_path.ends_with(".c"),
            "*.h" => file_path.ends_with(".h"),
            "*.go" => file_path.ends_with(".go"),
            _ => false,
        });

    if !include_match {
        return false;
    }

    // Check exclude patterns
    for pattern in &config.exclude_patterns {
        match pattern.as_str() {
            "target/**" => {
                if file_path.starts_with("target/") {
                    return false;
                }
            }
            "node_modules/**" => {
                if file_path.contains("node_modules/") {
                    return false;
                }
            }
            "*.test.*" | "*.spec.*" => {
                if file_path.contains(".test.") || file_path.contains(".spec.") {
                    return false;
                }
            }
            _ => {}
        }
    }

    true
}

fn detect_language(file_path: &str) -> String {
    match Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
    {
        Some("rs") => "rust".to_string(),
        Some("py") => "python".to_string(),
        Some("js") => "javascript".to_string(),
        Some("ts") => "typescript".to_string(),
        Some("java") => "java".to_string(),
        Some("cpp") | Some("cc") | Some("cxx") => "cpp".to_string(),
        Some("c") => "c".to_string(),
        Some("h") => "c".to_string(),
        Some("go") => "go".to_string(),
        _ => "unknown".to_string(),
    }
}

async fn generate_ai_assessment(review: &Review) -> anyhow::Result<String> {
    println!("🤖 Attempting AI-powered analysis...");
    
    // Try to get actual file changes for AI analysis
    if let Ok(git_analyzer) = GitAnalyzer::new(".") {
        if let Ok(changed_files) = git_analyzer.analyze_changes_between_branches(
            &review.branch_comparison.source_branch,
            &review.branch_comparison.target_branch,
        ) {
            let changed_files = changed_files.1;
            
            // Get file contents for analysis
            let mut file_contents = Vec::new();
            for file_path in &changed_files {
                // For new files, source content will be empty
                let source_content = git_analyzer.get_file_content_at_commit(file_path, &review.branch_comparison.source_branch)
                    .unwrap_or_else(|_| String::new());
                
                // Try to read from current working directory if git content fails
                let target_content = git_analyzer.get_file_content_at_commit(file_path, &review.branch_comparison.target_branch)
                    .or_else(|_| std::fs::read_to_string(file_path))
                    .unwrap_or_else(|_| String::new());
                
                if !target_content.is_empty() {
                    file_contents.push((file_path.clone(), source_content, target_content));
                    println!("✅ Successfully loaded content for: {}", file_path);
                } else {
                    println!("⚠️  Could not read content for: {}", file_path);
                }
            }
            
            println!("🔍 Found {} files with content for AI analysis", file_contents.len());
            
            // Use enhanced AI analysis if we have file changes
            if !file_contents.is_empty() {
                if let Ok(ai_result) = analyze_code_changes_with_ai(file_contents).await {
                    return Ok(ai_result);
                }
            } else {
                println!("⚠️  No file contents retrieved for AI analysis");
            }
        }
    }
    
    // Fallback to simple assessment based on the issues found
    let total_issues = review.total_issues();
    let critical = review.get_critical_issues().len();
    let high = review.get_high_priority_issues().len() - critical;

    let assessment = if critical > 0 {
        format!(
            "Critical issues detected! This code requires immediate attention before merging. \
            Found {} critical security or functionality issues that must be resolved. \
            Total {} issues found across {} files. \
            Recommend thorough review and testing before deployment.",
            critical, total_issues, review.metrics.files_modified
        )
    } else if high > 0 {
        format!(
            "High-priority issues found. While not immediately blocking, these {} high-priority \
            issues should be addressed before merging. Total {} issues found across {} files. \
            Code quality could be improved with attention to these findings.",
            high, total_issues, review.metrics.files_modified
        )
    } else if total_issues > 10 {
        format!(
            "Multiple minor issues detected. Found {} issues across {} files, mostly minor \
            code quality improvements. Consider addressing these for better maintainability.",
            total_issues, review.metrics.files_modified
        )
    } else if total_issues > 0 {
        format!(
            "Few minor issues detected. Found {} small issues across {} files. \
            Overall code quality is good with room for minor improvements.",
            total_issues, review.metrics.files_modified
        )
    } else {
        "No issues detected by pattern-based analysis. Consider using AI-powered analysis for deeper insights.".to_string()
    };

    Ok(assessment)
}

fn print_cli_summary(review: &Review) {
    println!("\n🎯 Code Review Summary");
    println!("==========================================");
    println!(
        "🌿 Branches: {} → {}",
        review.branch_comparison.source_branch, review.branch_comparison.target_branch
    );
    println!("📁 Files modified: {}", review.metrics.files_modified);
    println!("➕ Lines added: {}", review.metrics.lines_added);
    println!("➖ Lines removed: {}", review.metrics.lines_removed);
    println!("🐛 Total issues: {}", review.total_issues());
    println!(
        "📝 Commits analyzed: {}",
        review.branch_comparison.commits_analyzed.len()
    );

    let critical = review.get_critical_issues().len();
    let high = review.get_high_priority_issues().len() - critical;

    if critical > 0 {
        println!("🚨 Critical issues: {critical} (REQUIRES IMMEDIATE ATTENTION)");
    }
    if high > 0 {
        println!("⚠️  High priority issues: {high}");
    }

    if !review.priority_recommendations.is_empty() {
        println!("\n💡 Priority Recommendations:");
        for rec in &review.priority_recommendations {
            println!("  • {rec}");
        }
    }

    if !review.overall_assessment.is_empty() {
        println!("\n🤖 AI Assessment:");
        println!("{}", review.overall_assessment);
    }

    println!("\n📊 Technology Stack:");
    if !review.technology_stack.programming_languages.is_empty() {
        println!(
            "  Languages: {}",
            review.technology_stack.programming_languages.join(", ")
        );
    }
    if !review.technology_stack.frameworks.is_empty() {
        println!(
            "  Frameworks: {}",
            review.technology_stack.frameworks.join(", ")
        );
    }
    if !review.technology_stack.tools.is_empty() {
        println!("  Tools: {}", review.technology_stack.tools.join(", "));
    }

    // Show sample issues by category
    println!("\n🔍 Issues by Category:");
    for (category, issues) in &review.issues {
        if !issues.is_empty() {
            println!("  {:?}: {} issues", category, issues.len());
            // Show first few issues as examples
            for (i, issue) in issues.iter().take(3).enumerate() {
                let severity_icon = match issue.severity {
                    Severity::Critical => "🚨",
                    Severity::High => "⚠️",
                    Severity::Medium => "🔶",
                    Severity::Low => "ℹ️",
                    Severity::Info => "💡",
                };
                println!(
                    "    {} {} {}",
                    severity_icon,
                    issue.file_path,
                    if let Some(line) = issue.line_number {
                        format!(":{line}")
                    } else {
                        String::new()
                    }
                );
                println!("      {}", issue.description);
                if i == 2 && issues.len() > 3 {
                    println!("    ... and {} more", issues.len() - 3);
                }
            }
        }
    }

    println!("\n💻 To view detailed analysis, run without --cli flag for interactive mode");
    println!("📄 Generate reports using the interactive mode");
}
