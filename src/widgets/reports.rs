use bevy::prelude::*;
use bevy_ratatui::{error::exit_on_error, terminal::RatatuiContext};
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, StatefulWidgetRef, WidgetRef},
};

use crate::{
    bevy_states::app::AppState,
    events::{app::AppEvent, reports::ReportsEvent},
    theme::THEME,
    widget_states::{
        analysis::AnalysisWidgetState,
        reports::{ExportStatus, ReportFormat, ReportsWidgetState, ViewMode},
    },
};

pub struct ReportsPlugin;

impl Plugin for ReportsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ReportsEvent>()
            .init_resource::<ReportsWidgetState>()
            .add_systems(PreUpdate, reports_event_handler)
            .add_systems(Update, sync_analysis_data)
            .add_systems(Update, render_reports.pipe(exit_on_error));
    }
}

fn sync_analysis_data(
    analysis_state: Res<AnalysisWidgetState>,
    mut reports_state: ResMut<ReportsWidgetState>,
) {
    // Sync review data from analysis to reports
    if let Some(review) = &analysis_state.review {
        if reports_state.review.is_none() {
            reports_state.set_review(review.clone());
        }
    }
}

fn reports_event_handler(
    mut reports_events: EventReader<ReportsEvent>,
    mut reports_state: ResMut<ReportsWidgetState>,
    mut app_events: EventWriter<AppEvent>,
) {
    for event in reports_events.read() {
        match event {
            ReportsEvent::KeyEvent(key_event) => {
                match key_event.code {
                    KeyCode::Esc => {
                        // Handle escape based on current view mode
                        match reports_state.view_mode {
                            ViewMode::Report => {
                                // Go back to selection view
                                reports_state.back_to_selection();
                            }
                            ViewMode::Selection => {
                                // Go back to overview
                                app_events.send(AppEvent::SwitchTo(AppState::Overview));
                            }
                        }
                    }
                    _ => {
                        // Only handle other keys on release to avoid double-triggering
                        if key_event.kind == KeyEventKind::Release {
                            match key_event.code {
                                KeyCode::Left => {
                                    reports_state.previous_format();
                                }
                                KeyCode::Right => {
                                    reports_state.next_format();
                                }
                                KeyCode::Tab => {
                                    reports_state.next_format();
                                }
                                KeyCode::Enter => {
                                    match reports_state.view_mode {
                                        ViewMode::Selection => {
                                            // Generate and show the report
                                            reports_state.generate_report();
                                        }
                                        ViewMode::Report => {
                                            // Export the current report
                                            export_report(&mut reports_state);
                                        }
                                    }
                                }
                                KeyCode::Char('a') => {
                                    app_events.send(AppEvent::SwitchTo(AppState::Analysis));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            ReportsEvent::MouseEvent(_mouse_event) => {
                // Handle mouse events if needed
            }
        }
    }
}

fn export_report(reports_state: &mut ReportsWidgetState) {
    if let Some(_review) = &reports_state.review {
        let format = match reports_state.selected_format {
            ReportFormat::Summary => "summary".to_string(),
            ReportFormat::Detailed => "detailed".to_string(),
            ReportFormat::Json => "json".to_string(),
            ReportFormat::Markdown => "markdown".to_string(),
        };

        reports_state.start_export(format.clone());

        // TODO: Implement actual file export
        let filename = format!(
            "code_review_report.{}",
            match reports_state.selected_format {
                ReportFormat::Json => "json",
                ReportFormat::Markdown => "md",
                _ => "txt",
            }
        );

        reports_state.complete_export(filename);
    }
}

fn render_reports(
    app_state: Res<State<AppState>>,
    mut ratatui_context: ResMut<RatatuiContext>,
    mut reports_state: ResMut<ReportsWidgetState>,
) -> color_eyre::Result<()> {
    if app_state.get() != &AppState::Reports {
        return Ok(());
    }

    ratatui_context.draw(|frame| {
        let area = frame.area();
        frame.render_stateful_widget_ref(ReportsWidget, area, &mut reports_state);
    })?;

    Ok(())
}

struct ReportsWidget;

impl StatefulWidgetRef for ReportsWidget {
    type State = ReportsWidgetState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // Content
                Constraint::Length(3), // Status bar
            ])
            .split(area);

        // Render title
        let title_text = match state.view_mode {
            ViewMode::Selection => "📊 Reports & Export",
            ViewMode::Report => "📄 Generated Report",
        };

        let title = Paragraph::new(title_text)
            .style(THEME.title_style())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(THEME.header_style()),
            );
        title.render_ref(chunks[0], buf);

        // Render content based on view mode
        match state.view_mode {
            ViewMode::Selection => {
                if state.review.is_some() {
                    self.render_report_content(chunks[1], buf, state);
                } else {
                    self.render_no_data(chunks[1], buf);
                }
            }
            ViewMode::Report => {
                self.render_generated_report(chunks[1], buf, state);
            }
        }

        // Render status bar
        self.render_status_bar(chunks[2], buf, state);
    }
}

impl ReportsWidget {
    fn render_no_data(&self, area: Rect, buf: &mut Buffer) {
        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from("No analysis data available"),
            Line::from(""),
            Line::from("Please run an analysis first before generating reports."),
            Line::from(""),
            Line::from("Press 'A' to go to the Analysis screen."),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("No Data")
                .title_style(THEME.warning_style()),
        );

        content.render_ref(area, buf);
    }

    fn render_report_content(&self, area: Rect, buf: &mut Buffer, state: &ReportsWidgetState) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40), // Format selection
                Constraint::Percentage(60), // Preview/Export
            ])
            .split(area);

        // Format selection
        self.render_format_selection(chunks[0], buf, state);

        // Preview/Export area
        self.render_export_area(chunks[1], buf, state);
    }

    fn render_format_selection(&self, area: Rect, buf: &mut Buffer, state: &ReportsWidgetState) {
        let formats = [
            (
                "Summary",
                ReportFormat::Summary,
                "Quick overview with key findings",
            ),
            (
                "Detailed",
                ReportFormat::Detailed,
                "Complete issue breakdown",
            ),
            ("JSON", ReportFormat::Json, "Machine-readable format"),
            (
                "Markdown",
                ReportFormat::Markdown,
                "Documentation-friendly format",
            ),
        ];

        let items: Vec<ListItem> = formats
            .iter()
            .map(|(name, format, description)| {
                let is_selected = *format == state.selected_format;
                let style = if is_selected {
                    THEME.selected_style()
                } else {
                    Style::default()
                };

                ListItem::new(vec![
                    Line::from(vec![Span::styled(
                        *name,
                        if is_selected {
                            THEME.selected_style()
                        } else {
                            THEME.text_primary.into()
                        },
                    )]),
                    Line::from(vec![Span::styled(*description, THEME.info_style())]),
                ])
                .style(style)
            })
            .collect();

        let format_list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Export Format")
                .title_style(THEME.header_style()),
        );

        WidgetRef::render_ref(&format_list, area, buf);
    }

    fn render_export_area(&self, area: Rect, buf: &mut Buffer, state: &ReportsWidgetState) {
        if let Some(review) = &state.review {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(8), // Preview
                    Constraint::Length(5), // Export button
                    Constraint::Min(3),    // Export status
                ])
                .split(area);

            // Preview
            self.render_preview(chunks[0], buf, state, review);

            // Export button
            self.render_export_button(chunks[1], buf);

            // Export status
            self.render_export_status(chunks[2], buf, state);
        }
    }

    fn render_preview(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &ReportsWidgetState,
        review: &crate::core::review::Review,
    ) {
        let preview_content = match state.selected_format {
            ReportFormat::Summary => {
                vec![
                    Line::from("# Code Review Summary"),
                    Line::from(""),
                    Line::from(format!("Files analyzed: {}", review.files_count)),
                    Line::from(format!("Total issues: {}", review.issues_count)),
                    Line::from(format!("Critical: {}", review.critical_issues)),
                    Line::from(format!("High: {}", review.high_issues)),
                ]
            }
            ReportFormat::Detailed => {
                vec![
                    Line::from("# Detailed Code Review Report"),
                    Line::from(""),
                    Line::from("## Issues Found:"),
                    Line::from(format!("- {} Critical issues", review.critical_issues)),
                    Line::from(format!("- {} High priority issues", review.high_issues)),
                    Line::from("(Full details in exported file)"),
                ]
            }
            ReportFormat::Json => {
                vec![
                    Line::from("{"),
                    Line::from(
                        "  \"files_count\": {},".replace("{}", &review.files_count.to_string()),
                    ),
                    Line::from(
                        "  \"issues_count\": {},".replace("{}", &review.issues_count.to_string()),
                    ),
                    Line::from(
                        "  \"critical_issues\": {},"
                            .replace("{}", &review.critical_issues.to_string()),
                    ),
                    Line::from("  \"issues\": [...]"),
                    Line::from("}"),
                ]
            }
            ReportFormat::Markdown => {
                vec![
                    Line::from("# Code Review Report"),
                    Line::from(""),
                    Line::from("## Summary"),
                    Line::from(format!("- **Files analyzed**: {}", review.files_count)),
                    Line::from(format!("- **Total issues**: {}", review.issues_count)),
                    Line::from(""),
                    Line::from("## Issues"),
                ]
            }
        };

        let preview = Paragraph::new(preview_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Preview")
                    .title_style(THEME.header_style()),
            )
            .wrap(ratatui::widgets::Wrap { trim: true });

        preview.render_ref(area, buf);
    }

    fn render_export_button(&self, area: Rect, buf: &mut Buffer) {
        let button = Paragraph::new("� Generate Report (Press Enter)")
            .style(THEME.button_style(false))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(THEME.primary)),
            );

        button.render_ref(area, buf);
    }

    fn render_export_status(&self, area: Rect, buf: &mut Buffer, state: &ReportsWidgetState) {
        let (status_text, status_style) = match &state.export_status {
            ExportStatus::None => ("Ready to export".to_string(), THEME.info_style()),
            ExportStatus::Exporting(format) => (
                format!("Exporting {format} report..."),
                THEME.warning_style(),
            ),
            ExportStatus::Success(path) => (
                format!("✅ Exported successfully to: {path}"),
                THEME.success_style(),
            ),
        };

        let status = Paragraph::new(status_text)
            .style(status_style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Status")
                    .title_style(THEME.header_style()),
            );

        status.render_ref(area, buf);
    }

    fn render_status_bar(&self, area: Rect, buf: &mut Buffer, state: &ReportsWidgetState) {
        let status_text = match state.view_mode {
            ViewMode::Selection => {
                if state.review.is_some() {
                    "Use ←→ or Tab to change format, Enter to generate report, A for analysis, Esc to go back"
                } else {
                    "A to run analysis, Esc to go back"
                }
            }
            ViewMode::Report => "Enter to export report, Esc to go back to selection",
        };

        let status = Paragraph::new(status_text)
            .style(THEME.info_style())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::TOP)
                    .border_style(THEME.info_style()),
            );

        status.render_ref(area, buf);
    }

    fn render_generated_report(&self, area: Rect, buf: &mut Buffer, state: &ReportsWidgetState) {
        if let Some(report_content) = &state.generated_report {
            // Split the report into lines for scrollable display
            let lines: Vec<Line> = report_content
                .lines()
                .map(|line| Line::from(line.to_string()))
                .collect();

            let report = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(
                            " {} Report ",
                            match state.selected_format {
                                ReportFormat::Summary => "Summary",
                                ReportFormat::Detailed => "Detailed",
                                ReportFormat::Json => "JSON",
                                ReportFormat::Markdown => "Markdown",
                            }
                        ))
                        .title_style(THEME.header_style()),
                )
                .wrap(ratatui::widgets::Wrap { trim: false })
                .scroll((0, 0)); // TODO: Add scrolling support

            report.render_ref(area, buf);
        } else {
            let error = Paragraph::new("No report generated")
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("Error")
                        .title_style(THEME.error_style()),
                );
            error.render_ref(area, buf);
        }
    }
}
