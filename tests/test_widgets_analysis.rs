use ai_code_buddy::{
    app_events_handler,
    args::Args,
    bevy_states::app::AppState,
    core::review::{CommitStatus, Issue, Review},
    events::analysis::AnalysisEvent,
    theme::THEME,
    widget_states::analysis::AnalysisWidgetState,
    widgets::analysis::{AnalysisPlugin, AnalysisWidget},
};
use bevy::{app::App, ecs::event::Events, prelude::*, state::app::StatesPlugin};
use bevy_ratatui::event::{KeyEvent as BevyKeyEvent, MouseEvent};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    backend::TestBackend,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::StatefulWidgetRef,
    Terminal,
};

#[test]
fn test_analysis_widget_state_default() {
    let state = AnalysisWidgetState::default();
    assert!(!state.is_analyzing);
    assert_eq!(state.progress, 0.0);
    assert_eq!(state.current_file, "");
    assert!(state.review.is_none());
    assert_eq!(state.selected_issue, 0);
}

#[test]
fn test_analysis_widget_state_start_analysis() {
    let mut state = AnalysisWidgetState::default();
    state.is_analyzing = false;
    state.progress = 50.0;
    state.current_file = "test.rs".to_string();
    state.review = Some(Review {
        issues: vec![Issue {
            category: "Test".to_string(),
            description: "Test issue".to_string(),
            file: "test.rs".to_string(),
            line: 1,
            severity: "high".to_string(),
            commit_status: CommitStatus::Modified,
        }],
        ..Default::default()
    });

    state.start_analysis();

    assert!(state.is_analyzing);
    assert_eq!(state.progress, 0.0);
    assert_eq!(state.current_file, "");
    assert!(state.review.is_none());
}

#[test]
fn test_analysis_widget_state_update_progress() {
    let mut state = AnalysisWidgetState::default();
    state.update_progress(75.0, "src/main.rs".to_string());

    assert_eq!(state.progress, 75.0);
    assert_eq!(state.current_file, "src/main.rs");
}

#[test]
fn test_analysis_widget_state_complete_analysis() {
    let mut state = AnalysisWidgetState::default();
    state.is_analyzing = true;
    state.progress = 50.0;

    let review = Review {
        issues: vec![
            Issue {
                category: "Security".to_string(),
                description: "Security issue".to_string(),
                file: "src/main.rs".to_string(),
                line: 42,
                severity: "high".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                category: "Performance".to_string(),
                description: "Performance issue".to_string(),
                file: "src/lib.rs".to_string(),
                line: 10,
                severity: "medium".to_string(),
                commit_status: CommitStatus::Staged,
            },
        ],
        ..Default::default()
    };

    state.complete_analysis(review.clone());

    assert!(!state.is_analyzing);
    assert_eq!(state.progress, 100.0);
    assert_eq!(state.selected_issue, 0);
    assert_eq!(state.review.as_ref().unwrap().issues.len(), 2);
}

#[test]
fn test_analysis_widget_state_move_issue_selection() {
    let mut state = AnalysisWidgetState::default();
    let review = Review {
        issues: vec![
            Issue {
                category: "Issue1".to_string(),
                description: "First issue".to_string(),
                file: "file1.rs".to_string(),
                line: 1,
                severity: "high".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                category: "Issue2".to_string(),
                description: "Second issue".to_string(),
                file: "file2.rs".to_string(),
                line: 2,
                severity: "medium".to_string(),
                commit_status: CommitStatus::Staged,
            },
            Issue {
                category: "Issue3".to_string(),
                description: "Third issue".to_string(),
                file: "file3.rs".to_string(),
                line: 3,
                severity: "low".to_string(),
                commit_status: CommitStatus::Untracked,
            },
        ],
        ..Default::default()
    };

    state.review = Some(review);

    // Test moving down
    state.move_issue_selection(1);
    assert_eq!(state.selected_issue, 1);

    // Test moving down again
    state.move_issue_selection(1);
    assert_eq!(state.selected_issue, 2);

    // Test moving down beyond bounds
    state.move_issue_selection(1);
    assert_eq!(state.selected_issue, 2);

    // Test moving up
    state.move_issue_selection(-1);
    assert_eq!(state.selected_issue, 1);

    // Test moving up beyond bounds
    state.move_issue_selection(-10);
    assert_eq!(state.selected_issue, 0);
}

