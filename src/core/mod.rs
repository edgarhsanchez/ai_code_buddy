pub mod ai_analyzer;
pub mod analysis;
pub mod git;
pub mod review;

use crate::{
    args::{Args, OutputFormat},
    version::APP_VERSION,
};
use anyhow::Result;

pub fn run_cli_mode(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 AI Code Review Tool v{APP_VERSION} (CLI Mode)");
    println!("📂 Repository: {}", args.repo_path);
    println!(
        "🌿 Comparing: {} → {}",
        args.source_branch, args.target_branch
    );

    if args.show_credits {
        show_credits();
        return Ok(());
    }

    // Perform analysis
    let review_result = analysis::perform_analysis(&args)?;

    // Output results
    match args.output_format {
        OutputFormat::Summary => print_summary(&review_result),
        OutputFormat::Detailed => print_detailed(&review_result),
        OutputFormat::Json => print_json(&review_result)?,
        OutputFormat::Markdown => print_markdown(&review_result),
    }

    Ok(())
}

fn show_credits() {
    println!("🎉 AI Code Buddy v{APP_VERSION} - Credits & Contributors");
    println!("==========================================");
    println!();
    println!("📚 About AI Code Buddy:");
    println!("An intelligent code analysis tool with elegant Bevy-powered TUI");
    println!("that provides comprehensive code reviews with AI assistance.");
    println!();
    println!("🔧 Built with:");
    println!("  • Rust 🦀 - Systems programming language");
    println!("  • Bevy - Data-driven game engine for TUI");
    println!("  • Ratatui - Terminal UI library");
    println!("  • Git2 - Git repository analysis");
    println!("  • Kalosm - AI/ML framework");
    println!();
    println!("💡 Want to contribute? Visit: https://github.com/edgarhsanchez/ai_code_buddy");
}

fn print_summary(review: &review::Review) {
    println!("\n🎯 Code Review Summary");
    println!("==========================================");
    println!("📁 Files analyzed: {}", review.files_count);
    println!("🐛 Total issues: {}", review.issues_count);
    println!("⚠️  Severity breakdown:");
    println!("  🚨 Critical: {}", review.critical_issues);
    println!("  ⚠️  High: {}", review.high_issues);
    println!("  🔶 Medium: {}", review.medium_issues);
    println!("  ℹ️  Low: {}", review.low_issues);
}

fn print_detailed(review: &review::Review) {
    print_summary(review);
    println!("\n🔍 Detailed Analysis:");
    println!("==========================================");
    for issue in &review.issues {
        let severity_icon = match issue.severity.as_str() {
            "Critical" => "🚨",
            "High" => "⚠️",
            "Medium" => "🔶",
            "Low" => "ℹ️",
            _ => "💡",
        };
        let commit_icon = match issue.commit_status {
            review::CommitStatus::Committed => "✅",
            review::CommitStatus::Staged => "🟡",
            review::CommitStatus::Modified => "🔴",
            review::CommitStatus::Untracked => "🆕",
        };
        let status_text = match issue.commit_status {
            review::CommitStatus::Committed => "committed",
            review::CommitStatus::Staged => "staged",
            review::CommitStatus::Modified => "modified",
            review::CommitStatus::Untracked => "untracked",
        };
        println!(
            "{} {} {} (Line {}) [{}]: {}",
            severity_icon, commit_icon, issue.file, issue.line, status_text, issue.description
        );
    }
}

fn print_json(review: &review::Review) -> Result<()> {
    let json = serde_json::to_string_pretty(review)?;
    println!("{json}");
    Ok(())
}

fn print_markdown(review: &review::Review) {
    println!("# Code Review Report\n");
    println!("## Summary\n");
    println!("- **Files analyzed**: {}", review.files_count);
    println!("- **Total issues**: {}", review.issues_count);
    println!("- **Critical**: {}", review.critical_issues);
    println!("- **High**: {}", review.high_issues);
    println!("- **Medium**: {}", review.medium_issues);
    println!("- **Low**: {}", review.low_issues);
    println!("\n## Issues\n");
    for issue in &review.issues {
        let status_badge = match issue.commit_status {
            review::CommitStatus::Committed => {
                "![Committed](https://img.shields.io/badge/status-committed-green)"
            }
            review::CommitStatus::Staged => {
                "![Staged](https://img.shields.io/badge/status-staged-yellow)"
            }
            review::CommitStatus::Modified => {
                "![Modified](https://img.shields.io/badge/status-modified-red)"
            }
            review::CommitStatus::Untracked => {
                "![Untracked](https://img.shields.io/badge/status-untracked-blue)"
            }
        };
        println!(
            "- **{}:{}** - {} - {} {} - {}",
            issue.file, issue.line, issue.severity, status_badge, issue.category, issue.description
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::review::{CommitStatus, Issue, Review};

    fn sample_review() -> Review {
        Review {
            files_count: 1,
            issues_count: 1,
            critical_issues: 1,
            high_issues: 0,
            medium_issues: 0,
            low_issues: 0,
            issues: vec![Issue {
                file: "src/lib.rs".into(),
                line: 1,
                severity: "Critical".into(),
                category: "Security".into(),
                description: "test".into(),
                commit_status: CommitStatus::Committed,
            }],
        }
    }

    #[test]
    fn test_print_functions() {
        let r = sample_review();
        // Ensure these don't panic
        print_summary(&r);
        print_detailed(&r);
        print_markdown(&r);
        assert!(print_json(&r).is_ok());
    }

    #[test]
    fn test_show_credits() {
        show_credits();
    }
}
