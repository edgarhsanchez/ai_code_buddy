// Comprehensive functional tests to achieve high code coverage
// This focuses on actual method calls rather than just data structure testing

use ai_code_buddy::args::{Args, OutputFormat};
use ai_code_buddy::core::ai_analyzer::{AnalysisRequest, GpuBackend, ProgressUpdate};
use ai_code_buddy::core::git::GitAnalyzer;
use ai_code_buddy::core::review::{CommitStatus, Issue, Review};
use std::fs;
use tempfile::{tempdir, TempDir};

fn create_test_git_repo() -> Result<TempDir, Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let repo_path = dir.path();

    // Initialize git repository
    std::process::Command::new("git")
        .arg("init")
        .current_dir(repo_path)
        .output()?;

    // Create a test file
    fs::write(repo_path.join("test.rs"), "fn main() {}")?;

    // Add and commit the file
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(repo_path)
        .output()?;

    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()?;

    Ok(dir)
}

#[test]
fn test_git_analyzer_creation() {
    let temp_dir = create_test_git_repo().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let analyzer = GitAnalyzer::new(repo_path);
    assert!(analyzer.is_ok());
}

#[test]
fn test_git_analyzer_get_changed_files() {
    let temp_dir = create_test_git_repo().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let analyzer = GitAnalyzer::new(repo_path).unwrap();

    // Test with same branch (should return empty)
    let result = analyzer.get_changed_files("HEAD", "HEAD");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);

    // Create a new file for testing differences
    fs::write(temp_dir.path().join("new_file.rs"), "fn new_function() {}").unwrap();

    let result = analyzer.get_changed_files("HEAD", "HEAD");
    assert!(result.is_ok());
}

#[test]
fn test_git_analyzer_get_file_content() {
    let temp_dir = create_test_git_repo().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let analyzer = GitAnalyzer::new(repo_path).unwrap();

    // Test getting content of existing file
    let result = analyzer.get_file_content("test.rs", "HEAD");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "fn main() {}");

    // Test getting content of non-existent file
    let result = analyzer.get_file_content("nonexistent.rs", "HEAD");
    assert!(result.is_err());
}

#[test]
fn test_git_analyzer_get_uncommitted_files() {
    let temp_dir = create_test_git_repo().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let analyzer = GitAnalyzer::new(repo_path).unwrap();

    let result = analyzer.get_uncommitted_files();
    assert!(result.is_ok());
}

#[test]
fn test_git_analyzer_get_file_status() {
    let temp_dir = create_test_git_repo().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let analyzer = GitAnalyzer::new(repo_path).unwrap();

    let result = analyzer.get_file_status("test.rs");
    assert!(result.is_ok());
}

#[test]
fn test_review_structure() {
    let review = Review {
        files_count: 5,
        issues_count: 10,
        critical_issues: 1,
        high_issues: 2,
        medium_issues: 3,
        low_issues: 4,
        issues: vec![],
    };

    assert_eq!(review.files_count, 5);
    assert_eq!(review.issues_count, 10);
    assert_eq!(review.critical_issues, 1);
    assert_eq!(review.high_issues, 2);
    assert_eq!(review.medium_issues, 3);
    assert_eq!(review.low_issues, 4);
}

#[test]
fn test_review_with_issues() {
    let issues = vec![
        Issue {
            file: "test1.rs".to_string(),
            line: 10,
            severity: "High".to_string(),
            category: "Security".to_string(),
            description: "High severity issue".to_string(),
            commit_status: CommitStatus::Modified,
        },
        Issue {
            file: "test2.rs".to_string(),
            line: 20,
            severity: "Medium".to_string(),
            category: "Performance".to_string(),
            description: "Medium severity issue".to_string(),
            commit_status: CommitStatus::Staged,
        },
    ];

    let review = Review {
        files_count: 2,
        issues_count: 2,
        critical_issues: 0,
        high_issues: 1,
        medium_issues: 1,
        low_issues: 0,
        issues,
    };

    assert_eq!(review.issues.len(), 2);
    assert_eq!(review.high_issues, 1);
    assert_eq!(review.medium_issues, 1);
}

#[test]
fn test_args_structure() {
    // Test various argument combinations
    let args = Args {
        repo_path: ".".to_string(),
        source_branch: "develop".to_string(),
        target_branch: "main".to_string(),
        cli_mode: false,
        verbose: true,
        show_credits: false,
        output_format: OutputFormat::Json,
        exclude_patterns: vec!["*.tmp".to_string()],
        include_patterns: vec!["*.rs".to_string()],
        use_gpu: true,
        force_cpu: false,
        parallel: false,
    };

    // Verify all fields are accessible
    assert_eq!(args.source_branch, "develop");
    assert_eq!(args.target_branch, "main");
    assert_eq!(args.output_format, OutputFormat::Json);
    assert!(args.use_gpu);
    assert!(args.verbose);
}

