use ai_code_buddy::core::review::{CommitStatus, Issue, Review};

#[test]
fn test_review_new() {
    let review = Review {
        files_count: 5,
        issues_count: 3,
        critical_issues: 1,
        high_issues: 1,
        medium_issues: 1,
        low_issues: 0,
        issues: vec![],
    };

    assert_eq!(review.files_count, 5);
    assert_eq!(review.issues_count, 3);
    assert_eq!(review.critical_issues, 1);
    assert_eq!(review.high_issues, 1);
    assert_eq!(review.medium_issues, 1);
    assert_eq!(review.low_issues, 0);
    assert!(review.issues.is_empty());
}

#[test]
fn test_issue_creation() {
    let issue = Issue {
        file: "src/main.rs".to_string(),
        line: 42,
        severity: "High".to_string(),
        category: "Security".to_string(),
        description: "Potential SQL injection vulnerability".to_string(),
        commit_status: CommitStatus::Modified,
    };

    assert_eq!(issue.file, "src/main.rs");
    assert_eq!(issue.line, 42);
    assert_eq!(issue.severity, "High");
    assert_eq!(issue.category, "Security");
    assert_eq!(issue.description, "Potential SQL injection vulnerability");
    assert!(matches!(issue.commit_status, CommitStatus::Modified));
}

#[test]
fn test_commit_status_variants() {
    let statuses = [
        CommitStatus::Committed,
        CommitStatus::Staged,
        CommitStatus::Modified,
        CommitStatus::Untracked,
    ];

    assert_eq!(statuses.len(), 4);
}

#[test]
fn test_review_with_issues() {
    let issues = vec![
        Issue {
            file: "src/lib.rs".to_string(),
            line: 10,
            severity: "Critical".to_string(),
            category: "Security".to_string(),
            description: "Buffer overflow detected".to_string(),
            commit_status: CommitStatus::Staged,
        },
        Issue {
            file: "src/utils.rs".to_string(),
            line: 25,
            severity: "Medium".to_string(),
            category: "Performance".to_string(),
            description: "Inefficient algorithm detected".to_string(),
            commit_status: CommitStatus::Modified,
        },
    ];

    let review = Review {
        files_count: 2,
        issues_count: 2,
        critical_issues: 1,
        high_issues: 0,
        medium_issues: 1,
        low_issues: 0,
        issues: issues.clone(),
    };

    assert_eq!(review.issues.len(), 2);
    assert_eq!(review.issues[0].severity, "Critical");
    assert_eq!(review.issues[1].severity, "Medium");
}

#[test]
fn test_review_serialization() {
    let review = Review {
        files_count: 1,
        issues_count: 1,
        critical_issues: 0,
        high_issues: 1,
        medium_issues: 0,
        low_issues: 0,
        issues: vec![Issue {
            file: "test.rs".to_string(),
            line: 5,
            severity: "High".to_string(),
            category: "Bug".to_string(),
            description: "Possible null pointer dereference".to_string(),
            commit_status: CommitStatus::Committed,
        }],
    };

    let json = serde_json::to_string(&review).unwrap();
    assert!(json.contains("\"files_count\":1"));
    assert!(json.contains("\"test.rs\""));

    let deserialized: Review = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.files_count, review.files_count);
    assert_eq!(deserialized.issues.len(), review.issues.len());
}
