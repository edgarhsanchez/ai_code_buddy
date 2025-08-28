use ai_code_buddy::{
    app_events_handler,
    args::{Args, OutputFormat},
    bevy_states::app::AppState,
    events::{
        analysis::AnalysisEvent, app::AppEvent, overview::OverviewEvent, reports::ReportsEvent,
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
