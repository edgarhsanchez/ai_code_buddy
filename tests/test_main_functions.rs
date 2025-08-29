use ai_code_buddy::{
    app_events_handler,
    args::{Args, OutputFormat},
    bevy_states::app::AppState,
    events::{
        analysis::AnalysisEvent, app::AppEvent, credits::CreditsEvent, overview::OverviewEvent,
        reports::ReportsEvent,
    },
    initialize_app, keyboard_events_handler, mouse_events_handler,
};
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_ratatui::event::{KeyEvent, MouseEvent};

fn build_app_with_state() -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin)
        .add_event::<AppEvent>()
        .add_event::<KeyEvent>()
        .add_event::<MouseEvent>()
        .add_event::<OverviewEvent>()
        .add_event::<AnalysisEvent>()
        .add_event::<ReportsEvent>()
        .add_event::<CreditsEvent>()
        .init_state::<AppState>()
        .insert_resource(Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "HEAD".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: OutputFormat::Summary,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: true,
            parallel: false,
            disable_ai: false,
        });
    app
}

#[test]
fn test_initialize_app_sets_state() {
    let mut app = build_app_with_state();
    app.add_systems(Update, initialize_app);
    app.update();
    let state = app.world().resource::<State<AppState>>();
    assert_eq!(state.get(), &AppState::Overview);
}

#[test]
fn test_app_events_handler_switch_and_exit() {
    let mut app = build_app_with_state();
    app.add_event::<AppExit>();
    app.add_systems(Update, app_events_handler);

    // Switch to Analysis
    {
        let mut ev = app.world_mut().resource_mut::<Events<AppEvent>>();
        ev.send(AppEvent::SwitchTo(AppState::Analysis));
    }
    app.update();
    // State transitions apply after the frame; run another frame to finalize transition
    app.update();
    let state = app.world().resource::<State<AppState>>();
    assert_eq!(state.get(), &AppState::Analysis);

    // Send Exit
    {
        let mut ev = app.world_mut().resource_mut::<Events<AppEvent>>();
        ev.send(AppEvent::Exit);
    }
    app.update();
    let exit_events = app.world().resource::<Events<AppExit>>();
    let cursor = exit_events.get_cursor();
    assert!(cursor.len(exit_events) > 0);
}

#[test]
fn test_keyboard_events_routed_by_state() {
    let mut app = build_app_with_state();
    app.add_systems(Update, keyboard_events_handler);

    // Start in Overview, send a KeyEvent and ensure it's routed to OverviewEvent bus
    {
        let mut keys = app.world_mut().resource_mut::<Events<KeyEvent>>();
        // Wrap a crossterm KeyEvent in the bevy_ratatui KeyEvent tuple struct
        let ct_key = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Tab,
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        };
        keys.send(KeyEvent(ct_key));
    }
    app.update();
    let overview_bus = app.world().resource::<Events<OverviewEvent>>();
    let cursor = overview_bus.get_cursor();
    assert!(cursor.len(overview_bus) > 0);

    // Change state to Reports and send another KeyEvent
    {
        let mut next = app.world_mut().resource_mut::<NextState<AppState>>();
        next.set(AppState::Reports);
    }
    app.update();
    {
        let mut keys = app.world_mut().resource_mut::<Events<KeyEvent>>();
        let ct_key = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Right,
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        };
        keys.send(KeyEvent(ct_key));
    }
    app.update();
    let reports_bus = app.world().resource::<Events<ReportsEvent>>();
    let cursor = reports_bus.get_cursor();
    assert!(cursor.len(reports_bus) > 0);
}

#[test]
fn test_keyboard_events_quit_key() {
    let mut app = build_app_with_state();
    app.add_systems(Update, keyboard_events_handler);

    // Send 'q' key release event (quit key)
    {
        let mut keys = app.world_mut().resource_mut::<Events<KeyEvent>>();
        let ct_key = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('q'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Release,
            state: crossterm::event::KeyEventState::NONE,
        };
        keys.send(KeyEvent(ct_key));
    }
    app.update();

    // Check that AppEvent::Exit was sent
    let app_events = app.world().resource::<Events<AppEvent>>();
    let cursor = app_events.get_cursor();
    assert!(cursor.len(app_events) > 0);
}

