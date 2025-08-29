use ai_code_buddy::widget_states::overview::{
    OverviewComponent, OverviewWidgetState, RepoInfo, SelectionDirection,
};
use ratatui::layout::Rect;

#[test]
fn test_overview_widget_state_default() {
    let state = OverviewWidgetState::default();

    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);
    assert_eq!(state.hovered_component, None);
    assert!(state.registered_components.is_empty());
    assert!(!state.show_help);

    // Check default repo info
    assert_eq!(state.repo_info.path, ".");
    assert_eq!(state.repo_info.source_branch, "main");
    assert_eq!(state.repo_info.target_branch, "HEAD");
    assert_eq!(state.repo_info.files_to_analyze, 0);
}

#[test]
fn test_overview_component_enum_values() {
    let components = [
        OverviewComponent::StartAnalysis,
        OverviewComponent::ViewReports,
        OverviewComponent::Settings,
        OverviewComponent::Help,
        OverviewComponent::Exit,
    ];

    // Test that components are cloneable and comparable
    for component in &components {
        let cloned = component.clone();
        assert_eq!(*component, cloned);
    }
}

#[test]
fn test_repo_info_clone() {
    let repo_info = RepoInfo {
        path: "/test/repo".to_string(),
        source_branch: "feature/test".to_string(),
        target_branch: "develop".to_string(),
        files_to_analyze: 42,
    };

    let cloned = repo_info.clone();
    assert_eq!(repo_info.path, cloned.path);
    assert_eq!(repo_info.source_branch, cloned.source_branch);
    assert_eq!(repo_info.target_branch, cloned.target_branch);
    assert_eq!(repo_info.files_to_analyze, cloned.files_to_analyze);
}

#[test]
fn test_move_selection_next() {
    let mut state = OverviewWidgetState::default();

    // Test forward navigation through all components
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

    // Test wrapping around to the beginning
    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);
}

#[test]
fn test_move_selection_previous() {
    let mut state = OverviewWidgetState::default();

    // Test backward navigation from the beginning (should wrap to end)
    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);

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
fn test_selection_direction_debug() {
    let next = SelectionDirection::Next;
    let previous = SelectionDirection::Previous;

    // Test that SelectionDirection implements Debug
    let _debug_next = format!("{next:?}");
    let _debug_previous = format!("{previous:?}");
}

#[test]
fn test_is_over_with_registered_component() {
    let mut state = OverviewWidgetState::default();

    // Register a component with a specific rect
    let rect = Rect::new(10, 5, 20, 3);
    state
        .registered_components
        .insert(OverviewComponent::StartAnalysis, rect);

    // Test coordinates inside the rect
    assert!(state.is_over(OverviewComponent::StartAnalysis, 15, 6));
    assert!(state.is_over(OverviewComponent::StartAnalysis, 10, 5)); // Top-left corner
    assert!(state.is_over(OverviewComponent::StartAnalysis, 29, 7)); // Bottom-right corner

    // Test coordinates outside the rect
    assert!(!state.is_over(OverviewComponent::StartAnalysis, 9, 6)); // Left of rect
    assert!(!state.is_over(OverviewComponent::StartAnalysis, 30, 6)); // Right of rect
    assert!(!state.is_over(OverviewComponent::StartAnalysis, 15, 4)); // Above rect
    assert!(!state.is_over(OverviewComponent::StartAnalysis, 15, 8)); // Below rect
}

#[test]
fn test_is_over_with_unregistered_component() {
    let state = OverviewWidgetState::default();

    // Test with a component that hasn't been registered
    assert!(!state.is_over(OverviewComponent::ViewReports, 10, 10));
}

#[test]
fn test_update_hover_with_components() {
    let mut state = OverviewWidgetState::default();

    // Register multiple components
    state
        .registered_components
        .insert(OverviewComponent::StartAnalysis, Rect::new(10, 5, 20, 3));
    state
        .registered_components
        .insert(OverviewComponent::ViewReports, Rect::new(10, 10, 20, 3));
    state
        .registered_components
        .insert(OverviewComponent::Exit, Rect::new(10, 15, 20, 3));

    // Test hovering over first component
    state.update_hover(15, 6);
    assert_eq!(
        state.hovered_component,
        Some(OverviewComponent::StartAnalysis)
    );

    // Test hovering over second component
    state.update_hover(15, 11);
    assert_eq!(
        state.hovered_component,
        Some(OverviewComponent::ViewReports)
    );

    // Test hovering over third component
    state.update_hover(15, 16);
    assert_eq!(state.hovered_component, Some(OverviewComponent::Exit));

    // Test hovering over empty area
    state.update_hover(50, 50);
    assert_eq!(state.hovered_component, None);
}

#[test]
fn test_update_hover_no_components() {
    let mut state = OverviewWidgetState::default();

    // Test hovering when no components are registered
    state.update_hover(10, 10);
    assert_eq!(state.hovered_component, None);
}

