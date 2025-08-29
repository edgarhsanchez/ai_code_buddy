use ai_code_buddy::core::review::{CommitStatus, Issue, Review};
use ai_code_buddy::widget_states::analysis::AnalysisWidgetState;
use ai_code_buddy::widget_states::overview::{
    OverviewComponent, OverviewWidgetState, RepoInfo, SelectionDirection,
};
use ai_code_buddy::widget_states::reports::{
    ExportStatus, ReportFormat, ReportsWidgetState, ViewMode,
};
use ratatui::layout::Rect;

#[test]
fn test_analysis_widget_state_default() {
    let state = AnalysisWidgetState::default();

    assert!(!state.is_analyzing);
    assert_eq!(state.progress, 0.0);
    assert!(state.current_file.is_empty());
    assert!(state.review.is_none());
    assert_eq!(state.selected_issue, 0);
}

#[test]
fn test_start_analysis() {
    let mut state = AnalysisWidgetState {
        is_analyzing: false,
        progress: 50.0,
        current_file: "old_file.rs".to_string(),
        review: Some(Review {
            files_count: 1,
            issues_count: 0,
            critical_issues: 0,
            high_issues: 0,
            medium_issues: 0,
            low_issues: 0,
            issues: vec![],
        }),
        selected_issue: 5,
    };

    state.start_analysis();

    assert!(state.is_analyzing);
    assert_eq!(state.progress, 0.0);
    assert!(state.current_file.is_empty());
    assert!(state.review.is_none());
    assert_eq!(state.selected_issue, 5); // start_analysis doesn't reset selected_issue
}

#[test]
fn test_update_progress() {
    let mut state = AnalysisWidgetState::default();

    state.update_progress(0.5, "src/main.rs".to_string());

    assert_eq!(state.progress, 0.5);
    assert_eq!(state.current_file, "src/main.rs");
}

#[test]
fn test_complete_analysis() {
    let mut state = AnalysisWidgetState::default();
    state.start_analysis();

    let review = Review {
        files_count: 2,
        issues_count: 1,
        critical_issues: 0,
        high_issues: 1,
        medium_issues: 0,
        low_issues: 0,
        issues: vec![Issue {
            file: "src/test.rs".to_string(),
            line: 10,
            severity: "High".to_string(),
            category: "Security".to_string(),
            description: "Potential vulnerability".to_string(),
            commit_status: CommitStatus::Modified,
        }],
    };

    state.complete_analysis(review.clone());

    assert!(!state.is_analyzing);
    assert_eq!(state.progress, 100.0);
    assert_eq!(state.current_file, ""); // current_file should be cleared
    assert!(state.review.is_some());
    assert_eq!(state.review.as_ref().unwrap().files_count, 2);
    assert_eq!(state.review.as_ref().unwrap().issues_count, 1);
}

#[test]
fn test_move_issue_selection_forward() {
    let mut state = AnalysisWidgetState::default();
    let review = Review {
        files_count: 1,
        issues_count: 3,
        critical_issues: 0,
        high_issues: 0,
        medium_issues: 0,
        low_issues: 3,
        issues: vec![
            Issue {
                file: "file1.rs".to_string(),
                line: 1,
                severity: "Low".to_string(),
                category: "Style".to_string(),
                description: "Issue 1".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                file: "file2.rs".to_string(),
                line: 2,
                severity: "Low".to_string(),
                category: "Style".to_string(),
                description: "Issue 2".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                file: "file3.rs".to_string(),
                line: 3,
                severity: "Low".to_string(),
                category: "Style".to_string(),
                description: "Issue 3".to_string(),
                commit_status: CommitStatus::Modified,
            },
        ],
    };

    state.complete_analysis(review);

    // Start at issue 0
    assert_eq!(state.selected_issue, 0);

    // Move forward
    state.move_issue_selection(1);
    assert_eq!(state.selected_issue, 1);

    // Move forward
    state.move_issue_selection(1);
    assert_eq!(state.selected_issue, 2);

    // Should clamp at max (2)
    state.move_issue_selection(1);
    assert_eq!(state.selected_issue, 2);
}

#[test]
fn test_move_issue_selection_backward() {
    let mut state = AnalysisWidgetState::default();
    let review = Review {
        files_count: 1,
        issues_count: 3,
        critical_issues: 0,
        high_issues: 0,
        medium_issues: 0,
        low_issues: 3,
        issues: (0..3)
            .map(|i| Issue {
                file: format!("file{i}.rs"),
                line: i,
                severity: "Low".to_string(),
                category: "Style".to_string(),
                description: format!("Issue {i}"),
                commit_status: CommitStatus::Modified,
            })
            .collect(),
    };

    state.complete_analysis(review);

    // Start at issue 0, should clamp at 0
    state.move_issue_selection(-1);
    assert_eq!(state.selected_issue, 0);

    // Move to issue 2 first
    state.move_issue_selection(2);
    assert_eq!(state.selected_issue, 2);

    // Move backward
    state.move_issue_selection(-1);
    assert_eq!(state.selected_issue, 1);

    // Move backward
    state.move_issue_selection(-1);
    assert_eq!(state.selected_issue, 0);
}

