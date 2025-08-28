// Focused tests for Reports Widget State functionality
// Target: 100% coverage of src/widget_states/reports.rs (0/74 lines currently covered)

use ai_code_buddy::core::review::{CommitStatus, Issue, Review};
use ai_code_buddy::widget_states::reports::{
    ExportStatus, ReportFormat, ReportsWidgetState, ViewMode,
};

// Helper function to create a test review with issues
fn create_test_review() -> Review {
    Review {
        files_count: 5,
        issues_count: 10,
        critical_issues: 2,
        high_issues: 3,
        medium_issues: 3,
        low_issues: 2,
        issues: vec![
            Issue {
                file: "src/main.rs".to_string(),
                line: 42,
                category: "Security".to_string(),
                severity: "Critical".to_string(),
                description: "Potential buffer overflow vulnerability".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                file: "src/utils.rs".to_string(),
                line: 15,
                category: "Performance".to_string(),
                severity: "High".to_string(),
                description: "Inefficient algorithm detected".to_string(),
                commit_status: CommitStatus::Staged,
            },
            Issue {
                file: "src/config.rs".to_string(),
                line: 28,
                category: "Code Quality".to_string(),
                severity: "Medium".to_string(),
                description: "Consider using more descriptive variable names".to_string(),
                commit_status: CommitStatus::Committed,
            },
            Issue {
                file: "src/helpers.rs".to_string(),
                line: 7,
                category: "Style".to_string(),
                severity: "Low".to_string(),
                description: "Missing documentation for public function".to_string(),
                commit_status: CommitStatus::Untracked,
            },
        ],
    }
}

#[test]
fn test_reports_widget_state_default() {
    let state = ReportsWidgetState::default();

    assert!(state.review.is_none());
    assert_eq!(state.selected_format, ReportFormat::Summary);
    assert!(matches!(state.export_status, ExportStatus::None));
    assert!(state.generated_report.is_none());
    assert_eq!(state.view_mode, ViewMode::Selection);
}

#[test]
fn test_set_review() {
    let mut state = ReportsWidgetState::default();
    let review = create_test_review();

    state.set_review(review.clone());

    assert!(state.review.is_some());
    let stored_review = state.review.unwrap();
    assert_eq!(stored_review.files_count, 5);
    assert_eq!(stored_review.issues_count, 10);
    assert_eq!(stored_review.critical_issues, 2);
}

#[test]
fn test_format_cycling_next() {
    let mut state = ReportsWidgetState::default();

    // Start with Summary
    assert_eq!(state.selected_format, ReportFormat::Summary);

    // Summary -> Detailed
    state.next_format();
    assert_eq!(state.selected_format, ReportFormat::Detailed);

    // Detailed -> Json
    state.next_format();
    assert_eq!(state.selected_format, ReportFormat::Json);

    // Json -> Markdown
    state.next_format();
    assert_eq!(state.selected_format, ReportFormat::Markdown);

    // Markdown -> Summary (cycle back)
    state.next_format();
    assert_eq!(state.selected_format, ReportFormat::Summary);
}

#[test]
fn test_format_cycling_previous() {
    let mut state = ReportsWidgetState::default();

    // Start with Summary -> Markdown (reverse cycle)
    state.previous_format();
    assert_eq!(state.selected_format, ReportFormat::Markdown);

    // Markdown -> Json
    state.previous_format();
    assert_eq!(state.selected_format, ReportFormat::Json);

    // Json -> Detailed
    state.previous_format();
    assert_eq!(state.selected_format, ReportFormat::Detailed);

    // Detailed -> Summary
    state.previous_format();
    assert_eq!(state.selected_format, ReportFormat::Summary);
}

#[test]
fn test_export_status_transitions() {
    let mut state = ReportsWidgetState::default();

    // Start with None
    assert!(matches!(state.export_status, ExportStatus::None));

    // Start export
    state.start_export("pdf".to_string());
    if let ExportStatus::Exporting(format) = &state.export_status {
        assert_eq!(format, "pdf");
    } else {
        panic!("Expected Exporting status");
    }

    // Complete export
    state.complete_export("/path/to/report.pdf".to_string());
    if let ExportStatus::Success(path) = &state.export_status {
        assert_eq!(path, "/path/to/report.pdf");
    } else {
        panic!("Expected Success status");
    }
}