#[test]
fn test_analysis_widget_state_move_issue_selection_empty() {
    let mut state = AnalysisWidgetState::default();
    state.selected_issue = 5;

    // Should not panic with empty review
    state.move_issue_selection(1);
    assert_eq!(state.selected_issue, 5);

    state.move_issue_selection(-1);
    assert_eq!(state.selected_issue, 5);
}

#[test]
fn test_analysis_widget_render_default_state() {
    let mut state = AnalysisWidgetState::default();
    let widget = AnalysisWidget;

    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    widget.render_ref(area, &mut buf, &mut state);

    // Check that title is rendered by collecting text from the title area
    let mut title_text = String::new();
    for x in 30..50 {
        let cell = &buf[(x, 1)];
        if !cell.symbol().is_empty() {
            title_text.push_str(cell.symbol());
        }
    }

    assert!(
        title_text.contains("Code Analysis"),
        "Title should contain 'Code Analysis', found: '{}'",
        title_text
    );
}

#[test]
fn test_analysis_widget_render_analyzing_state() {
    let mut state = AnalysisWidgetState::default();
    state.is_analyzing = true;
    state.progress = 75.0;
    state.current_file = "src/main.rs".to_string();

    let widget = AnalysisWidget;
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    widget.render_ref(area, &mut buf, &mut state);

    // Check that progress information is rendered
    let content = buf.content();
    let content_str = content.iter().map(|cell| cell.symbol()).collect::<String>();

    assert!(content_str.contains("75"));
    assert!(content_str.contains("src/main.rs"));
}

#[test]
fn test_analysis_widget_render_completed_state() {
    let mut state = AnalysisWidgetState::default();
    let review = Review {
        issues: vec![
            Issue {
                category: "Security".to_string(),
                description: "Potential security vulnerability".to_string(),
                file: "src/auth.rs".to_string(),
                line: 25,
                severity: "high".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                category: "Performance".to_string(),
                description: "Inefficient algorithm".to_string(),
                file: "src/algorithm.rs".to_string(),
                line: 100,
                severity: "medium".to_string(),
                commit_status: CommitStatus::Staged,
            },
        ],
        ..Default::default()
    };

    state.review = Some(review);
    state.selected_issue = 1; // Select second issue

    let widget = AnalysisWidget;
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    widget.render_ref(area, &mut buf, &mut state);

    // Check that issues are rendered
    let content = buf.content();
    let content_str = content.iter().map(|cell| cell.symbol()).collect::<String>();

    assert!(content_str.contains("Security"));
    assert!(content_str.contains("Performance"));
    assert!(content_str.contains("Potential security vulnerability"));
    assert!(content_str.contains("Inefficient algorithm"));
}

#[test]
fn test_analysis_widget_render_empty_issues() {
    let mut state = AnalysisWidgetState::default();
    let review = Review {
        issues: vec![],
        ..Default::default()
    };

    state.review = Some(review);

    let widget = AnalysisWidget;
    let area = Rect::new(0, 0, 80, 24);
    let mut buf = Buffer::empty(area);

    widget.render_ref(area, &mut buf, &mut state);

    // Check that "no issues found" message is rendered
    let content = buf.content();
    let content_str = content.iter().map(|cell| cell.symbol()).collect::<String>();

    assert!(content_str.contains("No issues found"));
}