#[test]
fn test_move_issue_selection_empty_issues() {
    let mut state = AnalysisWidgetState::default();
    let review = Review {
        files_count: 1,
        issues_count: 0,
        critical_issues: 0,
        high_issues: 0,
        medium_issues: 0,
        low_issues: 0,
        issues: vec![],
    };

    state.complete_analysis(review);

    // Should stay at 0 when no issues exist
    state.move_issue_selection(1);
    assert_eq!(state.selected_issue, 0);

    state.move_issue_selection(-1);
    assert_eq!(state.selected_issue, 0);
}

// === Overview Widget State Tests ===

#[test]
fn test_overview_widget_state_default() {
    let state = OverviewWidgetState::default();

    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);
    assert!(state.hovered_component.is_none());
    assert!(state.registered_components.is_empty());
    assert!(!state.show_help);
    assert_eq!(state.repo_info.path, ".");
    assert_eq!(state.repo_info.source_branch, "main");
    assert_eq!(state.repo_info.target_branch, "HEAD");
    assert_eq!(state.repo_info.files_to_analyze, 0);
}

#[test]
fn test_move_selection_next() {
    let mut state = OverviewWidgetState::default();

    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);

    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::ViewReports);

    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::Settings);

    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::Credits);

    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::Help);

    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::Exit);

    // Should wrap around
    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);
}

#[test]
fn test_move_selection_previous() {
    let mut state = OverviewWidgetState::default();

    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);

    // Should wrap to end
    state.move_selection(SelectionDirection::Previous);
    assert_eq!(state.selected_component, OverviewComponent::Exit);

    state.move_selection(SelectionDirection::Previous);
    assert_eq!(state.selected_component, OverviewComponent::Help);

    state.move_selection(SelectionDirection::Previous);
    assert_eq!(state.selected_component, OverviewComponent::Credits);

    state.move_selection(SelectionDirection::Previous);
    assert_eq!(state.selected_component, OverviewComponent::Settings);

    state.move_selection(SelectionDirection::Previous);
    assert_eq!(state.selected_component, OverviewComponent::ViewReports);

    state.move_selection(SelectionDirection::Previous);
    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);
}

#[test]
fn test_is_over_component() {
    let mut state = OverviewWidgetState::default();

    // Register a component with a rect
    let rect = Rect::new(5, 5, 10, 3);
    state
        .registered_components
        .insert(OverviewComponent::StartAnalysis, rect);

    // Test coordinates inside the rect
    assert!(state.is_over(OverviewComponent::StartAnalysis, 7, 6));
    assert!(state.is_over(OverviewComponent::StartAnalysis, 5, 5));
    assert!(state.is_over(OverviewComponent::StartAnalysis, 14, 7));

    // Test coordinates outside the rect
    assert!(!state.is_over(OverviewComponent::StartAnalysis, 4, 6));
    assert!(!state.is_over(OverviewComponent::StartAnalysis, 15, 6));
    assert!(!state.is_over(OverviewComponent::StartAnalysis, 7, 4));
    assert!(!state.is_over(OverviewComponent::StartAnalysis, 7, 8));

    // Test unregistered component
    assert!(!state.is_over(OverviewComponent::ViewReports, 7, 6));
}

#[test]
fn test_update_hover() {
    let mut state = OverviewWidgetState::default();

    // Register components with rects
    state
        .registered_components
        .insert(OverviewComponent::StartAnalysis, Rect::new(0, 0, 10, 2));
    state
        .registered_components
        .insert(OverviewComponent::ViewReports, Rect::new(0, 3, 10, 2));
    state
        .registered_components
        .insert(OverviewComponent::Settings, Rect::new(0, 6, 10, 2));

    // Test hovering over first component
    state.update_hover(5, 1);
    assert_eq!(
        state.hovered_component,
        Some(OverviewComponent::StartAnalysis)
    );

    // Test hovering over second component
    state.update_hover(5, 4);
    assert_eq!(
        state.hovered_component,
        Some(OverviewComponent::ViewReports)
    );

    // Test hovering over third component
    state.update_hover(5, 7);
    assert_eq!(state.hovered_component, Some(OverviewComponent::Settings));

    // Test hovering over empty area
    state.update_hover(15, 1);
    assert!(state.hovered_component.is_none());

    state.update_hover(5, 10);
    assert!(state.hovered_component.is_none());
}