#[test]
fn test_keyboard_events_quit_key_press_ignored() {
    let mut app = build_app_with_state();
    app.add_systems(Update, keyboard_events_handler);

    // Send 'q' key press event (should be ignored, only release triggers)
    {
        let mut keys = app.world_mut().resource_mut::<Events<KeyEvent>>();
        let ct_key = crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('q'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        keys.send(KeyEvent(ct_key));
    }
    app.update();

    // Check that no AppEvent::Exit was sent
    let app_events = app.world().resource::<Events<AppEvent>>();
    let cursor = app_events.get_cursor();
    assert_eq!(cursor.len(app_events), 0);
}

#[test]
fn test_app_events_handler_multiple_events() {
    let mut app = build_app_with_state();
    app.add_event::<AppExit>();
    app.add_systems(Update, app_events_handler);

    // Send multiple events
    {
        let mut ev = app.world_mut().resource_mut::<Events<AppEvent>>();
        ev.send(AppEvent::SwitchTo(AppState::Analysis));
        ev.send(AppEvent::SwitchTo(AppState::Reports));
        ev.send(AppEvent::Exit);
    }
    app.update();

    // Check that all events were processed
    let exit_events = app.world().resource::<Events<AppExit>>();
    let cursor = exit_events.get_cursor();
    assert!(cursor.len(exit_events) > 0);
}

#[test]
fn test_state_transitions_overview_to_analysis() {
    let mut app = build_app_with_state();
    app.add_systems(Update, app_events_handler);

    // Start in Overview
    let state = app.world().resource::<State<AppState>>();
    assert_eq!(state.get(), &AppState::Overview);

    // Switch to Analysis
    {
        let mut ev = app.world_mut().resource_mut::<Events<AppEvent>>();
        ev.send(AppEvent::SwitchTo(AppState::Analysis));
    }
    app.update();
    app.update(); // State transitions apply after the frame

    let state = app.world().resource::<State<AppState>>();
    assert_eq!(state.get(), &AppState::Analysis);
}

#[test]
fn test_state_transitions_analysis_to_reports() {
    let mut app = build_app_with_state();
    app.add_systems(Update, app_events_handler);

    // Switch to Analysis first
    {
        let mut ev = app.world_mut().resource_mut::<Events<AppEvent>>();
        ev.send(AppEvent::SwitchTo(AppState::Analysis));
    }
    app.update();
    app.update();

    // Then switch to Reports
    {
        let mut ev = app.world_mut().resource_mut::<Events<AppEvent>>();
        ev.send(AppEvent::SwitchTo(AppState::Reports));
    }
    app.update();
    app.update();

    let state = app.world().resource::<State<AppState>>();
    assert_eq!(state.get(), &AppState::Reports);
}

#[test]
fn test_state_transitions_reports_to_overview() {
    let mut app = build_app_with_state();
    app.add_systems(Update, app_events_handler);

    // Switch to Reports first
    {
        let mut ev = app.world_mut().resource_mut::<Events<AppEvent>>();
        ev.send(AppEvent::SwitchTo(AppState::Reports));
    }
    app.update();
    app.update();

    // Then switch back to Overview
    {
        let mut ev = app.world_mut().resource_mut::<Events<AppEvent>>();
        ev.send(AppEvent::SwitchTo(AppState::Overview));
    }
    app.update();
    app.update();

    let state = app.world().resource::<State<AppState>>();
    assert_eq!(state.get(), &AppState::Overview);
}

#[test]
fn test_mouse_events_different_states() {
    let mut app = build_app_with_state();
    app.add_systems(Update, mouse_events_handler);

    // Test mouse events in different states
    let states = vec![AppState::Overview, AppState::Analysis, AppState::Reports];

    for target_state in states {
        // Change state
        {
            let mut next = app.world_mut().resource_mut::<NextState<AppState>>();
            next.set(target_state.clone());
        }
        app.update();

        // Send mouse event
        {
            let mut mouse = app.world_mut().resource_mut::<Events<MouseEvent>>();
            let ct_mouse = crossterm::event::MouseEvent {
                kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
                column: 5,
                row: 5,
                modifiers: crossterm::event::KeyModifiers::NONE,
            };
            mouse.send(MouseEvent(ct_mouse));
        }
        app.update();

        // Verify event was routed to correct bus
        match target_state {
            AppState::Overview => {
                let overview_bus = app.world().resource::<Events<OverviewEvent>>();
                let cursor = overview_bus.get_cursor();
                assert!(cursor.len(overview_bus) > 0);
            }
            AppState::Analysis => {
                let analysis_bus = app.world().resource::<Events<AnalysisEvent>>();
                let cursor = analysis_bus.get_cursor();
                assert!(cursor.len(analysis_bus) > 0);
            }
            AppState::Reports => {
                let reports_bus = app.world().resource::<Events<ReportsEvent>>();
                let cursor = reports_bus.get_cursor();
                assert!(cursor.len(reports_bus) > 0);
            }
            AppState::Credits => {
                // Credits state doesn't have a specific event bus in this test
                // The test would need to be updated if credits events are added
            }
        }
    }
}

#[test]
fn test_initialize_app_with_different_args() {
    let mut app = App::new();
    let args = Args {
        repo_path: "/custom/path".to_string(),
        source_branch: "develop".to_string(),
        target_branch: "feature-branch".to_string(),
        cli_mode: false,
        verbose: true,
        show_credits: false,
        output_format: OutputFormat::Detailed,
        exclude_patterns: vec!["*.tmp".to_string()],
        include_patterns: vec!["*.rs".to_string()],
        use_gpu: true,
        force_cpu: false,
        parallel: false,
        disable_ai: false,
    };

    app.add_plugins(StatesPlugin)
        .add_event::<AppEvent>()
        .init_state::<AppState>()
        .insert_resource(args.clone())
        .add_systems(Update, initialize_app);

    app.update();

    // Verify args are accessible
    let stored_args = app.world().resource::<Args>();
    assert_eq!(stored_args.repo_path, "/custom/path");
    assert_eq!(stored_args.source_branch, "develop");
    assert_eq!(stored_args.target_branch, "feature-branch");
    assert!(stored_args.use_gpu);
}

#[test]
fn test_mouse_events_routed_by_state() {
    let mut app = build_app_with_state();
    app.add_systems(Update, mouse_events_handler);

    // Start in Overview, send a MouseEvent and ensure it's routed
    {
        let mut mouse = app.world_mut().resource_mut::<Events<MouseEvent>>();
        let ct_mouse = crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Moved,
            column: 1,
            row: 1,
            modifiers: crossterm::event::KeyModifiers::NONE,
        };
        mouse.send(MouseEvent(ct_mouse));
    }
    app.update();
    let overview_bus = app.world().resource::<Events<OverviewEvent>>();
    let cursor = overview_bus.get_cursor();
    assert!(cursor.len(overview_bus) > 0);
}
