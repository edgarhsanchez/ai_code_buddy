use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    #[allow(dead_code)]
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IssueCategory {
    Security,
    Performance,
    Maintainability,
    Readability,
    Testing,
    Documentation,
    #[allow(dead_code)]
    Architecture,
    Style,
    #[allow(dead_code)]
    BestPractices,
    PotentialBugs,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeIssue {
    pub category: IssueCategory,
    pub severity: Severity,
    pub description: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub suggestion: String,
    pub code_snippet: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    #[allow(dead_code)]
    pub hash: String,
    #[allow(dead_code)]
    pub author: String,
    #[allow(dead_code)]
    pub message: String,
    #[allow(dead_code)]
    pub timestamp: String,
    pub files_changed: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TechnologyStack {
    pub programming_languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub tools: Vec<String>,
    #[allow(dead_code)]
    pub databases: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BranchComparison {
    pub source_branch: String,
    pub target_branch: String,
    pub commits_analyzed: Vec<CommitInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub files_modified: u32,
    pub lines_added: u32,
    pub lines_removed: u32,
    #[allow(dead_code)]
    pub complexity_score: Option<f32>,
    #[allow(dead_code)]
    pub test_coverage: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Review {
    pub branch_comparison: BranchComparison,
    pub metrics: CodeMetrics,
    pub technology_stack: TechnologyStack,
    pub issues: HashMap<IssueCategory, Vec<CodeIssue>>,
    pub overall_assessment: String,
    pub priority_recommendations: Vec<String>,
    pub timestamp: String,
}

impl Review {
    pub fn add_issue(&mut self, issue: CodeIssue) {
        self.issues
            .entry(issue.category.clone())
            .or_default()
            .push(issue);
    }

    pub fn total_issues(&self) -> usize {
        self.issues.values().map(|v| v.len()).sum()
    }

    pub fn get_critical_issues(&self) -> Vec<&CodeIssue> {
        self.issues
            .values()
            .flatten()
            .filter(|issue| matches!(issue.severity, Severity::Critical))
            .collect()
    }

    pub fn get_high_priority_issues(&self) -> Vec<&CodeIssue> {
        self.issues
            .values()
            .flatten()
            .filter(|issue| matches!(issue.severity, Severity::High | Severity::Critical))
            .collect()
    }
}

impl Default for Review {
    fn default() -> Self {
        Self {
            branch_comparison: BranchComparison {
                source_branch: "feature".to_string(),
                target_branch: "develop".to_string(),
                commits_analyzed: Vec::new(),
            },
            metrics: CodeMetrics {
                files_modified: 0,
                lines_added: 0,
                lines_removed: 0,
                complexity_score: None,
                test_coverage: None,
            },
            technology_stack: TechnologyStack {
                programming_languages: Vec::new(),
                frameworks: Vec::new(),
                tools: Vec::new(),
                databases: Vec::new(),
            },
            issues: HashMap::new(),
            overall_assessment: String::new(),
            priority_recommendations: Vec::new(),
            timestamp: String::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ReviewConfig {
    #[allow(dead_code)]
    pub source_branch: String,
    #[allow(dead_code)]
    pub target_branch: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    #[allow(dead_code)]
    pub max_issues_per_category: usize,
}

impl Default for ReviewConfig {
    fn default() -> Self {
        Self {
            source_branch: "feature".to_string(),
            target_branch: "develop".to_string(),
            include_patterns: vec!["*.rs".to_string()],
            exclude_patterns: vec!["target/**".to_string()],
            max_issues_per_category: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_creation() {
        let review = Review::default();
        assert_eq!(review.issues.len(), 0);
        assert_eq!(review.total_issues(), 0);
    }

    #[test]
    fn test_add_issue() {
        let mut review = Review::default();

        let issue = CodeIssue {
            category: IssueCategory::Security,
            severity: Severity::High,
            description: "Test issue".to_string(),
            file_path: "test.rs".to_string(),
            line_number: Some(10),
            suggestion: "Fix this".to_string(),
            code_snippet: Some("let x = 1;".to_string()),
        };

        review.add_issue(issue);
        assert_eq!(review.total_issues(), 1);
    }

    #[test]
    fn test_severity_ordering() {
        use std::cmp::Ordering;

        assert_eq!(Severity::Critical.cmp(&Severity::High), Ordering::Less);
        assert_eq!(Severity::High.cmp(&Severity::Medium), Ordering::Less);
        assert_eq!(Severity::Medium.cmp(&Severity::Low), Ordering::Less);
        assert_eq!(Severity::Low.cmp(&Severity::Info), Ordering::Less);
    }

    #[test]
    fn test_get_critical_issues() {
        let mut review = Review::default();

        let critical_issue = CodeIssue {
            category: IssueCategory::Security,
            severity: Severity::Critical,
            description: "Critical issue".to_string(),
            file_path: "test.rs".to_string(),
            line_number: Some(10),
            suggestion: "Fix immediately".to_string(),
            code_snippet: None,
        };

        let low_issue = CodeIssue {
            category: IssueCategory::Style,
            severity: Severity::Low,
            description: "Style issue".to_string(),
            file_path: "test.rs".to_string(),
            line_number: Some(20),
            suggestion: "Consider fixing".to_string(),
            code_snippet: None,
        };

        review.add_issue(critical_issue);
        review.add_issue(low_issue);

        let critical_issues = review.get_critical_issues();
        assert_eq!(critical_issues.len(), 1);
        assert_eq!(critical_issues[0].severity, Severity::Critical);
    }
}