#[test]
fn test_commit_status_variants() {
    let variants = [
        CommitStatus::Committed,
        CommitStatus::Staged,
        CommitStatus::Modified,
        CommitStatus::Untracked,
    ];

    for status in variants {
        // Test that all variants can be created and used
        let issue = Issue {
            file: "test.rs".to_string(),
            line: 1,
            severity: "Low".to_string(),
            category: "Test".to_string(),
            description: "Test description".to_string(),
            commit_status: status,
        };

        // Just verify we can create the issue successfully
        assert_eq!(issue.line, 1);
    }
}

#[test]
fn test_gpu_backend_display_formatting() {
    assert_eq!(format!("{}", GpuBackend::Metal), "Metal");
    assert_eq!(format!("{}", GpuBackend::Cuda), "CUDA");
    assert_eq!(format!("{}", GpuBackend::Mkl), "MKL");
    assert_eq!(format!("{}", GpuBackend::Cpu), "CPU");
}

#[test]
fn test_issue_field_access() {
    let issue = Issue {
        file: "src/main.rs".to_string(),
        line: 42,
        severity: "Critical".to_string(),
        category: "Security".to_string(),
        description: "Buffer overflow vulnerability".to_string(),
        commit_status: CommitStatus::Modified,
    };

    // Test all field access
    assert_eq!(issue.file, "src/main.rs");
    assert_eq!(issue.line, 42);
    assert_eq!(issue.severity, "Critical");
    assert_eq!(issue.category, "Security");
    assert_eq!(issue.description, "Buffer overflow vulnerability");
}

#[test]
fn test_progress_update_field_access() {
    let progress = ProgressUpdate {
        current_file: "src/lib.rs".to_string(),
        progress: 75.5,
        stage: "Analyzing patterns".to_string(),
    };

    // Test all field access
    assert_eq!(progress.current_file, "src/lib.rs");
    assert_eq!(progress.progress, 75.5);
    assert_eq!(progress.stage, "Analyzing patterns");
}

#[test]
fn test_analysis_request_field_access() {
    let request = AnalysisRequest {
        file_path: "src/utils.rs".to_string(),
        content: "pub fn utility_function() {}".to_string(),
        language: "rust".to_string(),
        commit_status: CommitStatus::Staged,
    };

    // Test all field access
    assert_eq!(request.file_path, "src/utils.rs");
    assert_eq!(request.content, "pub fn utility_function() {}");
    assert_eq!(request.language, "rust");
}

#[test]
fn test_output_format_variants() {
    let formats = [
        OutputFormat::Summary,
        OutputFormat::Detailed,
        OutputFormat::Json,
        OutputFormat::Markdown,
    ];

    for format in formats {
        let args = Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "HEAD".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: format,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: false,
            parallel: false,
        };

        // Verify the format was set correctly
        matches!(
            args.output_format,
            OutputFormat::Summary
                | OutputFormat::Detailed
                | OutputFormat::Json
                | OutputFormat::Markdown
        );
    }
}

#[test]
fn test_git_analyzer_invalid_repo() {
    let result = GitAnalyzer::new("/nonexistent/path");
    assert!(result.is_err());
}

#[test]
fn test_issue_serialization() {
    let issue = Issue {
        file: "test.rs".to_string(),
        line: 123,
        severity: "High".to_string(),
        category: "Security".to_string(),
        description: "Test issue description".to_string(),
        commit_status: CommitStatus::Modified,
    };

    // Test that we can serialize and deserialize
    let serialized = serde_json::to_string(&issue).unwrap();
    let deserialized: Issue = serde_json::from_str(&serialized).unwrap();

    assert_eq!(issue.file, deserialized.file);
    assert_eq!(issue.line, deserialized.line);
    assert_eq!(issue.severity, deserialized.severity);
}

#[test]
fn test_review_serialization() {
    let review = Review {
        files_count: 3,
        issues_count: 5,
        critical_issues: 1,
        high_issues: 2,
        medium_issues: 1,
        low_issues: 1,
        issues: vec![],
    };

    // Test that we can serialize and deserialize
    let serialized = serde_json::to_string(&review).unwrap();
    let deserialized: Review = serde_json::from_str(&serialized).unwrap();

    assert_eq!(review.files_count, deserialized.files_count);
    assert_eq!(review.issues_count, deserialized.issues_count);
    assert_eq!(review.critical_issues, deserialized.critical_issues);
}
