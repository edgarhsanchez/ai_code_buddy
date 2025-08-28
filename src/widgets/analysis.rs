use bevy::prelude::*;
use bevy_ratatui::{error::exit_on_error, terminal::RatatuiContext};
use bevy_tokio_tasks::TokioTasksRuntime;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, StatefulWidgetRef, WidgetRef},
};

use crate::{
    args::Args,
    bevy_states::app::AppState,
    core,
    events::{analysis::AnalysisEvent, app::AppEvent},
    theme::THEME,
    widget_states::analysis::AnalysisWidgetState,
};

pub struct AnalysisPlugin;

impl Plugin for AnalysisPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnalysisEvent>()
            .init_resource::<AnalysisWidgetState>()
            .add_systems(PreUpdate, analysis_event_handler)
            .add_systems(Update, render_analysis.pipe(exit_on_error));
    }
}

pub fn analysis_event_handler(
    mut analysis_events: EventReader<AnalysisEvent>,
    mut analysis_state: ResMut<AnalysisWidgetState>,
    mut app_events: EventWriter<AppEvent>,
    args: Res<Args>,
    tokio_runtime: ResMut<TokioTasksRuntime>,
) {
    for event in analysis_events.read() {
        match event {
            AnalysisEvent::KeyEvent(key_event) => {
                match key_event.code {
                    KeyCode::Esc => {
                        // Always allow going back to overview with Escape
                        // If analysis is running, this will stop it and go back
                        if analysis_state.is_analyzing {
                            analysis_state.is_analyzing = false;
                        }
                        app_events.send(AppEvent::SwitchTo(AppState::Overview));
                    }
                    _ => {
                        // Only handle other keys on release to avoid double-triggering
                        if key_event.kind == KeyEventKind::Release {
                            match key_event.code {
                                KeyCode::Enter => {
                                    if !analysis_state.is_analyzing
                                        && analysis_state.review.is_none()
                                    {
                                        start_analysis(&mut analysis_state, &args, &tokio_runtime);
                                    }
                                }
                                KeyCode::Up => {
                                    if !analysis_state.is_analyzing {
                                        analysis_state.move_issue_selection(-1);
                                    }
                                }
                                KeyCode::Down => {
                                    if !analysis_state.is_analyzing {
                                        analysis_state.move_issue_selection(1);
                                    }
                                }
                                KeyCode::Char('r') => {
                                    if !analysis_state.is_analyzing {
                                        app_events.send(AppEvent::SwitchTo(AppState::Reports));
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            AnalysisEvent::MouseEvent(_mouse_event) => {
                // Handle mouse events if needed
            }
        }
    }
}

fn start_analysis(
    analysis_state: &mut AnalysisWidgetState,
    args: &Args,
    tokio_runtime: &TokioTasksRuntime,
) {
    analysis_state.start_analysis();

    let args = args.clone();
    tokio_runtime.spawn_background_task(|mut ctx| async move {
        // Create a channel for progress updates
        use tokio::sync::mpsc;
        let (progress_tx, mut progress_rx) = mpsc::unbounded_channel();

        // Spawn task to handle progress updates
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let mut ctx = ctx_clone;
            while let Some((progress, current_file)) = progress_rx.recv().await {
                ctx.run_on_main_thread(move |ctx| {
                    if let Some(mut analysis_state) =
                        ctx.world.get_resource_mut::<AnalysisWidgetState>()
                    {
                        analysis_state.update_progress(progress, current_file);
                    }
                })
                .await;
            }
        });

        // Create progress callback that sends to channel
        let progress_callback = {
            let tx = progress_tx.clone();
            Box::new(move |progress: f64, current_file: String| {
                let _ = tx.send((progress, current_file));
            }) as Box<dyn Fn(f64, String) + Send + Sync>
        };

        // Perform actual AI-powered analysis
        match core::analysis::perform_analysis_with_progress(&args, Some(progress_callback)).await {
            Ok(review) => {
                // Close progress channel
                drop(progress_tx);

                ctx.run_on_main_thread(move |ctx| {
                    if let Some(mut analysis_state) =
                        ctx.world.get_resource_mut::<AnalysisWidgetState>()
                    {
                        analysis_state.complete_analysis(review);
                    }
                })
                .await;
            }
            Err(e) => {
                eprintln!("AI analysis failed: {e}");
                drop(progress_tx);

                ctx.run_on_main_thread(move |ctx| {
                    if let Some(mut analysis_state) =
                        ctx.world.get_resource_mut::<AnalysisWidgetState>()
                    {
                        analysis_state.is_analyzing = false;
                    }
                })
                .await;
            }
        }
    });
}

fn render_analysis(
    app_state: Res<State<AppState>>,
    mut ratatui_context: ResMut<RatatuiContext>,
    mut analysis_state: ResMut<AnalysisWidgetState>,
) -> color_eyre::Result<()> {
    if app_state.get() != &AppState::Analysis {
        return Ok(());
    }

    ratatui_context.draw(|frame| {
        let area = frame.area();
        frame.render_stateful_widget_ref(AnalysisWidget, area, &mut analysis_state);
    })?;

    Ok(())
}

pub struct AnalysisWidget;

impl StatefulWidgetRef for AnalysisWidget {
    type State = AnalysisWidgetState;

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
        let title = Paragraph::new("🔍 Code Analysis")
            .style(THEME.title_style())
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(THEME.header_style()),
            );
        title.render_ref(chunks[0], buf);

        // Render content based on state
        if state.is_analyzing {
            self.render_analysis_progress(chunks[1], buf, state);
        } else if let Some(review) = &state.review {
            self.render_results(chunks[1], buf, state, review);
        } else {
            self.render_start_screen(chunks[1], buf);
        }

        // Render status bar
        self.render_status_bar(chunks[2], buf, state);
    }
}

impl AnalysisWidget {
    fn render_start_screen(&self, area: Rect, buf: &mut Buffer) {
        let content = Paragraph::new(vec![
            Line::from(""),
            Line::from("Press Enter to start the code analysis"),
            Line::from(""),
            Line::from("This will analyze your Git repository for:"),
            Line::from("• Security vulnerabilities"),
            Line::from("• Performance issues"),
            Line::from("• Code quality problems"),
            Line::from("• Best practice violations"),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Ready to Analyze")
                .title_style(THEME.header_style()),
        );

        content.render_ref(area, buf);
    }

    fn render_analysis_progress(&self, area: Rect, buf: &mut Buffer, state: &AnalysisWidgetState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Progress bar
                Constraint::Min(3),    // Current file
            ])
            .split(area);

        // Progress bar
        let progress = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Analysis Progress")
                    .title_style(THEME.header_style()),
            )
            .gauge_style(THEME.success_style())
            .percent(state.progress as u16)
            .label(format!("{:.1}%", state.progress));

        progress.render_ref(chunks[0], buf);

        // Current file
        let current_file = Paragraph::new(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Currently analyzing: ", THEME.info_style()),
                Span::raw(&state.current_file),
            ]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Status")
                .title_style(THEME.header_style()),
        );

        current_file.render_ref(chunks[1], buf);
    }

    fn render_results(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &AnalysisWidgetState,
        review: &crate::core::review::Review,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Summary
                Constraint::Percentage(70), // Issue list
            ])
            .split(area);

        // Summary
        self.render_summary(chunks[0], buf, review);

        // Issue list
        self.render_issue_list(chunks[1], buf, state, review);
    }

    fn render_summary(&self, area: Rect, buf: &mut Buffer, review: &crate::core::review::Review) {
        let summary_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("📁 Files: ", THEME.info_style()),
                Span::raw(format!("{}", review.files_count)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🐛 Total Issues: ", THEME.info_style()),
                Span::raw(format!("{}", review.issues_count)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("🚨 Critical: ", THEME.error_style()),
                Span::raw(format!("{}", review.critical_issues)),
            ]),
            Line::from(vec![
                Span::styled("⚠️  High: ", THEME.warning_style()),
                Span::raw(format!("{}", review.high_issues)),
            ]),
            Line::from(vec![
                Span::styled("🔶 Medium: ", THEME.warning_style()),
                Span::raw(format!("{}", review.medium_issues)),
            ]),
            Line::from(vec![
                Span::styled("ℹ️  Low: ", THEME.info_style()),
                Span::raw(format!("{}", review.low_issues)),
            ]),
        ];

        let summary = Paragraph::new(summary_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Summary")
                .title_style(THEME.header_style()),
        );

        summary.render_ref(area, buf);
    }

    fn render_issue_list(
        &self,
        area: Rect,
        buf: &mut Buffer,
        state: &AnalysisWidgetState,
        review: &crate::core::review::Review,
    ) {
        if review.issues.is_empty() {
            let no_issues = Paragraph::new(vec![
                Line::from(""),
                Line::from("🎉 No issues found!"),
                Line::from(""),
                Line::from("Your code looks clean. Great job!"),
            ])
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Issues")
                    .title_style(THEME.header_style()),
            );
            no_issues.render_ref(area, buf);
            return;
        }

        let items: Vec<ListItem> = review
            .issues
            .iter()
            .enumerate()
            .map(|(i, issue)| {
                let severity_icon = match issue.severity.as_str() {
                    "Critical" => "🚨",
                    "High" => "⚠️",
                    "Medium" => "🔶",
                    "Low" => "ℹ️",
                    _ => "💡",
                };

                let severity_style = match issue.severity.as_str() {
                    "Critical" => THEME.error_style(),
                    "High" => THEME.warning_style(),
                    "Medium" => THEME.warning_style(),
                    "Low" => THEME.info_style(),
                    _ => Style::default(),
                };

                let is_selected = i == state.selected_issue;

                // Create a multi-line item for better readability
                let lines = vec![
                    Line::from(vec![
                        Span::styled(format!("{severity_icon} "), severity_style),
                        Span::styled(issue.severity.to_string(), severity_style),
                        Span::raw("  "),
                        Span::styled(format!("{}:{}", issue.file, issue.line), THEME.info_style()),
                    ]),
                    Line::from(vec![
                        Span::raw("   "),
                        Span::styled(format!("{}: ", issue.category), THEME.header_style()),
                        Span::raw(issue.description.to_string()),
                    ]),
                    Line::from(""), // Empty line for spacing
                ];

                let style = if is_selected {
                    THEME.selected_style()
                } else {
                    Style::default()
                };

                ListItem::new(lines).style(style)
            })
            .collect();

        let issue_list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(
                        "Issues ({}/{})",
                        state.selected_issue + 1,
                        review.issues.len().max(1)
                    ))
                    .title_style(THEME.header_style()),
            )
            .highlight_style(THEME.selected_style());

        WidgetRef::render_ref(&issue_list, area, buf);
    }

    fn render_status_bar(&self, area: Rect, buf: &mut Buffer, state: &AnalysisWidgetState) {
        let status_text = if state.is_analyzing {
            "Analysis in progress... Please wait"
        } else if state.review.is_some() {
            "Use ↑↓ to navigate issues, R for reports, Esc to go back"
        } else {
            "Enter to start analysis, Esc to go back"
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
}
