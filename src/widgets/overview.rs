use bevy::prelude::*;
use bevy_ratatui::{error::exit_on_error, terminal::RatatuiContext};
use crossterm::event::{KeyCode, KeyEventKind, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, StatefulWidgetRef, WidgetRef},
};

use crate::{
    args::Args,
    bevy_states::app::AppState,
    events::{app::AppEvent, overview::OverviewEvent},
    theme::THEME,
    version::APP_VERSION,
    widget_states::overview::{OverviewComponent, OverviewWidgetState, SelectionDirection},
};

pub struct OverviewPlugin;

impl Plugin for OverviewPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OverviewEvent>()
            .init_resource::<OverviewWidgetState>()
            .add_systems(Startup, initialize_overview_state)
            .add_systems(PreUpdate, overview_event_handler)
            .add_systems(Update, render_overview.pipe(exit_on_error));
    }
}

fn initialize_overview_state(mut overview_state: ResMut<OverviewWidgetState>, args: Res<Args>) {
    overview_state.repo_info.path = args.repo_path.clone();
    overview_state.repo_info.source_branch = args.source_branch.clone();
    overview_state.repo_info.target_branch = args.target_branch.clone();

    // TODO: Calculate files to analyze
    overview_state.repo_info.files_to_analyze = 42; // Placeholder
}

