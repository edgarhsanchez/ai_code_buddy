use ai_code_buddy::core::git::GitAnalyzer;
use ai_code_buddy::core::review::CommitStatus;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn create_test_repo() -> (TempDir, String) {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    // Initialize git repo
    Command::new("git")
        .args(["init"])
        .current_dir(&repo_path)
        .output()
        .expect("Failed to init git repo");

    // Configure git
    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Create initial commit
    fs::write(format!("{repo_path}/README.md"), "# Test Repo").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    (temp_dir, repo_path)
}

#[test]
fn test_git_analyzer_new_valid_repo() {
    let (_temp_dir, repo_path) = create_test_repo();
    let result = GitAnalyzer::new(&repo_path);
    assert!(result.is_ok());
}

#[test]
fn test_git_analyzer_new_invalid_repo() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let result = GitAnalyzer::new(repo_path);
    assert!(result.is_err());
}

#[test]
fn test_get_changed_files_same_branch() {
    let (_temp_dir, repo_path) = create_test_repo();
    let analyzer = GitAnalyzer::new(&repo_path).unwrap();

    let result = analyzer.get_changed_files("HEAD", "HEAD");
    assert!(result.is_ok());
    let files = result.unwrap();
    assert!(files.is_empty()); // No changes between same commit
}

#[test]
fn test_get_changed_files_with_changes() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Create a new file
    fs::write(format!("{repo_path}/test.rs"), "fn main() {}").unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", "Add test file"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let analyzer = GitAnalyzer::new(&repo_path).unwrap();
    // Compare the current working tree to HEAD to see uncommitted changes
    // First create an uncommitted change
    fs::write(format!("{repo_path}/new_file.rs"), "fn test() {}").unwrap();

    let result = analyzer.get_uncommitted_files();

    assert!(result.is_ok());
    let files = result.unwrap();
    assert!(!files.is_empty()); // Should have uncommitted changes
}

#[test]
fn test_get_file_content() {
    let (_temp_dir, repo_path) = create_test_repo();
    let analyzer = GitAnalyzer::new(&repo_path).unwrap();

    let content = analyzer.get_file_content("README.md", "HEAD");
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "# Test Repo");
}

#[test]
fn test_get_file_content_nonexistent() {
    let (_temp_dir, repo_path) = create_test_repo();
    let analyzer = GitAnalyzer::new(&repo_path).unwrap();

    let content = analyzer.get_file_content("nonexistent.rs", "HEAD");
    assert!(content.is_err());
}

#[test]
fn test_get_file_content_staged_file() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Create and stage a file
    fs::write(format!("{repo_path}/staged.rs"), "fn staged() {}").unwrap();

    Command::new("git")
        .args(["add", "staged.rs"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let analyzer = GitAnalyzer::new(&repo_path).unwrap();
    let content = analyzer.get_file_content("staged.rs", "HEAD");
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "fn staged() {}");
}

#[test]
fn test_get_file_content_modified_file() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Create and commit a file
    fs::write(format!("{repo_path}/modified.rs"), "fn original() {}").unwrap();
    Command::new("git")
        .args(["add", "modified.rs"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Add modified file"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Modify the file
    fs::write(format!("{repo_path}/modified.rs"), "fn modified() {}").unwrap();

    let analyzer = GitAnalyzer::new(&repo_path).unwrap();
    let content = analyzer.get_file_content("modified.rs", "HEAD");
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "fn modified() {}");
}

#[test]
fn test_get_file_status_staged() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Create and stage a file
    fs::write(format!("{repo_path}/staged_status.rs"), "fn test() {}").unwrap();
    Command::new("git")
        .args(["add", "staged_status.rs"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    let analyzer = GitAnalyzer::new(&repo_path).unwrap();
    let status = analyzer.get_file_status("staged_status.rs");
    assert!(status.is_ok());
    assert!(matches!(status.unwrap(), CommitStatus::Staged));
}

#[test]
fn test_get_file_status_modified() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Create and commit a file
    fs::write(format!("{repo_path}/modified_status.rs"), "fn test() {}").unwrap();
    Command::new("git")
        .args(["add", "modified_status.rs"])
        .current_dir(&repo_path)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "Add file"])
        .current_dir(&repo_path)
        .output()
        .unwrap();

    // Modify the file
    fs::write(format!("{repo_path}/modified_status.rs"), "fn modified() {}").unwrap();

    let analyzer = GitAnalyzer::new(&repo_path).unwrap();
    let status = analyzer.get_file_status("modified_status.rs");
    assert!(status.is_ok());
    assert!(matches!(status.unwrap(), CommitStatus::Modified));
}

#[test]
fn test_get_file_status_untracked() {
    let (_temp_dir, repo_path) = create_test_repo();

    // Create an untracked file
    fs::write(format!("{repo_path}/untracked.rs"), "fn test() {}").unwrap();

    let analyzer = GitAnalyzer::new(&repo_path).unwrap();
    let status = analyzer.get_file_status("untracked.rs");
    assert!(status.is_ok());
    assert!(matches!(status.unwrap(), CommitStatus::Untracked));
}

#[test]
fn test_get_file_status_committed() {
    let (_temp_dir, repo_path) = create_test_repo();

    let analyzer = GitAnalyzer::new(&repo_path).unwrap();
    let status = analyzer.get_file_status("README.md");
    assert!(status.is_ok());
    assert!(matches!(status.unwrap(), CommitStatus::Committed));
}
