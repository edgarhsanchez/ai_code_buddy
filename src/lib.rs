pub mod args;
pub mod core;
pub mod theme;
pub mod version;

// Re-export version constants for convenience
pub use version::{APP_NAME, APP_VERSION};

// Platform guards for GPU features to keep CI/platform builds sane
// NVIDIA CUDA must only be built on Windows runners
#[cfg(all(feature = "gpu-cuda", not(target_os = "windows")))]
compile_error!(
    "The `gpu-cuda` feature is only supported on Windows builds. Remove the feature or run on a Windows runner."
);
// Apple Metal must only be built on macOS
#[cfg(all(feature = "gpu-metal", not(target_os = "macos")))]
compile_error!(
    "The `gpu-metal` feature is only supported on macOS builds. Remove the feature or run on a macOS runner."
);

pub mod widgets {
    pub mod analysis;
    pub mod credits;
    pub mod overview;
    pub mod reports;
}

pub mod widget_states {
    pub mod analysis;
    pub mod credits;
    pub mod overview;
    pub mod reports;
}

pub mod events {
    pub mod analysis;
    pub mod app;
    pub mod credits;
    pub mod overview;
    pub mod reports;
}

pub mod bevy_states {
    pub mod app;
}

// Re-export commonly used types for easier testing
pub use args::{Args, OutputFormat};
pub use core::analysis::perform_analysis;

// Re-export main application functions for testing
pub use main_functions::*;

mod main_functions {
    use crate::{args::Args, bevy_states::app::AppState, events::app::AppEvent};
    use bevy::prelude::*;
    use bevy_ratatui::event::{KeyEvent, MouseEvent};

    pub fn initialize_app(mut next_state: ResMut<NextState<AppState>>, args: Res<Args>) {
        println!("🚀 AI Code Buddy v{} - Initializing...", crate::APP_VERSION);
        println!("📂 Repository: {}", args.repo_path);
        println!(
            "🌿 Branches: {} → {}",
            args.source_branch, args.target_branch
        );

        next_state.set(AppState::Overview);
    }

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

    pub fn keyboard_events_handler(
        app_state: Res<State<AppState>>,
        mut keyboard_events: EventReader<KeyEvent>,
        mut overview_events: EventWriter<crate::events::overview::OverviewEvent>,
        mut analysis_events: EventWriter<crate::events::analysis::AnalysisEvent>,
        mut reports_events: EventWriter<crate::events::reports::ReportsEvent>,
        mut credits_events: EventWriter<crate::events::credits::CreditsEvent>,
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
                    overview_events.send(crate::events::overview::OverviewEvent::KeyEvent(
                        event.clone(),
                    ));
                }
                AppState::Analysis => {
                    analysis_events.send(crate::events::analysis::AnalysisEvent::KeyEvent(
                        event.clone(),
                    ));
                }
                AppState::Reports => {
                    reports_events.send(crate::events::reports::ReportsEvent::KeyEvent(
                        event.clone(),
                    ));
                }
                AppState::Credits => {
                    credits_events.send(crate::events::credits::CreditsEvent::KeyEvent(
                        event.clone(),
                    ));
                }
            }
        }
    }

    pub fn mouse_events_handler(
        app_state: Res<State<AppState>>,
        mut mouse_events: EventReader<MouseEvent>,
        mut overview_events: EventWriter<crate::events::overview::OverviewEvent>,
        mut analysis_events: EventWriter<crate::events::analysis::AnalysisEvent>,
        mut reports_events: EventWriter<crate::events::reports::ReportsEvent>,
        mut credits_events: EventWriter<crate::events::credits::CreditsEvent>,
    ) {
        let app_state = app_state.get();

        for event in mouse_events.read() {
            match app_state {
                AppState::Overview => {
                    overview_events
                        .send(crate::events::overview::OverviewEvent::MouseEvent(*event));
                }
                AppState::Analysis => {
                    analysis_events
                        .send(crate::events::analysis::AnalysisEvent::MouseEvent(*event));
                }
                AppState::Reports => {
                    reports_events.send(crate::events::reports::ReportsEvent::MouseEvent(*event));
                }
                AppState::Credits => {
                    credits_events.send(crate::events::credits::CreditsEvent::MouseEvent(
                        *event,
                    ));
                }
            }
        }
    }
}
pub use core::review::Review;