#[test]
fn test_update_hover_edge_coordinates() {
    let mut state = OverviewWidgetState::default();

    // Register a component
    let rect = Rect::new(10, 5, 20, 3);
    state
        .registered_components
        .insert(OverviewComponent::Help, rect);

    // Test edge coordinates
    state.update_hover(10, 5); // Top-left corner
    assert_eq!(state.hovered_component, Some(OverviewComponent::Help));

    state.update_hover(29, 7); // Bottom-right corner
    assert_eq!(state.hovered_component, Some(OverviewComponent::Help));

    // Test just outside the rect
    state.update_hover(9, 5); // One pixel left
    assert_eq!(state.hovered_component, None);

    state.update_hover(30, 7); // One pixel right
    assert_eq!(state.hovered_component, None);
}

#[test]
fn test_state_modifications() {
    let mut state = OverviewWidgetState {
        selected_component: OverviewComponent::Settings,
        ..Default::default()
    };

    // Test modifying selected component
    assert_eq!(state.selected_component, OverviewComponent::Settings);

    // Test modifying hovered component
    state.hovered_component = Some(OverviewComponent::Help);
    assert_eq!(state.hovered_component, Some(OverviewComponent::Help));

    // Test modifying help visibility
    state.show_help = true;
    assert!(state.show_help);

    // Test modifying repo info
    state.repo_info.path = "/custom/path".to_string();
    state.repo_info.source_branch = "feature/new".to_string();
    state.repo_info.target_branch = "main".to_string();
    state.repo_info.files_to_analyze = 100;

    assert_eq!(state.repo_info.path, "/custom/path");
    assert_eq!(state.repo_info.source_branch, "feature/new");
    assert_eq!(state.repo_info.target_branch, "main");
    assert_eq!(state.repo_info.files_to_analyze, 100);
}

#[test]
fn test_comprehensive_selection_workflow() {
    let mut state = OverviewWidgetState::default();

    // Test a complete workflow of selecting all components
    let components = [
        OverviewComponent::StartAnalysis,
        OverviewComponent::ViewReports,
        OverviewComponent::Settings,
        OverviewComponent::Credits,
        OverviewComponent::Help,
        OverviewComponent::Exit,
    ];

    // Navigate forward through all components
    for (i, expected_component) in components.iter().enumerate() {
        assert_eq!(state.selected_component, *expected_component);

        if i < components.len() - 1 {
            state.move_selection(SelectionDirection::Next);
        }
    }

    // Test wrapping around
    state.move_selection(SelectionDirection::Next);
    assert_eq!(state.selected_component, OverviewComponent::StartAnalysis);
}

#[test]
fn test_comprehensive_hover_workflow() {
    let mut state = OverviewWidgetState::default();

    // Register all components with non-overlapping rects
    let mut y = 0;
    for component in [
        OverviewComponent::StartAnalysis,
        OverviewComponent::ViewReports,
        OverviewComponent::Settings,
        OverviewComponent::Help,
        OverviewComponent::Exit,
    ] {
        state
            .registered_components
            .insert(component, Rect::new(0, y, 50, 3));
        y += 5;
    }

    // Test hovering over each component
    let test_cases = [
        (25, 1, Some(OverviewComponent::StartAnalysis)),
        (25, 6, Some(OverviewComponent::ViewReports)),
        (25, 11, Some(OverviewComponent::Settings)),
        (25, 16, Some(OverviewComponent::Help)),
        (25, 21, Some(OverviewComponent::Exit)),
        (25, 25, None), // Outside all components
    ];

    for (x, y, expected_hover) in test_cases {
        state.update_hover(x, y);
        assert_eq!(state.hovered_component, expected_hover);
    }
}

#[test]
fn test_registered_components_management() {
    let mut state = OverviewWidgetState::default();

    // Test inserting components
    let rect1 = Rect::new(0, 0, 10, 5);
    let rect2 = Rect::new(10, 0, 10, 5);

    state
        .registered_components
        .insert(OverviewComponent::StartAnalysis, rect1);
    state
        .registered_components
        .insert(OverviewComponent::ViewReports, rect2);

    assert_eq!(state.registered_components.len(), 2);
    assert_eq!(
        state
            .registered_components
            .get(&OverviewComponent::StartAnalysis),
        Some(&rect1)
    );
    assert_eq!(
        state
            .registered_components
            .get(&OverviewComponent::ViewReports),
        Some(&rect2)
    );

    // Test clearing components
    state.registered_components.clear();
    assert!(state.registered_components.is_empty());
}

#[test]
fn test_help_toggle_state() {
    let mut state = OverviewWidgetState::default();

    // Test initial help state
    assert!(!state.show_help);

    // Test toggling help on
    state.show_help = true;
    assert!(state.show_help);

    // Test toggling help off
    state.show_help = false;
    assert!(!state.show_help);
}