fn overview_event_handler(
    mut overview_events: EventReader<OverviewEvent>,
    mut overview_state: ResMut<OverviewWidgetState>,
    mut app_events: EventWriter<AppEvent>,
) {
    for event in overview_events.read() {
        match event {
            OverviewEvent::KeyEvent(key_event) => {
                if key_event.kind == KeyEventKind::Release {
                    // If help is showing, any key closes it
                    if overview_state.show_help {
                        overview_state.show_help = false;
                        return;
                    }

                    match key_event.code {
                        KeyCode::Tab => {
                            overview_state.move_selection(SelectionDirection::Next);
                        }
                        KeyCode::BackTab => {
                            overview_state.move_selection(SelectionDirection::Previous);
                        }
                        KeyCode::Up => {
                            overview_state.move_selection(SelectionDirection::Previous);
                        }
                        KeyCode::Down => {
                            overview_state.move_selection(SelectionDirection::Next);
                        }
                        KeyCode::Enter => match overview_state.selected_component {
                            OverviewComponent::Help => {
                                overview_state.show_help = !overview_state.show_help;
                            }
                            _ => {
                                handle_selection(
                                    &overview_state.selected_component,
                                    &mut app_events,
                                );
                            }
                        },
                        _ => {}
                    }
                }
            }
            OverviewEvent::MouseEvent(mouse_event) => {
                // If help is showing, any click closes it
                if overview_state.show_help {
                    if let MouseEventKind::Up(_) = mouse_event.kind {
                        overview_state.show_help = false;
                    }
                    return;
                }

                match mouse_event.kind {
                    MouseEventKind::Up(_) => {
                        let x = mouse_event.column;
                        let y = mouse_event.row;

                        let components: Vec<_> = overview_state
                            .registered_components
                            .clone()
                            .into_iter()
                            .collect();
                        for (component, _rect) in components {
                            if overview_state.is_over(component.clone(), x, y) {
                                overview_state.selected_component = component.clone();
                                match component {
                                    OverviewComponent::Help => {
                                        overview_state.show_help = !overview_state.show_help;
                                    }
                                    _ => {
                                        handle_selection(&component, &mut app_events);
                                    }
                                }
                                break;
                            }
                        }
                    }
                    MouseEventKind::Moved => {
                        let x = mouse_event.column;
                        let y = mouse_event.row;
                        overview_state.update_hover(x, y);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn handle_selection(component: &OverviewComponent, app_events: &mut EventWriter<AppEvent>) {
    match component {
        OverviewComponent::StartAnalysis => {
            app_events.send(AppEvent::SwitchTo(AppState::Analysis));
        }
        OverviewComponent::ViewReports => {
            app_events.send(AppEvent::SwitchTo(AppState::Reports));
        }
        OverviewComponent::Settings => {
            // TODO: Implement settings
        }
        OverviewComponent::Help => {
            // Show help dialog - for now we'll add this as a state toggle
            // In a real implementation, this might open a help dialog
        }
        OverviewComponent::Exit => {
            app_events.send(AppEvent::Exit);
        }
    }
}

fn render_overview(
    app_state: Res<State<AppState>>,
    mut ratatui_context: ResMut<RatatuiContext>,
    mut overview_state: ResMut<OverviewWidgetState>,
) -> color_eyre::Result<()> {
    if app_state.get() != &AppState::Overview {
        return Ok(());
    }

    ratatui_context.draw(|frame| {
        let area = frame.area();
        frame.render_stateful_widget_ref(OverviewWidget, area, &mut overview_state);
    })?;

    Ok(())
}

struct OverviewWidget;

impl StatefulWidgetRef for OverviewWidget {
    type State = OverviewWidgetState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.registered_components.clear();

        if state.show_help {
            self.render_help_overlay(area, buf, state);
            return;
        }

        // Main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Length(8), // Repository info
                Constraint::Min(10),   // Menu buttons
                Constraint::Length(3), // Status bar
            ])
            .split(area);

        // Render title
        self.render_title(chunks[0], buf);

        // Render repository info
        self.render_repo_info(chunks[1], buf, state);

        // Render menu
        self.render_menu(chunks[2], buf, state);

        // Render status bar
        self.render_status_bar(chunks[3], buf);
    }
}

impl OverviewWidget {
    fn render_title(&self, area: Rect, buf: &mut Buffer) {
        let title = Paragraph::new(format!("🤖 AI Code Buddy v{}", APP_VERSION))
            .style(THEME.title_style())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(THEME.header_style()),
            );
        title.render_ref(area, buf);
    }

    fn render_repo_info(&self, area: Rect, buf: &mut Buffer, state: &OverviewWidgetState) {
        let info_lines = vec![
            Line::from(vec![
                Span::styled("📂 Repository: ", THEME.info_style()),
                Span::raw(&state.repo_info.path),
            ]),
            Line::from(vec![
                Span::styled("🌿 Source Branch: ", THEME.info_style()),
                Span::raw(&state.repo_info.source_branch),
            ]),
            Line::from(vec![
                Span::styled("🎯 Target Branch: ", THEME.info_style()),
                Span::raw(&state.repo_info.target_branch),
            ]),
            Line::from(vec![
                Span::styled("📊 Files to Analyze: ", THEME.info_style()),
                Span::raw(format!("{}", state.repo_info.files_to_analyze)),
            ]),
        ];

        let repo_info = Paragraph::new(info_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Repository Information")
                    .title_style(THEME.header_style()),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        repo_info.render_ref(area, buf);
    }

    fn render_menu(&self, area: Rect, buf: &mut Buffer, state: &mut OverviewWidgetState) {
        // Center the menu items
        let menu_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area);

        let menu_area = menu_layout[1];

        let items_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Start Analysis
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // View Reports
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Settings
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Help
                Constraint::Length(1), // Spacer
                Constraint::Length(3), // Exit
            ])
            .split(menu_area);

        self.render_menu_button(
            items_layout[0],
            buf,
            state,
            OverviewComponent::StartAnalysis,
            "🚀 Start Analysis",
        );

        self.render_menu_button(
            items_layout[2],
            buf,
            state,
            OverviewComponent::ViewReports,
            "📊 View Reports",
        );

        self.render_menu_button(
            items_layout[4],
            buf,
            state,
            OverviewComponent::Settings,
            "⚙️  Settings",
        );

        self.render_menu_button(
            items_layout[6],
            buf,
            state,
            OverviewComponent::Help,
            "❓ Help",
        );

        self.render_menu_button(
            items_layout[8],
            buf,
            state,
            OverviewComponent::Exit,
            "🚪 Exit",
        );
    }

    fn render_menu_button(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut OverviewWidgetState,
        component: OverviewComponent,
        text: &str,
    ) {
        let is_selected = state.selected_component == component;
        let is_hovered = state.hovered_component == Some(component.clone());

        let style = if is_selected {
            THEME.selected_style()
        } else if is_hovered {
            THEME.button_hover_style()
        } else {
            THEME.button_normal_style()
        };

        let border_style = if is_selected {
            THEME.selected_style()
        } else if is_hovered {
            THEME.button_hover_style()
        } else {
            Style::default()
        };

        let button = Paragraph::new(text)
            .style(style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style),
            );

        button.render_ref(area, buf);
        state.registered_components.insert(component, area);
    }

    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        let status = Paragraph::new("Use ↑↓ or Tab to navigate, Enter to select, Q to quit")
            .style(THEME.info_style())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(THEME.info_style()),
            );

        status.render_ref(area, buf);
    }

    fn render_help_overlay(&self, area: Rect, buf: &mut Buffer, state: &mut OverviewWidgetState) {
        // Create a centered help dialog
        let help_area = {
            let vertical = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(area);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(15),
                    Constraint::Percentage(70),
                    Constraint::Percentage(15),
                ])
                .split(vertical[1])[1]
        };

        // Clear the background
        for y in help_area.top()..help_area.bottom() {
            for x in help_area.left()..help_area.right() {
                buf.cell_mut((x, y)).unwrap().set_bg(THEME.background);
            }
        }

        let help_content = vec![
            Line::from("🤖 AI Code Buddy - Help"),
            Line::from(""),
            Line::from("🎯 What it does:"),
            Line::from("  • Analyzes Git repositories for code quality issues"),
            Line::from("  • Detects security vulnerabilities (OWASP Top 10)"),
            Line::from("  • Provides performance and maintainability suggestions"),
            Line::from("  • Compares code changes between Git branches"),
            Line::from(""),
            Line::from("⌨️  Keyboard Controls:"),
            Line::from("  • ↑/↓ or Tab/Shift+Tab: Navigate menu"),
            Line::from("  • Enter: Select menu item"),
            Line::from("  • q: Quit application"),
            Line::from(""),
            Line::from("🖱️  Mouse Controls:"),
            Line::from("  • Click: Select menu item"),
            Line::from("  • Hover: Highlight menu item"),
            Line::from(""),
            Line::from("📋 Menu Options:"),
            Line::from("  • 🚀 Start Analysis: Begin analyzing the repository"),
            Line::from("  • 📊 View Reports: See analysis results and export"),
            Line::from("  • ⚙️  Settings: Configure analysis options"),
            Line::from("  • ❓ Help: Show this help screen"),
            Line::from("  • 🚪 Exit: Quit the application"),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key or click anywhere to close help",
                Style::default()
                    .fg(THEME.accent)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        let help_dialog = Paragraph::new(help_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Help & Controls ")
                    .title_style(THEME.title_style())
                    .border_style(THEME.primary_style()),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        help_dialog.render_ref(help_area, buf);

        // Register the entire help area as clickable to close help
        state
            .registered_components
            .insert(OverviewComponent::Help, help_area);
    }
}
