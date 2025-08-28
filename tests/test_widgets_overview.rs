use ai_code_buddy::{
    args::{Args, OutputFormat},
    bevy_states::app::AppState,
    events::app::AppEvent,
    events::overview::OverviewEvent,
    widget_states::overview::{
        OverviewComponent, OverviewWidgetState, RepoInfo, SelectionDirection,
    },
    widgets::overview::{initialize_overview_state, overview_event_handler, OverviewWidget},
};
use bevy::{app::App, ecs::event::Events, prelude::*, state::app::StatesPlugin};
use bevy_ratatui::event::{KeyEvent, MouseEvent};
use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::{buffer::Buffer, layout::Rect, widgets::StatefulWidgetRef};

// Test initialization
#[test]
fn test_initialize_overview_state() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .init_resource::<OverviewWidgetState>()
        .insert_resource(Args {
            repo_path: "/test/repo".to_string(),
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: OutputFormat::Summary,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: false,
        });

    // Add the system
    app.add_systems(Startup, initialize_overview_state);
    app.update();

    let overview_state = app.world().get_resource::<OverviewWidgetState>().unwrap();
    assert_eq!(overview_state.repo_info.path, "/test/repo");
    assert_eq!(overview_state.repo_info.source_branch, "feature");
    assert_eq!(overview_state.repo_info.target_branch, "main");
    assert_eq!(overview_state.repo_info.files_to_analyze, 42); // Placeholder value
}

// Test keyboard event handling
#[test]
fn test_overview_keyboard_navigation() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<AppState>()
        .add_event::<OverviewEvent>()
        .add_event::<AppEvent>()
        .init_resource::<OverviewWidgetState>()
        .insert_resource(Args {
            repo_path: "/test/repo".to_string(),
            source_branch: "feature".to_string(),
            target_branch: "main".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: OutputFormat::Summary,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: false,
        });

    // Set initial state
    let mut state = app
        .world_mut()
        .get_resource_mut::<NextState<AppState>>()
        .unwrap();
    state.set(AppState::Overview);
    app.update();

    // Send Tab key event
    let mut events = app
        .world_mut()
        .get_resource_mut::<Events<OverviewEvent>>()
        .unwrap();
    events.send(OverviewEvent::KeyEvent(KeyEvent(
        crossterm::event::KeyEvent {
            code: KeyCode::Tab,
            modifiers: KeyModifiers::empty(),
            kind: KeyEventKind::Release,
            state: crossterm::event::KeyEventState::empty(),
        },
    )));

    // Add the event handler system
    app.add_systems(Update, overview_event_handler);
    app.update();

    let overview_state = app.world().get_resource::<OverviewWidgetState>().unwrap();
    assert_eq!(
        overview_state.selected_component,
        OverviewComponent::ViewReports
    );
}

// Test widget state methods
#[test]
fn test_overview_widget_state_methods() {
    let mut state = OverviewWidgetState::default();

    // Test move_selection
    state.selected_component = OverviewComponent::StartAnalysis;
    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::ViewReports);

    state.move_selection(SelectionDirection::Previous);
    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);

    // Test is_over method
    state.registered_components.insert(
        OverviewComponent::Help,
        Rect {
            x: 5,
            y: 5,
            width: 10,
            height: 5,
        },
    );

    assert!(state.is_over(OverviewComponent::Help, 7, 7)); // Inside rect
    assert!(!state.is_over(OverviewComponent::Help, 20, 20)); // Outside rect

    // Test update_hover
    state.update_hover(7, 7);
    assert_eq!(state.hovered_component, Some(OverviewComponent::Help));

    state.update_hover(20, 20);
    assert_eq!(state.hovered_component, None);
}

// Test widget rendering
#[test]
fn test_overview_widget_rendering() {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
    let mut state = OverviewWidgetState::default();

    // Set up some test data
    state.repo_info = RepoInfo {
        path: "/test/repo".to_string(),
        source_branch: "feature".to_string(),
        target_branch: "main".to_string(),
        files_to_analyze: 42,
    };

    let widget = OverviewWidget;
    widget.render_ref(Rect::new(0, 0, 80, 24), &mut buffer, &mut state);

    // Verify that components were registered during rendering
    assert!(!state.registered_components.is_empty());
    assert!(state
        .registered_components
        .contains_key(&OverviewComponent::StartAnalysis));
    assert!(state
        .registered_components
        .contains_key(&OverviewComponent::ViewReports));
    assert!(state
        .registered_components
        .contains_key(&OverviewComponent::Settings));
    assert!(state
        .registered_components
        .contains_key(&OverviewComponent::Help));
    assert!(state
        .registered_components
        .contains_key(&OverviewComponent::Exit));
}

// Test edge cases
#[test]
fn test_overview_cyclic_navigation() {
    let mut state = OverviewWidgetState::default();

    // Test forward navigation cycles back to start
    state.selected_component = OverviewComponent::Exit;
    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);

    // Test backward navigation cycles to end
    state.selected_component = OverviewComponent::StartAnalysis;
    state.move_selection(SelectionDirection::Previous);
    assert_eq!(state.selected_component, OverviewComponent::Exit);
}
