use crate::args::Args;
use crate::core::{
    ai_analyzer::{AIAnalyzer, AnalysisRequest, ProgressUpdate},
    git::GitAnalyzer,
    review::Review,
};
use anyhow::Result;
use tokio::sync::mpsc;

pub async fn perform_analysis_with_progress(
    args: &Args,
    progress_callback: Option<Box<dyn Fn(f64, String) + Send + Sync>>,
) -> Result<Review> {
    println!("📊 Starting AI-powered analysis...");

    let git_analyzer = GitAnalyzer::new(&args.repo_path)?;

    // Get changed files between branches
    let changed_files = git_analyzer.get_changed_files(&args.source_branch, &args.target_branch)?;

    println!("📈 Found {} changed files", changed_files.len());

    let mut review = Review {
        files_count: changed_files.len(),
        issues_count: 0,
        critical_issues: 0,
        high_issues: 0,
        medium_issues: 0,
        low_issues: 0,
        issues: Vec::new(),
    };

    // Initialize AI analyzer
    let use_gpu = args.use_gpu && !args.force_cpu;
    if args.force_cpu {
        println!("💻 CPU mode forced by user with --cpu flag");
    } else if args.use_gpu {
        println!("🚀 GPU acceleration enabled (auto-detected or requested)");
    }
    let ai_analyzer = AIAnalyzer::new(use_gpu).await?;

    // Create progress channel
    let (progress_tx, mut progress_rx) = mpsc::unbounded_channel::<ProgressUpdate>();

    // Spawn task to handle progress updates
    if let Some(callback) = progress_callback {
        tokio::spawn(async move {
            while let Some(update) = progress_rx.recv().await {
                // Format the current file with stage information
                let status_message = if update.stage.is_empty() {
                    update.current_file
                } else {
                    format!("{} - {}", update.current_file, update.stage)
                };
                callback(update.progress, status_message);
            }
        });
    }

    // Analyze each file
    let total_files = changed_files.len() as f64;
    for (index, file_path) in changed_files.iter().enumerate() {
        if should_analyze_file(file_path, args) {
            let commit_status = git_analyzer
                .get_file_status(file_path)
                .unwrap_or(crate::core::review::CommitStatus::Committed);

            let status_indicator = match commit_status {
                crate::core::review::CommitStatus::Committed => "📄",
                crate::core::review::CommitStatus::Staged => "📑",
                crate::core::review::CommitStatus::Modified => "📝",
                crate::core::review::CommitStatus::Untracked => "📄",
            };

            let file_progress = (index as f64 / total_files) * 100.0;
            println!(
                "  {status_indicator} Analyzing: {file_path} ({commit_status:?}) [{file_progress:.1}%]"
            );

            if let Ok(content) = git_analyzer.get_file_content(file_path, &args.target_branch) {
                let request = AnalysisRequest {
                    file_path: file_path.clone(),
                    content,
                    language: detect_language(file_path),
                    commit_status,
                };

                match ai_analyzer
                    .analyze_file(request, Some(progress_tx.clone()))
                    .await
                {
                    Ok(file_issues) => {
                        for issue in file_issues {
                            match issue.severity.as_str() {
                                "Critical" => review.critical_issues += 1,
                                "High" => review.high_issues += 1,
                                "Medium" => review.medium_issues += 1,
                                "Low" => review.low_issues += 1,
                                _ => {}
                            }
                            review.issues.push(issue);
                            review.issues_count += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to analyze {file_path}: {e}");
                    }
                }
            }
        }
    }

    // Close progress channel
    drop(progress_tx);

    println!(
        "✅ AI analysis complete! Found {} issues.",
        review.issues_count
    );
    Ok(review)
}

pub fn perform_analysis(args: &Args) -> Result<Review> {
    // Create a simple runtime for synchronous callers
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(perform_analysis_with_progress(args, None))
}

fn should_analyze_file(file_path: &str, args: &Args) -> bool {
    // Check include patterns
    if !args.include_patterns.is_empty() {
        let matches_include = args
            .include_patterns
            .iter()
            .any(|pattern| file_matches_pattern(file_path, pattern));
        if !matches_include {
            return false;
        }
    }

    // Check exclude patterns
    for pattern in &args.exclude_patterns {
        if file_matches_pattern(file_path, pattern) {
            return false;
        }
    }

    // Default exclusions
    if file_path.starts_with("target/")
        || file_path.contains("node_modules/")
        || file_path.ends_with(".lock")
        || file_path.ends_with(".log")
    {
        return false;
    }

    true
}

fn file_matches_pattern(file_path: &str, pattern: &str) -> bool {
    // Simple pattern matching - can be enhanced with glob
    if pattern.starts_with("*.") {
        let extension = &pattern[1..];
        file_path.ends_with(extension)
    } else if let Some(prefix) = pattern.strip_suffix("/**") {
        file_path.starts_with(prefix)
    } else {
        file_path.contains(pattern)
    }
}

fn detect_language(file_path: &str) -> String {
    use std::path::Path;
    let path = Path::new(file_path);
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("rs") => "rust".to_string(),
        Some("js") => "javascript".to_string(),
        Some("ts") => "typescript".to_string(),
        Some("py") => "python".to_string(),
        Some("java") => "java".to_string(),
        Some("cpp") | Some("cc") | Some("cxx") => "cpp".to_string(),
        Some("c") => "c".to_string(),
        Some("go") => "go".to_string(),
        Some("php") => "php".to_string(),
        Some("rb") => "ruby".to_string(),
        Some("cs") => "csharp".to_string(),
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_args(include: Vec<&str>, exclude: Vec<&str>) -> Args {
        Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "HEAD".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: crate::args::OutputFormat::Summary,
            include_patterns: include.into_iter().map(|s| s.to_string()).collect(),
            exclude_patterns: exclude.into_iter().map(|s| s.to_string()).collect(),
            use_gpu: false,
            force_cpu: true,
        }
    }

    #[test]
    fn test_file_matches_pattern_variants() {
        assert!(file_matches_pattern("src/lib.rs", "*.rs"));
        assert!(file_matches_pattern("src/core/mod.rs", "src/**"));
        assert!(file_matches_pattern("foo/bar/baz.txt", "bar"));
        assert!(!file_matches_pattern("src/lib.rs", "*.py"));
    }

    #[test]
    fn test_should_analyze_file_include_exclude() {
        // Include only rs
        let args = mk_args(vec!["*.rs"], vec![]);
        assert!(should_analyze_file("src/lib.rs", &args));
        assert!(!should_analyze_file("src/app.py", &args));

        // Exclude target and logs by default
        let args2 = mk_args(vec![], vec![]);
        assert!(!should_analyze_file("target/debug/build.rs", &args2));
        assert!(!should_analyze_file("foo/node_modules/pkg/index.js", &args2));
        assert!(!should_analyze_file("foo/app.log", &args2));
        assert!(should_analyze_file("src/main.rs", &args2));

        // Explicit exclude wins
        let args3 = mk_args(vec![], vec!["*.rs"]);
        assert!(!should_analyze_file("src/lib.rs", &args3));
    }

    #[test]
    fn test_detect_language_extensions() {
        assert_eq!(detect_language("a.rs"), "rust");
        assert_eq!(detect_language("a.js"), "javascript");
        assert_eq!(detect_language("a.ts"), "typescript");
        assert_eq!(detect_language("a.py"), "python");
        assert_eq!(detect_language("a.unknown"), "unknown");
    }
}
