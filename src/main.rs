mod widgets {
    pub mod analysis;
    pub mod credits;
    pub mod overview;
    pub mod reports;
}

mod widget_states {
    pub mod analysis;
    pub mod credits;
    pub mod overview;
    pub mod reports;
}

mod events {
    pub mod analysis;
    pub mod app;
    pub mod credits;
    pub mod overview;
    pub mod reports;
}

mod bevy_states {
    pub mod app;
}

mod args;
mod core;
mod theme;
mod version;

use std::{error::Error, io::stdout, time::Duration};

use bevy_states::app::AppState;
use clap::Parser;
use events::app::AppEvent;
use widgets::{
    analysis::AnalysisPlugin, credits::CreditsPlugin, overview::OverviewPlugin,
    reports::ReportsPlugin,
};

use bevy::{app::ScheduleRunnerPlugin, prelude::*, state::app::StatesPlugin};
use bevy_ratatui::{
    event::{KeyEvent, MouseEvent},
    RatatuiPlugins,
};
use crossterm::{
    cursor::{DisableBlinking, EnableBlinking, SetCursorStyle},
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args = args::Args::parse();

    // Handle special flags first
    if args.list_models {
        args::Args::print_model_help();
        return Ok(());
    }

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
        .add_plugins(CreditsPlugin)
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

#[cfg_attr(test, allow(dead_code))]
pub fn initialize_app(mut next_state: ResMut<NextState<AppState>>, args: Res<args::Args>) {
    println!(
        "🚀 AI Code Buddy v{} - Initializing...",
        crate::version::APP_VERSION
    );
    println!("📂 Repository: {}", args.repo_path);
    println!(
        "🌿 Comparing: {} → {}",
        args.get_source_branch(&args.repo_path), args.get_target_branch()
    );

    next_state.set(AppState::Overview);
}

#[cfg_attr(test, allow(dead_code))]
pub fn app_events_handler(
    _app_state: Res<State<AppState>>,
    mut send_app_state: ResMut<NextState<AppState>>,
    mut app_events: EventReader<AppEvent>,
    mut app_exit: EventWriter<AppExit>,
) {
    for event in app_events.read() {
        match event {
            AppEvent::SwitchTo(new_state) => {
                send_app_state.set(*new_state);
            }
            AppEvent::Exit => {
                app_exit.send_default();
            }
        }
    }
}

#[cfg_attr(test, allow(dead_code))]
pub fn keyboard_events_handler(
    app_state: Res<State<AppState>>,
    mut keyboard_events: EventReader<KeyEvent>,
    mut overview_events: EventWriter<events::overview::OverviewEvent>,
    mut analysis_events: EventWriter<events::analysis::AnalysisEvent>,
    mut reports_events: EventWriter<events::reports::ReportsEvent>,
    mut credits_events: EventWriter<events::credits::CreditsEvent>,
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
            AppState::Credits => {
                credits_events.send(events::credits::CreditsEvent::KeyEvent(event.clone()));
            }
        }
    }
}

#[cfg_attr(test, allow(dead_code))]
#[cfg_attr(test, allow(dead_code))]
pub fn mouse_events_handler(
    app_state: Res<State<AppState>>,
    mut mouse_events: EventReader<MouseEvent>,
    mut overview_events: EventWriter<events::overview::OverviewEvent>,
    mut analysis_events: EventWriter<events::analysis::AnalysisEvent>,
    mut reports_events: EventWriter<events::reports::ReportsEvent>,
    mut credits_events: EventWriter<events::credits::CreditsEvent>,
) {
    let app_state = app_state.get();

    for event in mouse_events.read() {
        match app_state {
            AppState::Overview => {
                overview_events.send(events::overview::OverviewEvent::MouseEvent(*event));
            }
            AppState::Analysis => {
                analysis_events.send(events::analysis::AnalysisEvent::MouseEvent(*event));
            }
            AppState::Reports => {
                reports_events.send(events::reports::ReportsEvent::MouseEvent(*event));
            }
            AppState::Credits => {
                credits_events.send(events::credits::CreditsEvent::MouseEvent(*event));
            }
        }
    }
}