#[test]
fn test_analysis_widget_render_small_area() {
    let mut state = AnalysisWidgetState::default();
    let widget = AnalysisWidget;

    let area = Rect::new(0, 0, 20, 10); // Very small area
    let mut buf = Buffer::empty(area);

    // Should not panic with small area
    widget.render_ref(area, &mut buf, &mut state);
}

#[test]
fn test_analysis_widget_render_large_area() {
    let mut state = AnalysisWidgetState::default();
    let widget = AnalysisWidget;

    let area = Rect::new(0, 0, 200, 100); // Large area
    let mut buf = Buffer::empty(area);

    widget.render_ref(area, &mut buf, &mut state);

    // Should handle large areas gracefully
    assert!(buf.area().width >= 200);
    assert!(buf.area().height >= 100);
}

#[test]
fn test_analysis_event_handler_escape_key() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<AnalysisEvent>()
        .add_event::<ai_code_buddy::events::app::AppEvent>()
        .add_event::<bevy::app::AppExit>()
        .init_resource::<AnalysisWidgetState>()
        .insert_resource(Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "HEAD".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: ai_code_buddy::args::OutputFormat::Summary,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: false,

            parallel: false,
            disable_ai: false,
        })
        .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default());

    // Set state to Analysis
    let mut state = app
        .world_mut()
        .get_resource_mut::<NextState<AppState>>()
        .unwrap();
    state.set(AppState::Analysis);
    app.update();

    // Set up analyzing state
    let mut analysis_state = app
        .world_mut()
        .get_resource_mut::<AnalysisWidgetState>()
        .unwrap();
    analysis_state.is_analyzing = true;

    // Send escape key event
    let mut events = app
        .world_mut()
        .get_resource_mut::<Events<AnalysisEvent>>()
        .unwrap();
    events.send(AnalysisEvent::KeyEvent(BevyKeyEvent(KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: crossterm::event::KeyEventState::empty(),
    })));

    // Add the system
    app.add_systems(
        Update,
        ai_code_buddy::widgets::analysis::analysis_event_handler,
    );
    app.add_systems(Update, app_events_handler);

    app.update();
    app.update(); // Run another update to process the AppEvent

    // Check that analysis was stopped and state changed
    let analysis_state = app.world().get_resource::<AnalysisWidgetState>().unwrap();
    assert!(!analysis_state.is_analyzing);

    let app_state = app.world().get_resource::<State<AppState>>().unwrap();
    assert_eq!(*app_state.get(), AppState::Overview);
}

#[test]
fn test_analysis_event_handler_enter_key_start_analysis() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<AnalysisEvent>()
        .add_event::<ai_code_buddy::events::app::AppEvent>()
        .add_event::<bevy::app::AppExit>()
        .init_resource::<AnalysisWidgetState>()
        .insert_resource(Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "HEAD".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: ai_code_buddy::args::OutputFormat::Summary,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: false,
            parallel: false,
            disable_ai: false,
        })
        .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default());

    // Set state to Analysis
    let mut state = app
        .world_mut()
        .get_resource_mut::<NextState<AppState>>()
        .unwrap();
    state.set(AppState::Analysis);
    app.update();

    // Send enter key event
    let mut events = app
        .world_mut()
        .get_resource_mut::<Events<AnalysisEvent>>()
        .unwrap();
    events.send(AnalysisEvent::KeyEvent(BevyKeyEvent(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Release,
        state: crossterm::event::KeyEventState::empty(),
    })));

    // Add the system
    app.add_systems(
        Update,
        ai_code_buddy::widgets::analysis::analysis_event_handler,
    );

    app.update();

    // Check that analysis completed successfully
    let analysis_state = app.world().get_resource::<AnalysisWidgetState>().unwrap();
    assert!(!analysis_state.is_analyzing); // Analysis should be complete
    assert!(analysis_state.review.is_some()); // Should have review data
    assert_eq!(analysis_state.progress, 100.0); // Should be 100% complete
}