#[test]
fn test_generate_summary_report() {
    let mut state = ReportsWidgetState::default();
    let review = create_test_review();
    state.set_review(review);
    state.selected_format = ReportFormat::Summary;

    let report = state.generate_report();

    assert!(report.is_some());
    let report_content = report.unwrap();

    // Check that summary contains key information
    assert!(report_content.contains("AI Code Review Summary"));
    assert!(report_content.contains("Files analyzed: 5"));
    assert!(report_content.contains("Total issues found: 10"));
    assert!(report_content.contains("Critical: 2 issues"));
    assert!(report_content.contains("High: 3 issues"));
    assert!(report_content.contains("Medium: 3 issues"));
    assert!(report_content.contains("Low: 2 issues"));

    // Check that view mode changed to Report
    assert_eq!(state.view_mode, ViewMode::Report);
    assert!(state.generated_report.is_some());
}

#[test]
fn test_generate_detailed_report() {
    let mut state = ReportsWidgetState::default();
    let review = create_test_review();
    state.set_review(review);
    state.selected_format = ReportFormat::Detailed;

    let report = state.generate_report();

    assert!(report.is_some());
    let report_content = report.unwrap();

    // Check detailed report structure
    assert!(report_content.contains("AI Code Review - Detailed Report"));
    assert!(report_content.contains("CRITICAL ISSUES"));
    assert!(report_content.contains("HIGH PRIORITY ISSUES"));
    assert!(report_content.contains("MEDIUM PRIORITY ISSUES"));
    assert!(report_content.contains("LOW PRIORITY ISSUES"));

    // Check specific issue details
    assert!(report_content.contains("src/main.rs"));
    assert!(report_content.contains("Line: 42"));
    assert!(report_content.contains("Potential buffer overflow vulnerability"));
    assert!(report_content.contains("src/utils.rs"));
    assert!(report_content.contains("Inefficient algorithm detected"));
}

#[test]
fn test_generate_json_report() {
    let mut state = ReportsWidgetState::default();
    let review = create_test_review();
    state.set_review(review);
    state.selected_format = ReportFormat::Json;

    let report = state.generate_report();

    assert!(report.is_some());
    let report_content = report.unwrap();

    // Check that it's valid JSON by trying to parse key elements
    assert!(report_content.contains("\"files_count\""));
    assert!(report_content.contains("\"issues_count\""));
    assert!(report_content.contains("\"critical_issues\""));
    assert!(report_content.contains("\"issues\""));
    assert!(report_content.contains("\"severity\""));

    // Should be pretty-printed JSON
    assert!(report_content.contains("  ")); // Indentation
}

#[test]
fn test_generate_markdown_report() {
    let mut state = ReportsWidgetState::default();
    let review = create_test_review();
    state.set_review(review);
    state.selected_format = ReportFormat::Markdown;

    let report = state.generate_report();

    assert!(report.is_some());
    let report_content = report.unwrap();

    // Check markdown structure
    assert!(report_content.contains("# 🤖 AI Code Review Report"));
    assert!(report_content.contains("## 📊 Summary"));
    assert!(report_content.contains("- **Files analyzed:** 5"));
    assert!(report_content.contains("- **Total issues:** 10"));
    assert!(report_content.contains("## 📋 Issues by Severity"));
    assert!(report_content.contains("### 🚨 Critical Priority Issues"));
    assert!(report_content.contains("### ⚠️ High Priority Issues"));
    assert!(report_content.contains("- **File:** `src/main.rs`"));
    assert!(report_content.contains("*Report generated by AI Code Buddy*"));
}

#[test]
fn test_generate_report_without_review() {
    let mut state = ReportsWidgetState::default();
    // Don't set a review

    let report = state.generate_report();

    assert!(report.is_none());
    assert_eq!(state.view_mode, ViewMode::Selection); // Should remain in selection mode
    assert!(state.generated_report.is_none());
}

#[test]
fn test_generate_report_with_no_issues() {
    let mut state = ReportsWidgetState::default();
    let clean_review = Review {
        files_count: 3,
        issues_count: 0,
        critical_issues: 0,
        high_issues: 0,
        medium_issues: 0,
        low_issues: 0,
        issues: vec![],
    };

    state.set_review(clean_review);
    state.selected_format = ReportFormat::Detailed;

    let report = state.generate_report();

    assert!(report.is_some());
    let report_content = report.unwrap();

    // Should show "no issues" message
    assert!(report_content.contains("No issues found! Your code looks great!"));
}

#[test]
fn test_back_to_selection() {
    let mut state = ReportsWidgetState::default();
    let review = create_test_review();
    state.set_review(review);

    // Generate a report (switches to Report view)
    state.generate_report();
    assert_eq!(state.view_mode, ViewMode::Report);

    // Go back to selection
    state.back_to_selection();
    assert_eq!(state.view_mode, ViewMode::Selection);
}

#[test]
fn test_view_mode_enum_values() {
    // Test enum variants
    let selection = ViewMode::Selection;
    let report = ViewMode::Report;

    // Test equality
    assert_eq!(selection, ViewMode::Selection);
    assert_eq!(report, ViewMode::Report);
    assert_ne!(selection, report);
}

