use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Review {
    pub files_count: usize,
    pub issues_count: usize,
    pub critical_issues: usize,
    pub high_issues: usize,
    pub medium_issues: usize,
    pub low_issues: usize,
    pub issues: Vec<Issue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Issue {
    pub file: String,
    pub line: u32,
    pub severity: String,
    pub category: String,
    pub description: String,
    pub commit_status: CommitStatus,
    /// The actual line(s) of code that triggered this issue
    pub code_snippet: String,
    /// Optional: Additional context lines around the issue
    pub context_lines: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitStatus {
    Committed,
    Staged,
    Modified,
    Untracked,
}