#[test]
fn test_repo_info_update() {
    let state = OverviewWidgetState {
        repo_info: RepoInfo {
            path: "/path/to/repo".to_string(),
            source_branch: "feature/new-feature".to_string(),
            target_branch: "develop".to_string(),
            files_to_analyze: 42,
        },
        ..Default::default()
    };

    assert_eq!(state.repo_info.path, "/path/to/repo");
    assert_eq!(state.repo_info.source_branch, "feature/new-feature");
    assert_eq!(state.repo_info.target_branch, "develop");
    assert_eq!(state.repo_info.files_to_analyze, 42);
}

// === Reports Widget State Tests ===

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
    let review = Review {
        files_count: 5,
        issues_count: 3,
        critical_issues: 1,
        high_issues: 1,
        medium_issues: 1,
        low_issues: 0,
        issues: vec![Issue {
            file: "src/test.rs".to_string(),
            line: 10,
            severity: "Critical".to_string(),
            category: "Security".to_string(),
            description: "SQL injection vulnerability".to_string(),
            commit_status: CommitStatus::Modified,
        }],
    };

    state.set_review(review.clone());

    assert!(state.review.is_some());
    assert_eq!(state.review.as_ref().unwrap().files_count, 5);
    assert_eq!(state.review.as_ref().unwrap().issues_count, 3);
}

#[test]
fn test_next_format() {
    let mut state = ReportsWidgetState::default();

    assert_eq!(state.selected_format, ReportFormat::Summary);

    state.next_format();
    assert_eq!(state.selected_format, ReportFormat::Detailed);

    state.next_format();
    assert_eq!(state.selected_format, ReportFormat::Json);

    state.next_format();
    assert_eq!(state.selected_format, ReportFormat::Markdown);

    // Should wrap around
    state.next_format();
    assert_eq!(state.selected_format, ReportFormat::Summary);
}

#[test]
fn test_previous_format() {
    let mut state = ReportsWidgetState::default();

    assert_eq!(state.selected_format, ReportFormat::Summary);

    // Should wrap to end
    state.previous_format();
    assert_eq!(state.selected_format, ReportFormat::Markdown);

    state.previous_format();
    assert_eq!(state.selected_format, ReportFormat::Json);

    state.previous_format();
    assert_eq!(state.selected_format, ReportFormat::Detailed);

    state.previous_format();
    assert_eq!(state.selected_format, ReportFormat::Summary);
}

#[test]
fn test_export_lifecycle() {
    let mut state = ReportsWidgetState::default();

    // Start with no export
    assert!(matches!(state.export_status, ExportStatus::None));

    // Start export
    state.start_export("json".to_string());
    assert!(matches!(state.export_status, ExportStatus::Exporting(_)));
    if let ExportStatus::Exporting(format) = &state.export_status {
        assert_eq!(format, "json");
    }

    // Complete export
    state.complete_export("/path/to/report.json".to_string());
    assert!(matches!(state.export_status, ExportStatus::Success(_)));
    if let ExportStatus::Success(path) = &state.export_status {
        assert_eq!(path, "/path/to/report.json");
    }
}

#[test]
fn test_generate_report_with_review() {
    let mut state = ReportsWidgetState::default();
    let review = Review {
        files_count: 2,
        issues_count: 2,
        critical_issues: 1,
        high_issues: 0,
        medium_issues: 1,
        low_issues: 0,
        issues: vec![
            Issue {
                file: "src/auth.rs".to_string(),
                line: 42,
                severity: "Critical".to_string(),
                category: "Security".to_string(),
                description: "Hardcoded password".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                file: "src/utils.rs".to_string(),
                line: 15,
                severity: "Medium".to_string(),
                category: "Performance".to_string(),
                description: "Inefficient loop".to_string(),
                commit_status: CommitStatus::Untracked,
            },
        ],
    };

    state.set_review(review);

    // Generate summary report
    state.selected_format = ReportFormat::Summary;
    let report = state.generate_report();

    assert!(report.is_some());
    assert!(state.generated_report.is_some());
    assert_eq!(state.view_mode, ViewMode::Report);

    let report_content = report.unwrap();
    assert!(report_content.contains("AI Code Review Summary"));
    assert!(report_content.contains("Files analyzed: 2"));
    assert!(report_content.contains("Total issues found: 2"));
}

#[test]
fn test_generate_report_without_review() {
    let mut state = ReportsWidgetState::default();

    let report = state.generate_report();

    assert!(report.is_none());
    assert!(state.generated_report.is_none());
    assert_eq!(state.view_mode, ViewMode::Selection);
}

#[test]
fn test_view_mode_transitions() {
    let mut state = ReportsWidgetState::default();
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

    // Start in selection mode
    assert_eq!(state.view_mode, ViewMode::Selection);

    // Generate report switches to report mode
    state.generate_report();
    assert_eq!(state.view_mode, ViewMode::Report);

    // Back to selection
    state.back_to_selection();
    assert_eq!(state.view_mode, ViewMode::Selection);
}