#[test]
fn test_summary_report_different_issue_levels() {
    let mut state = ReportsWidgetState::default();

    // Test with low issues only
    let review_low = Review {
        files_count: 5,
        issues_count: 1,
        critical_issues: 0,
        high_issues: 0,
        medium_issues: 0,
        low_issues: 1,
        issues: vec![Issue {
            file: "test.rs".to_string(),
            line: 10,
            severity: "Low".to_string(),
            category: "performance".to_string(),
            description: "Low severity issue".to_string(),
            commit_status: CommitStatus::Modified,
        }],
    };

    state.set_review(review_low);
    state.selected_format = ReportFormat::Summary;

    if let Some(report) = state.generate_report() {
        assert!(report.contains("💡"));
        assert!(report.contains("✅"));
    } else {
        panic!("Report generation failed");
    }
}

#[test]
fn test_detailed_report_unknown_severity() {
    let mut state = ReportsWidgetState::default();

    // Test with unknown severity (should be treated as low)
    let review = Review {
        files_count: 1,
        issues_count: 1,
        critical_issues: 0,
        high_issues: 0,
        medium_issues: 0,
        low_issues: 1,
        issues: vec![Issue {
            file: "test.rs".to_string(),
            line: 10,
            severity: "Unknown".to_string(),
            category: "misc".to_string(),
            description: "Unknown severity issue".to_string(),
            commit_status: CommitStatus::Modified,
        }],
    };

    state.set_review(review);
    state.selected_format = ReportFormat::Detailed;

    if let Some(report) = state.generate_report() {
        assert!(report.contains("ℹ️  LOW PRIORITY"));
        assert!(report.contains("Unknown severity issue"));
    } else {
        panic!("Report generation failed");
    }
}

#[test]
fn test_json_report_valid_structure() {
    let mut state = ReportsWidgetState::default();

    // Test with valid review (should succeed)
    let review = Review {
        files_count: 1,
        issues_count: 0,
        critical_issues: 0,
        high_issues: 0,
        medium_issues: 0,
        low_issues: 0,
        issues: vec![],
    };

    state.set_review(review);
    state.selected_format = ReportFormat::Json;

    if let Some(json_report) = state.generate_report() {
        assert!(json_report.contains("files_count"));
        assert!(json_report.contains("issues_count"));
    } else {
        panic!("JSON report generation failed");
    }
}
#[test]
fn test_report_format_enum_values() {
    // Test enum variants and equality
    assert_eq!(ReportFormat::Summary, ReportFormat::Summary);
    assert_eq!(ReportFormat::Detailed, ReportFormat::Detailed);
    assert_eq!(ReportFormat::Json, ReportFormat::Json);
    assert_eq!(ReportFormat::Markdown, ReportFormat::Markdown);

    // Test inequality
    assert_ne!(ReportFormat::Summary, ReportFormat::Detailed);
    assert_ne!(ReportFormat::Json, ReportFormat::Markdown);
}

#[test]
fn test_export_status_debug_clone() {
    // Test Debug and Clone traits
    let status1 = ExportStatus::None;
    let status2 = ExportStatus::Exporting("pdf".to_string());
    let status3 = ExportStatus::Success("/path/file.txt".to_string());

    // Test Clone
    let status1_clone = status1.clone();
    let status2_clone = status2.clone();
    let status3_clone = status3.clone();

    // Test Debug format
    assert!(format!("{status1_clone:?}").contains("None"));
    assert!(format!("{status2_clone:?}").contains("Exporting"));
    assert!(format!("{status2_clone:?}").contains("pdf"));
    assert!(format!("{status3_clone:?}").contains("Success"));
    assert!(format!("{status3_clone:?}").contains("/path/file.txt"));
}

#[test]
fn test_comprehensive_report_workflow() {
    let mut state = ReportsWidgetState::default();
    let review = create_test_review();

    // Complete workflow test
    assert_eq!(state.view_mode, ViewMode::Selection);

    // Set review
    state.set_review(review);

    // Cycle through formats and generate reports
    for format in [
        ReportFormat::Summary,
        ReportFormat::Detailed,
        ReportFormat::Json,
        ReportFormat::Markdown,
    ] {
        state.selected_format = format;

        let report = state.generate_report();
        assert!(report.is_some());
        assert_eq!(state.view_mode, ViewMode::Report);

        // Go back to selection
        state.back_to_selection();
        assert_eq!(state.view_mode, ViewMode::Selection);
    }

    // Test export workflow
    state.start_export("markdown".to_string());
    assert!(matches!(state.export_status, ExportStatus::Exporting(_)));

    state.complete_export("/exports/report.md".to_string());
    assert!(matches!(state.export_status, ExportStatus::Success(_)));
}