#[test]
fn test_analysis_event_handler_navigation_keys() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<AnalysisEvent>()
        .add_event::<ai_code_buddy::events::app::AppEvent>()
        .add_event::<bevy::app::AppExit>()
        .init_resource::<AnalysisWidgetState>()
        .insert_resource(Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "HEAD".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: ai_code_buddy::args::OutputFormat::Summary,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: false,

            parallel: false,
            disable_ai: false,
        })
        .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default());

    // Set state to Analysis
    let mut state = app
        .world_mut()
        .get_resource_mut::<NextState<AppState>>()
        .unwrap();
    state.set(AppState::Analysis);
    app.update();

    // Set up completed analysis with issues
    let mut analysis_state = app
        .world_mut()
        .get_resource_mut::<AnalysisWidgetState>()
        .unwrap();
    let review = Review {
        issues: vec![
            Issue {
                category: "Issue1".to_string(),
                description: "First issue".to_string(),
                file: "file1.rs".to_string(),
                line: 1,
                severity: "high".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                category: "Issue2".to_string(),
                description: "Second issue".to_string(),
                file: "file2.rs".to_string(),
                line: 2,
                severity: "medium".to_string(),
                commit_status: CommitStatus::Staged,
            },
        ],
        ..Default::default()
    };
    analysis_state.review = Some(review);
    analysis_state.selected_issue = 0;

    // Send down arrow key event
    let mut events = app
        .world_mut()
        .get_resource_mut::<Events<AnalysisEvent>>()
        .unwrap();
    events.send(AnalysisEvent::KeyEvent(BevyKeyEvent(KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Release,
        state: crossterm::event::KeyEventState::empty(),
    })));

    // Add the system
    app.add_systems(
        Update,
        ai_code_buddy::widgets::analysis::analysis_event_handler,
    );

    app.update();

    // Check that selection moved down
    let analysis_state = app.world().get_resource::<AnalysisWidgetState>().unwrap();
    assert_eq!(analysis_state.selected_issue, 1);
}

#[test]
fn test_analysis_event_handler_reports_key() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<AnalysisEvent>()
        .add_event::<ai_code_buddy::events::app::AppEvent>()
        .init_resource::<AnalysisWidgetState>()
        .insert_resource(Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "HEAD".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: ai_code_buddy::args::OutputFormat::Summary,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: false,

            parallel: false,
            disable_ai: false,
        })
        .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default());

    // Set state to Analysis
    let mut state = app
        .world_mut()
        .get_resource_mut::<NextState<AppState>>()
        .unwrap();
    state.set(AppState::Analysis);
    app.update();

    // Send 'r' key event
    let mut events = app
        .world_mut()
        .get_resource_mut::<Events<AnalysisEvent>>()
        .unwrap();
    events.send(AnalysisEvent::KeyEvent(BevyKeyEvent(KeyEvent {
        code: KeyCode::Char('r'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Release,
        state: crossterm::event::KeyEventState::empty(),
    })));

    // Add the system
    app.add_systems(
        Update,
        ai_code_buddy::widgets::analysis::analysis_event_handler,
    );

    app.update();

    // Check that app event was sent to switch to reports
    let app_events = app
        .world()
        .get_resource::<Events<ai_code_buddy::events::app::AppEvent>>()
        .unwrap();
    let mut cursor = app_events.get_cursor();
    let switch_found = cursor.read(app_events).any(|event| {
        matches!(
            event,
            ai_code_buddy::events::app::AppEvent::SwitchTo(AppState::Reports)
        )
    });
    assert!(switch_found, "Should have sent switch to reports event");
}

#[test]
fn test_analysis_plugin_setup() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(AnalysisPlugin);

    // Check that resources were initialized
    let analysis_state = app.world().get_resource::<AnalysisWidgetState>();
    assert!(analysis_state.is_some());

    // Check that events were added
    let analysis_events = app.world().get_resource::<Events<AnalysisEvent>>();
    assert!(analysis_events.is_some());
}
