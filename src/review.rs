use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum IssueCategory {
    Security,
    Performance,
    Maintainability,
    Readability,
    Testing,
    Documentation,
    Architecture,
    Style,
    BestPractices,
    PotentialBugs,
}

#[derive(Clone, Debug)]
pub struct CodeIssue {
    pub category: IssueCategory,
    pub severity: Severity,
    pub description: String,
    pub file_path: String,
    pub line_number: Option<u32>,
    pub suggestion: String,
    pub code_snippet: Option<String>,
}

#[derive(Clone, Debug)]
pub struct CommitInfo {
    pub hash: String,
    pub author: String,
    pub message: String,
    pub timestamp: String,
    pub files_changed: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct TechnologyStack {
    pub programming_languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub tools: Vec<String>,
    pub databases: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct BranchComparison {
    pub source_branch: String,
    pub target_branch: String,
    pub commits_analyzed: Vec<CommitInfo>,
}

#[derive(Clone, Debug)]
pub struct CodeMetrics {
    pub files_modified: u32,
    pub lines_added: u32,
    pub lines_removed: u32,
    pub complexity_score: Option<f32>,
    pub test_coverage: Option<f32>,
}

#[derive(Clone, Debug)]
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
        self.issues.entry(issue.category.clone()).or_insert_with(Vec::new).push(issue);
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
    pub source_branch: String,
    pub target_branch: String,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
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
