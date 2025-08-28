use bevy::prelude::*;
use bevy_ratatui::terminal::RatatuiContext;
use crossterm::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    bevy_states::app::AppState,
    core::credits::{get_library_dependencies, get_project_contributors},
    events::{app::AppEvent, credits::CreditsEvent},
    theme::THEME,
    widget_states::credits::{CreditsComponent, CreditsWidgetState},
};

pub struct CreditsPlugin;

impl Plugin for CreditsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreditsEvent>()
            .init_resource::<CreditsWidgetState>()
            .add_systems(
                PreUpdate,
                credits_event_handler.run_if(in_state(AppState::Credits)),
            )
            .add_systems(Update, render_credits.run_if(in_state(AppState::Credits)));
    }
}

pub fn credits_event_handler(
    mut credits_events: EventReader<CreditsEvent>,
    mut credits_state: ResMut<CreditsWidgetState>,
    mut app_events: EventWriter<AppEvent>,
) {
    for event in credits_events.read() {
        match event {
            CreditsEvent::KeyEvent(key_event) => {
                if key_event.kind == KeyEventKind::Release {
                    match key_event.code {
                        KeyCode::Up => {
                            credits_state.scroll_up();
                        }
                        KeyCode::Down => {
                            credits_state.scroll_down();
                        }
                        KeyCode::PageUp => {
                            credits_state.scroll_offset =
                                credits_state.scroll_offset.saturating_sub(20);
                        }
                        KeyCode::PageDown => {
                            credits_state.scroll_offset += 20;
                        }
                        KeyCode::Home => {
                            credits_state.scroll_offset = 0;
                        }
                        KeyCode::End => {
                            credits_state.scroll_offset =
                                credits_state.total_lines.saturating_sub(20);
                        }
                        KeyCode::Enter | KeyCode::Esc => {
                            app_events.send(AppEvent::SwitchTo(AppState::Overview));
                        }
                        _ => {}
                    }
                }
            }
            CreditsEvent::MouseEvent(mouse_event) => match mouse_event.kind {
                MouseEventKind::Up(_) => {
                    let x = mouse_event.column;
                    let y = mouse_event.row;

                    if credits_state.is_over(CreditsComponent::BackToOverview, x, y) {
                        app_events.send(AppEvent::SwitchTo(AppState::Overview));
                    } else if credits_state.is_over(CreditsComponent::ScrollUp, x, y) {
                        credits_state.scroll_up();
                    } else if credits_state.is_over(CreditsComponent::ScrollDown, x, y) {
                        credits_state.scroll_down();
                    }
                }
                MouseEventKind::ScrollUp => {
                    credits_state.scroll_up();
                }
                MouseEventKind::ScrollDown => {
                    credits_state.scroll_down();
                }
                _ => {}
            },
        }
    }
}

pub fn render_credits(
    mut ratatui: ResMut<RatatuiContext>,
    mut credits_state: ResMut<CreditsWidgetState>,
) {
    let _ = ratatui.draw(|frame| {
        let area = frame.area();

        // Clear the frame
        frame.render_widget(Clear, area);

        // Create main layout
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(10),   // Credits content
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Render header
        let header = Paragraph::new(Line::from(vec![
            Span::styled("🎉 AI Code Buddy", Style::default().fg(THEME.primary)),
            Span::raw(" - Credits & Acknowledgments"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Credits"));

        frame.render_widget(header, main_layout[0]);

        // Render credits content
        let credits_content = generate_credits_content(&mut credits_state);
        frame.render_widget(credits_content, main_layout[1]);

        // Render footer
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("↑/↓", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Scroll • "),
            Span::styled("Enter/Esc", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Back to Overview • "),
            Span::styled("Mouse", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" Click to navigate"),
        ]))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

        frame.render_widget(footer, main_layout[2]);
    });
}

fn generate_credits_content(state: &mut CreditsWidgetState) -> Paragraph {
    let mut lines = Vec::new();

    // Project Information
    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        "📚 About AI Code Buddy",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(
        "An intelligent code analysis tool with elegant Bevy-powered TUI",
    ));
    lines.push(Line::from(
        "that provides comprehensive code reviews with AI assistance.",
    ));
    lines.push(Line::from(
        "Repository: https://github.com/edgarhsanchez/ai_code_buddy",
    ));
    lines.push(Line::from(""));

    // Project Contributors
    lines.push(Line::from(vec![Span::styled(
        "👥 Project Contributors",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from("─────────────────────"));

    let contributors = get_project_contributors();
    for contributor in contributors {
        lines.push(Line::from(format!(
            "  • {} <{}> ({} commits)",
            contributor.name, contributor.email, contributor.contributions
        )));
    }
    lines.push(Line::from(""));

    // Library Dependencies
    lines.push(Line::from(vec![Span::styled(
        "📦 Library Dependencies & Licenses",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from("──────────────────────────────────"));

    let libraries = get_library_dependencies();
    for library in libraries {
        lines.push(Line::from(format!(
            "🔧 {} v{}",
            library.name, library.version
        )));
        lines.push(Line::from(format!("   📄 License: {}", library.license)));
        lines.push(Line::from(format!(
            "   📖 Description: {}",
            library.description
        )));
        lines.push(Line::from(format!(
            "   🔗 Repository: {}",
            library.repository
        )));
        lines.push(Line::from("   👥 Key Contributors:"));

        for contributor in &library.contributors {
            lines.push(Line::from(format!("     • {}", contributor)));
        }
        lines.push(Line::from(""));
    }

    // Special Thanks
    lines.push(Line::from(vec![Span::styled(
        "🙏 Special Thanks",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from("─────────────────"));
    lines.push(Line::from("  • The Rust Programming Language team"));
    lines.push(Line::from("  • All open source contributors"));
    lines.push(Line::from("  • The Bevy game engine community"));
    lines.push(Line::from("  • The broader Rust ecosystem"));
    lines.push(Line::from(""));

    // Call to Action
    lines.push(Line::from(vec![Span::styled(
        "💡 Want to contribute?",
        Style::default().add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(
        "Visit: https://github.com/edgarhsanchez/ai_code_buddy",
    ));
    lines.push(Line::from(
        "🐛 Found a bug? Report it: https://github.com/edgarhsanchez/ai_code_buddy/issues",
    ));

    // Update total lines count
    state.total_lines = lines.len();

    // Create scrollable paragraph
    let scroll = (state.scroll_offset as u16, 0);
    Paragraph::new(lines).scroll(scroll).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Credits (Scrollable)"),
    )
}
