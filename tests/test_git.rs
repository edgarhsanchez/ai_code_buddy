use ai_code_buddy::core::git::GitAnalyzer;
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
    fs::write(format!("{}/README.md", repo_path), "# Test Repo").unwrap();
    
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
    fs::write(format!("{}/test.rs", repo_path), "fn main() {}").unwrap();
    
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
    fs::write(format!("{}/new_file.rs", repo_path), "fn test() {}").unwrap();
    
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
