mod code_analyzer;
mod git_analyzer;
mod review;
mod security_analysis;
mod ui_simple;

use code_analyzer::CodeAnalyzer;
use git_analyzer::GitAnalyzer;
use review::{Review, ReviewConfig, Severity};
use std::path::Path;
use ui_simple::run_tui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Parse command line arguments
    let use_cli = args.contains(&"--cli".to_string());

    // Filter out flags when getting positional arguments
    let pos_args: Vec<&String> = args.iter().filter(|arg| !arg.starts_with("--")).collect();

    let source_branch = pos_args.get(1).map(|s| s.as_str()).unwrap_or("main");
    let target_branch = pos_args.get(2).map(|s| s.as_str()).unwrap_or("main");
    let repo_path = pos_args.get(3).map(|s| s.as_str()).unwrap_or(".");

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
    // For now, generate a simple assessment based on the issues found
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
        "Excellent! No significant issues detected. Code appears to follow best practices and quality standards.".to_string()
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
