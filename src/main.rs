mod widgets {
    pub mod overview;
    pub mod analysis; 
    pub mod reports;
}

mod widget_states {
    pub mod overview;
    pub mod analysis;
    pub mod reports;
}

mod events {
    pub mod app;
    pub mod overview;
    pub mod analysis;
    pub mod reports;
}

mod bevy_states {
    pub mod app;
}

mod theme;
mod args;
mod core;

use std::{error::Error, io::stdout, time::Duration};

use bevy_states::app::AppState;
use events::app::AppEvent;
use widgets::{analysis::AnalysisPlugin, overview::OverviewPlugin, reports::ReportsPlugin};
use clap::Parser;

use bevy::{
    app::ScheduleRunnerPlugin, prelude::*,
    state::app::StatesPlugin,
};
use bevy_ratatui::{
    RatatuiPlugins,
    event::{KeyEvent, MouseEvent},
};
use crossterm::{
    ExecutableCommand,
    cursor::{DisableBlinking, EnableBlinking, SetCursorStyle},
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = args::Args::parse();

    // Handle CLI mode
    if args.cli_mode {
        return core::run_cli_mode(args);
    }

    let frame_rate = Duration::from_secs_f64(1.0 / 60.0);
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    stdout().execute(EnableBlinking)?;
    stdout().execute(SetCursorStyle::BlinkingBar)?;
    stdout().execute(EnableBracketedPaste)?;
    enable_raw_mode()?;

    App::new()
        .add_plugins(bevy::log::LogPlugin::default())
        .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default())
        .add_plugins(RatatuiPlugins {
            enable_mouse_capture: true,
            ..default()
        })
        .add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(frame_rate)))
        .add_plugins(StatesPlugin)
        .insert_resource(args)
        .init_state::<AppState>()
        .add_plugins(OverviewPlugin)
        .add_plugins(AnalysisPlugin)
        .add_plugins(ReportsPlugin)
        .add_systems(Startup, initialize_app)
        .add_systems(PreUpdate, keyboard_events_handler)
        .add_systems(PreUpdate, mouse_events_handler)
        .add_systems(PreUpdate, app_events_handler)
        .add_event::<AppEvent>()
        .run();

    disable_raw_mode()?;
    stdout().execute(DisableBracketedPaste)?;
    stdout().execute(SetCursorStyle::DefaultUserShape)?;
    stdout().execute(DisableBlinking)?;
    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;
    ratatui::restore();
    Ok(())
}

fn initialize_app(
    mut next_state: ResMut<NextState<AppState>>,
    args: Res<args::Args>,
) {
    println!("🚀 AI Code Buddy v0.2.0 - Initializing...");
    println!("📂 Repository: {}", args.repo_path);
    println!("🌿 Branches: {} → {}", args.source_branch, args.target_branch);
    
    next_state.set(AppState::Overview);
}

fn app_events_handler(
    _app_state: Res<State<AppState>>,
    mut send_app_state: ResMut<NextState<AppState>>,
    mut app_events: EventReader<AppEvent>,
    mut app_exit: EventWriter<AppExit>,
) {
    for event in app_events.read() {
        match event {
            AppEvent::SwitchTo(new_state) => {
                send_app_state.set(new_state.clone());
            }
            AppEvent::Exit => {
                app_exit.send_default();
            }
        }
    }
}

fn keyboard_events_handler(
    app_state: Res<State<AppState>>,
    mut keyboard_events: EventReader<KeyEvent>,
    mut overview_events: EventWriter<events::overview::OverviewEvent>,
    mut analysis_events: EventWriter<events::analysis::AnalysisEvent>,
    mut reports_events: EventWriter<events::reports::ReportsEvent>,
    mut app_events: EventWriter<AppEvent>,
) {
    let app_state = app_state.get();
    
    for event in keyboard_events.read() {
        // Global key bindings
        if let crossterm::event::KeyCode::Char('q') = event.code {
            if event.kind == crossterm::event::KeyEventKind::Release {
                app_events.send(AppEvent::Exit);
                continue;
            }
        }

        match app_state {
            AppState::Overview => {
                overview_events.send(events::overview::OverviewEvent::KeyEvent(event.clone()));
            }
            AppState::Analysis => {
                analysis_events.send(events::analysis::AnalysisEvent::KeyEvent(event.clone()));
            }
            AppState::Reports => {
                reports_events.send(events::reports::ReportsEvent::KeyEvent(event.clone()));
            }
        }
    }
}

fn mouse_events_handler(
    app_state: Res<State<AppState>>,
    mut mouse_events: EventReader<MouseEvent>,
    mut overview_events: EventWriter<events::overview::OverviewEvent>,
    mut analysis_events: EventWriter<events::analysis::AnalysisEvent>,
    mut reports_events: EventWriter<events::reports::ReportsEvent>,
) {
    let app_state = app_state.get();
    
    for event in mouse_events.read() {
        match app_state {
            AppState::Overview => {
                overview_events.send(events::overview::OverviewEvent::MouseEvent(event.clone()));
            }
            AppState::Analysis => {
                analysis_events.send(events::analysis::AnalysisEvent::MouseEvent(event.clone()));
            }
            AppState::Reports => {
                reports_events.send(events::reports::ReportsEvent::MouseEvent(event.clone()));
            }
        }
    }
}