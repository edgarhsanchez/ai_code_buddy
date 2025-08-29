use ai_code_buddy::core::analysis::perform_analysis_with_progress;
use ai_code_buddy::{Args, OutputFormat};
use std::process::Command;
use std::{
    fs,
    sync::{Arc, Mutex},
};
use tempfile::TempDir;

// Helper to run a git command in a repo dir
fn git(repo: &str, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo)
        .status()
        .expect("failed to run git");
    assert!(status.success(), "git {args:?} failed");
}

fn setup_repo_with_branches() -> (TempDir, String) {
    let temp = TempDir::new().unwrap();
    let repo = temp.path().to_str().unwrap().to_string();

    // init and configure
    git(&repo, &["init"]);
    git(&repo, &["checkout", "-b", "main"]);
    git(&repo, &["config", "user.name", "Test User"]);
    git(&repo, &["config", "user.email", "test@example.com"]);

    // initial files and commit on main (include a shared file we'll later modify)
    fs::write(
        format!("{repo}/file.rs"),
        "fn main() { println!(\"hello\"); }\n",
    )
    .unwrap();
    fs::write(
        format!("{repo}/shared.rs"),
        "pub fn shared() { println!(\"base\"); }\n",
    )
    .unwrap();
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-m", "init"]);

    // branch feature from main
    git(&repo, &["checkout", "-b", "feature"]);

    // Modify committed file to create a diff vs main
    fs::write(
        format!("{repo}/file.rs"),
        "fn main() { let password = \"x\"; println!(\"changed\"); }\n",
    )
    .unwrap();
    git(&repo, &["add", "file.rs"]);
    git(
        &repo,
        &["commit", "-m", "change file to include security issue"],
    );

    // Create a staged file (in index only)
    fs::write(
        format!("{repo}/staged.rs"),
        "pub fn staged() { unsafe { /* risky */ } }\n",
    )
    .unwrap();
    git(&repo, &["add", "staged.rs"]);

    // Create an untracked file (working tree only)
    fs::write(
        format!("{repo}/untracked.rs"),
        "pub fn untracked() { eval(\"2+2\"); }\n",
    )
    .unwrap();

    // Modify a tracked file without staging to create WT_MODIFIED
    fs::write(
        format!("{repo}/shared.rs"),
        "pub fn shared() { let _x = 1; unwrap_me().unwrap(); }\n",
    )
    .unwrap();

    (temp, repo)
}

#[test]
fn test_perform_analysis_with_progress_covers_paths() {
    let (_temp, repo) = setup_repo_with_branches();

    let args = Args {
        repo_path: repo.clone(),
        source_branch: "main".to_string(),
        target_branch: "HEAD".to_string(), // feature HEAD
        cli_mode: true,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec!["*.rs".to_string()], // ensure we include our files
        use_gpu: false,
        force_cpu: true,
        parallel: false,
    };

    // Capture progress updates
    let progress: Arc<Mutex<Vec<(f64, String)>>> = Arc::new(Mutex::new(Vec::new()));
    let progress_clone = progress.clone();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let review = rt
        .block_on(perform_analysis_with_progress(
            &args,
            Some(Box::new(move |p, s| {
                progress_clone.lock().unwrap().push((p, s));
            })),
        ))
        .expect("analysis should succeed on prepared repo");

    // We should have analyzed our committed diff + staged + untracked files and found issues
    assert!(review.issues_count >= 1, "expected at least one issue");
    // Ensure severity counters add up
    assert_eq!(
        review.issues_count,
        review.critical_issues + review.high_issues + review.medium_issues + review.low_issues
    );

    // Progress callback should have been invoked at least once
    let calls = progress.lock().unwrap();
    assert!(!calls.is_empty(), "expected progress updates");
}
