use crate::review::{CodeIssue, IssueCategory, Review, Severity};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Tabs, Wrap,
    },
    Frame, Terminal,
};
use std::{
    collections::HashMap,
    io::{self, stdout, Stdout},
};

#[derive(Debug, Clone, PartialEq)]
pub enum AppTab {
    Overview,
    Issues,
    Files,
    Reports,
    Help,
}

#[derive(Debug)]
pub struct App {
    pub review: Review,
    pub current_tab: AppTab,
    pub should_quit: bool,
    pub all_issues: Vec<CodeIssue>,
    pub filtered_issues: Vec<usize>,
    pub selected_issue_index: usize,
    pub selected_file_index: usize,
    pub scroll_position: usize,
    pub issue_list_state: ListState,
    pub file_list_state: ListState,
    pub show_issue_detail: bool,
    pub status_message: Option<String>,
    pub selected_category: Option<IssueCategory>,
    pub selected_severity: Option<Severity>,
    pub help_scroll: usize,
}

impl App {
    pub fn new(review: Review) -> Self {
        let all_issues = Self::collect_all_issues(&review);
        let filtered_issues = (0..all_issues.len()).collect();
        let mut issue_list_state = ListState::default();
        if !filtered_issues.is_empty() {
            issue_list_state.select(Some(0));
        }
        let mut file_list_state = ListState::default();
        file_list_state.select(Some(0));

        Self {
            review,
            current_tab: AppTab::Overview,
            should_quit: false,
            all_issues,
            filtered_issues,
            selected_issue_index: 0,
            selected_file_index: 0,
            scroll_position: 0,
            issue_list_state,
            file_list_state,
            show_issue_detail: false,
            status_message: None,
            selected_category: None,
            selected_severity: None,
            help_scroll: 0,
        }
    }

    fn collect_all_issues(review: &Review) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for category_issues in review.issues.values() {
            for issue in category_issues {
                issues.push(issue.clone());
            }
        }
        // Sort by severity (Critical first)
        issues.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Critical => 0,
                Severity::High => 1,
                Severity::Medium => 2,
                Severity::Low => 3,
                Severity::Info => 4,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });
        issues
    }

    pub fn apply_filters(&mut self) {
        self.filtered_issues = self
            .all_issues
            .iter()
            .enumerate()
            .filter(|(_, issue)| {
                let category_match = self
                    .selected_category
                    .as_ref()
                    .map_or(true, |cat| &issue.category == cat);
                let severity_match = self
                    .selected_severity
                    .as_ref()
                    .map_or(true, |sev| &issue.severity == sev);
                category_match && severity_match
            })
            .map(|(idx, _)| idx)
            .collect();

        if self.selected_issue_index >= self.filtered_issues.len() {
            self.selected_issue_index = 0;
        }
        
        // Update list state
        if !self.filtered_issues.is_empty() {
            self.issue_list_state.select(Some(self.selected_issue_index));
        } else {
            self.issue_list_state.select(None);
        }
    }

    pub fn next_issue(&mut self) {
        if !self.filtered_issues.is_empty() {
            self.selected_issue_index = (self.selected_issue_index + 1) % self.filtered_issues.len();
            self.issue_list_state.select(Some(self.selected_issue_index));
        }
    }

    pub fn prev_issue(&mut self) {
        if !self.filtered_issues.is_empty() {
            self.selected_issue_index = if self.selected_issue_index > 0 {
                self.selected_issue_index - 1
            } else {
                self.filtered_issues.len() - 1
            };
            self.issue_list_state.select(Some(self.selected_issue_index));
        }
    }

    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            AppTab::Overview => AppTab::Issues,
            AppTab::Issues => AppTab::Files,
            AppTab::Files => AppTab::Reports,
            AppTab::Reports => AppTab::Help,
            AppTab::Help => AppTab::Overview,
        };
        self.show_issue_detail = false;
    }

    pub fn prev_tab(&mut self) {
        self.current_tab = match self.current_tab {
            AppTab::Overview => AppTab::Help,
            AppTab::Issues => AppTab::Overview,
            AppTab::Files => AppTab::Issues,
            AppTab::Reports => AppTab::Files,
            AppTab::Help => AppTab::Reports,
        };
        self.show_issue_detail = false;
    }

    pub fn get_current_issue(&self) -> Option<&CodeIssue> {
        self.filtered_issues
            .get(self.selected_issue_index)
            .and_then(|&idx| self.all_issues.get(idx))
    }

    pub fn get_files(&self) -> Vec<String> {
        self.review
            .branch_comparison
            .commits_analyzed
            .iter()
            .flat_map(|c| &c.files_changed)
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn clear_filters(&mut self) {
        self.selected_category = None;
        self.selected_severity = None;
        self.apply_filters();
        self.status_message = Some("Filters cleared".to_string());
    }

    pub fn generate_report(&mut self, report_type: u8) {
        match report_type {
            1 => {
                let report = self.generate_review_report();
                if std::fs::write("code_review_report.md", &report).is_ok() {
                    self.status_message = Some("✅ Full report saved to code_review_report.md".to_string());
                } else {
                    self.status_message = Some("❌ Failed to save report".to_string());
                }
            }
            2 => {
                let summary = generate_summary_report(self);
                if std::fs::write("code_review_summary.md", &summary).is_ok() {
                    self.status_message = Some("✅ Summary saved to code_review_summary.md".to_string());
                } else {
                    self.status_message = Some("❌ Failed to save summary".to_string());
                }
            }
            3 => {
                let critical_report = generate_critical_issues_report(self);
                if std::fs::write("critical_issues.md", &critical_report).is_ok() {
                    self.status_message = Some("✅ Critical issues report saved to critical_issues.md".to_string());
                } else {
                    self.status_message = Some("❌ Failed to save critical issues report".to_string());
                }
            }
            _ => {}
        }
    }

    pub fn generate_review_report(&self) -> String {
        if self.all_issues.is_empty() {
            return "No issues found in this review.".to_string();
        }

        let mut report = String::new();
        report.push_str("# Code Review Report\n\n");
        report.push_str(&format!(
            "**Branches Compared:** {} → {}\n",
            self.review.branch_comparison.source_branch,
            self.review.branch_comparison.target_branch
        ));
        report.push_str(&format!(
            "**Generated:** {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Add issues details
        for (i, issue) in self.all_issues.iter().enumerate() {
            let severity_icon = match issue.severity {
                Severity::Critical => "🚨",
                Severity::High => "⚠️",
                Severity::Medium => "🔶",
                Severity::Low => "ℹ️",
                Severity::Info => "💡",
            };

            report.push_str(&format!(
                "## {} Issue {}: {:?}\n\n",
                severity_icon,
                i + 1,
                issue.category
            ));
            report.push_str(&format!("**File:** `{}`\n", issue.file_path));
            if let Some(line) = issue.line_number {
                report.push_str(&format!("**Line:** {line}\n"));
            }
            report.push_str(&format!("**Severity:** {:?}\n\n", issue.severity));
            report.push_str(&format!("**Problem:** {}\n\n", issue.description));
            report.push_str(&format!("**Solution:** {}\n\n", issue.suggestion));

            if let Some(ref snippet) = issue.code_snippet {
                report.push_str("**Code Location:**\n```\n");
                report.push_str(snippet);
                report.push_str("\n```\n\n");
            }
            report.push_str("---\n\n");
        }

        report
    }
}

// Add missing structs and enums
#[derive(Debug, Clone)]
pub struct AppState {
    pub review: Review,
    pub view_mode: ViewMode,
    pub selected_issue_index: usize,
    pub selected_category: Option<IssueCategory>,
    pub selected_severity: Option<Severity>,
    pub all_issues: Vec<CodeIssue>,
    pub filtered_issues: Vec<usize>,
    pub scroll_position: usize,
    pub show_help: bool,
    pub selected_file_index: usize,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Overview,
    IssuesList,
    IssueDetail(usize),
    ReportGeneration,
    Files,
    Help,
}

// Add missing run_app function
async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                        break;
                    }
                    KeyCode::Tab => app.next_tab(),
                    KeyCode::BackTab => app.prev_tab(),
                    KeyCode::Char('h') => app.current_tab = AppTab::Help,
                    KeyCode::Char('o') => app.current_tab = AppTab::Overview,
                    KeyCode::Char('i') => app.current_tab = AppTab::Issues,
                    KeyCode::Char('f') => app.current_tab = AppTab::Files,
                    KeyCode::Char('r') => app.current_tab = AppTab::Reports,
                    KeyCode::Up | KeyCode::Char('k') => {
                        match app.current_tab {
                            AppTab::Issues => app.prev_issue(),
                            _ => {}
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        match app.current_tab {
                            AppTab::Issues => app.next_issue(),
                            _ => {}
                        }
                    }
                    KeyCode::Enter => {
                        if app.current_tab == AppTab::Issues {
                            app.show_issue_detail = !app.show_issue_detail;
                        }
                    }
                    KeyCode::Char('c') => {
                        if app.current_tab == AppTab::Issues {
                            app.clear_filters();
                        }
                    }
                    KeyCode::Char('1') | KeyCode::Char('2') | KeyCode::Char('3') => {
                        if app.current_tab == AppTab::Reports {
                            let report_type = match key.code {
                                KeyCode::Char('1') => 1,
                                KeyCode::Char('2') => 2,
                                KeyCode::Char('3') => 3,
                                _ => 1,
                            };
                            app.generate_report(report_type);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// Add missing UI function
fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    
    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(size);

    // Render header with tabs
    render_header(f, chunks[0], app);
    
    // Render main content based on current tab
    match app.current_tab {
        AppTab::Overview => render_overview(f, chunks[1], app),
        AppTab::Issues => render_issues(f, chunks[1], app),
        AppTab::Files => render_files(f, chunks[1], app),
        AppTab::Reports => render_reports(f, chunks[1], app),
        AppTab::Help => render_help(f, chunks[1], app),
    }
    
    // Render status bar
    render_status_bar(f, chunks[2], app);
}

fn render_header<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let titles = vec!["Overview", "Issues", "Files", "Reports", "Help"];
    let index = match app.current_tab {
        AppTab::Overview => 0,
        AppTab::Issues => 1,
        AppTab::Files => 2,
        AppTab::Reports => 3,
        AppTab::Help => 4,
    };
    
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("AI Code Buddy"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .select(index);
    
    f.render_widget(tabs, area);
}

fn render_overview<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let text = vec![
        Line::from(vec![Span::styled("📊 Code Review Overview", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(format!("🌿 Branches: {} → {}", 
            app.review.branch_comparison.source_branch,
            app.review.branch_comparison.target_branch)),
        Line::from(format!("📁 Files Modified: {}", app.review.metrics.files_modified)),
        Line::from(format!("➕ Lines Added: +{}", app.review.metrics.lines_added)),
        Line::from(format!("➖ Lines Removed: -{}", app.review.metrics.lines_removed)),
        Line::from(format!("🐛 Total Issues: {}", app.all_issues.len())),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Overview"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

fn render_issues<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    if app.show_issue_detail {
        if let Some(issue) = app.get_current_issue() {
            render_issue_detail(f, area, issue);
        }
    } else {
        let items: Vec<ListItem> = app.filtered_issues
            .iter()
            .map(|&idx| {
                if let Some(issue) = app.all_issues.get(idx) {
                    let severity_icon = match issue.severity {
                        Severity::Critical => "🔴",
                        Severity::High => "🟠",
                        Severity::Medium => "🟡",
                        Severity::Low => "🟢",
                        Severity::Info => "🔵",
                    };
                    let content = format!("{} [{:?}] {} {}", 
                        severity_icon, 
                        issue.category, 
                        issue.file_path,
                        if let Some(line) = issue.line_number {
                            format!(":{}", line)
                        } else {
                            String::new()
                        }
                    );
                    ListItem::new(content)
                } else {
                    ListItem::new("Invalid issue")
                }
            })
            .collect();

        let issues_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Issues"))
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("► ");

        f.render_stateful_widget(issues_list, area, &mut app.issue_list_state.clone());
    }
}

fn render_issue_detail<B: Backend>(f: &mut Frame<B>, area: Rect, issue: &CodeIssue) {
    let text = vec![
        Line::from(vec![Span::styled("🔍 Issue Details", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(format!("📁 File: {}", issue.file_path)),
        Line::from(format!("📂 Category: {:?}", issue.category)),
        Line::from(format!("⚠️ Severity: {:?}", issue.severity)),
        Line::from(""),
        Line::from(vec![Span::styled("📝 Description:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(format!("   {}", issue.description)),
        Line::from(""),
        Line::from(vec![Span::styled("💡 Suggestion:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from(format!("   {}", issue.suggestion)),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Issue Detail"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

fn render_files<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let files = app.get_files();
    let items: Vec<ListItem> = files
        .iter()
        .map(|file| {
            let issue_count = app.all_issues.iter().filter(|i| i.file_path == *file).count();
            let icon = if issue_count > 0 {
                match issue_count {
                    1..=2 => "🟡",
                    3..=5 => "🟠",
                    _ => "🔴",
                }
            } else {
                "✅"
            };
            
            let content = if issue_count > 0 {
                format!("{} {} ({} issues)", icon, file, issue_count)
            } else {
                format!("{} {}", icon, file)
            };
            ListItem::new(content)
        })
        .collect();

    let files_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Changed Files"))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("► ");

    f.render_stateful_widget(files_list, area, &mut app.file_list_state.clone());
}

fn render_reports<B: Backend>(f: &mut Frame<B>, area: Rect, _app: &App) {
    let text = vec![
        Line::from(vec![Span::styled("📄 Generate Report", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from("Choose a report format to generate:"),
        Line::from(""),
        Line::from(vec![Span::styled("1️⃣  Full Markdown Report", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("     • Complete analysis with all issues"),
        Line::from("     • AI assessment and recommendations"),
        Line::from(""),
        Line::from(vec![Span::styled("2️⃣  Executive Summary", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("     • High-level overview"),
        Line::from("     • Key metrics and priority issues"),
        Line::from(""),
        Line::from(vec![Span::styled("3️⃣  Critical Issues Only", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("     • Focus on critical and high-severity issues"),
        Line::from("     • Immediate action items"),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Reports"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

fn render_help<B: Backend>(f: &mut Frame<B>, area: Rect, _app: &App) {
    let text = vec![
        Line::from(vec![Span::styled("🆘 Help - AI Code Review Tool", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))]),
        Line::from(""),
        Line::from(vec![Span::styled("📋 Navigation Commands:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("   Tab/Shift+Tab - Switch between tabs"),
        Line::from("   o - Show overview screen"),
        Line::from("   i - Show issues list"),
        Line::from("   f - Show changed files"),
        Line::from("   r - Generate reports"),
        Line::from("   h - Show this help"),
        Line::from("   q - Exit the application"),
        Line::from(""),
        Line::from(vec![Span::styled("🐛 Issues Commands:", Style::default().add_modifier(Modifier::BOLD))]),
        Line::from("   ↑/k - Navigate to previous issue"),
        Line::from("   ↓/j - Navigate to next issue"),
        Line::from("   Enter - View issue details"),
        Line::from("   c - Clear filters"),
    ];
    
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

fn render_status_bar<B: Backend>(f: &mut Frame<B>, area: Rect, app: &App) {
    let status_text = if let Some(ref message) = app.status_message {
        message.clone()
    } else {
        match app.current_tab {
            AppTab::Overview => "[Tab] switch tabs | [q] quit".to_string(),
            AppTab::Issues => "[↑↓/jk] navigate | [Enter] details | [c] clear filters | [q] quit".to_string(),
            AppTab::Files => "[↑↓] navigate | [q] quit".to_string(),
            AppTab::Reports => "[1/2/3] generate report | [q] quit".to_string(),
            AppTab::Help => "[Tab] switch tabs | [q] quit".to_string(),
        }
    };
    
    let status = Paragraph::new(status_text)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));
    
    f.render_widget(status, area);
}

impl AppState {
    pub fn new(review: Review) -> Self {
        let all_issues = Self::collect_all_issues(&review);
        let filtered_issues = (0..all_issues.len()).collect();

        Self {
            review,
            view_mode: ViewMode::Overview,
            selected_issue_index: 0,
            selected_category: None,
            selected_severity: None,
            all_issues,
            filtered_issues,
            scroll_position: 0,
            show_help: false,
            selected_file_index: 0,
            status_message: None,
        }
    }

    fn collect_all_issues(review: &Review) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for category_issues in review.issues.values() {
            for issue in category_issues {
                issues.push(issue.clone());
            }
        }
        // Sort by severity (Critical first)
        issues.sort_by(|a, b| {
            let severity_order = |s: &Severity| match s {
                Severity::Critical => 0,
                Severity::High => 1,
                Severity::Medium => 2,
                Severity::Low => 3,
                Severity::Info => 4,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });
        issues
    }

    pub fn apply_filters(&mut self) {
        self.filtered_issues = self
            .all_issues
            .iter()
            .enumerate()
            .filter(|(_, issue)| {
                let category_match = self
                    .selected_category
                    .as_ref()
                    .is_none_or(|cat| &issue.category == cat);
                let severity_match = self
                    .selected_severity
                    .as_ref()
                    .is_none_or(|sev| &issue.severity == sev);
                category_match && severity_match
            })
            .map(|(idx, _)| idx)
            .collect();

        if self.selected_issue_index >= self.filtered_issues.len() {
            self.selected_issue_index = 0;
        }
    }

    #[allow(dead_code)]
    pub fn get_current_issue(&self) -> Option<&CodeIssue> {
        self.filtered_issues
            .get(self.selected_issue_index)
            .and_then(|&idx| self.all_issues.get(idx))
    }

    pub fn next_issue(&mut self) {
        if !self.filtered_issues.is_empty() {
            self.selected_issue_index =
                (self.selected_issue_index + 1) % self.filtered_issues.len();
        }
    }

    pub fn prev_issue(&mut self) {
        if !self.filtered_issues.is_empty() {
            self.selected_issue_index = if self.selected_issue_index > 0 {
                self.selected_issue_index - 1
            } else {
                self.filtered_issues.len() - 1
            };
        }
    }

    pub fn generate_review_report(&self) -> String {
        let mut report = String::new();

        report.push_str("# Code Review Report\n\n");
        report.push_str(&format!(
            "**Branches Compared:** {} → {}\n",
            self.review.branch_comparison.source_branch,
            self.review.branch_comparison.target_branch
        ));
        report.push_str(&format!(
            "**Generated:** {}\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Executive Summary
        report.push_str("## Executive Summary\n\n");
        let critical_count = self
            .all_issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Critical))
            .count();
        let high_count = self
            .all_issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::High))
            .count();
        let total_issues = self.all_issues.len();

        if critical_count > 0 {
            report.push_str(&format!(
                "⚠️ **CRITICAL:** {critical_count} critical issues require immediate attention before merge.\n"
            ));
        }
        if high_count > 0 {
            report.push_str(&format!(
                "🔶 **HIGH:** {high_count} high-priority issues should be addressed.\n"
            ));
        }
        report.push_str(&format!(
            "📊 **Total Issues:** {} findings across {} files.\n\n",
            total_issues, self.review.metrics.files_modified
        ));

        // Change Metrics
        report.push_str("## Change Metrics\n\n");
        report.push_str(&format!(
            "- **Files Modified:** {}\n",
            self.review.metrics.files_modified
        ));
        report.push_str(&format!(
            "- **Lines Added:** {}\n",
            self.review.metrics.lines_added
        ));
        report.push_str(&format!(
            "- **Lines Removed:** {}\n",
            self.review.metrics.lines_removed
        ));
        report.push_str(&format!(
            "- **Commits Analyzed:** {}\n\n",
            self.review.branch_comparison.commits_analyzed.len()
        ));

        // Priority Issues
        if critical_count > 0 || high_count > 0 {
            report.push_str("## Priority Issues\n\n");
            for issue in &self.all_issues {
                if matches!(issue.severity, Severity::Critical | Severity::High) {
                    let severity_icon = match issue.severity {
                        Severity::Critical => "🚨",
                        Severity::High => "⚠️",
                        _ => "",
                    };
                    report.push_str(&format!(
                        "{} **{:?}** in `{}`\n",
                        severity_icon, issue.category, issue.file_path
                    ));
                    if let Some(line) = issue.line_number {
                        report.push_str(&format!("   Line {}: {}\n", line, issue.description));
                    } else {
                        report.push_str(&format!("   {}\n", issue.description));
                    }
                    report.push_str(&format!("   *Recommendation:* {}\n\n", issue.suggestion));
                }
            }
        }

        // Issues by Category
        report.push_str("## Issues by Category\n\n");
        let mut category_issues: std::collections::HashMap<&IssueCategory, Vec<&CodeIssue>> =
            std::collections::HashMap::new();
        for issue in &self.all_issues {
            category_issues
                .entry(&issue.category)
                .or_default()
                .push(issue);
        }

        for (category, issues) in category_issues.iter() {
            report.push_str(&format!("### {:?} ({} issues)\n\n", category, issues.len()));
            for issue in issues {
                let severity_badge = match issue.severity {
                    Severity::Critical => "🚨 Critical",
                    Severity::High => "⚠️ High",
                    Severity::Medium => "🔶 Medium",
                    Severity::Low => "ℹ️ Low",
                    Severity::Info => "💡 Info",
                };
                report.push_str(&format!("- **{}** `{}` ", severity_badge, issue.file_path));
                if let Some(line) = issue.line_number {
                    report.push_str(&format!("(Line {line})"));
                }
                report.push_str(&format!(
                    "\n  {}\n  *Fix:* {}\n\n",
                    issue.description, issue.suggestion
                ));
            }
        }

        // AI Assessment
        if !self.review.overall_assessment.is_empty() {
            report.push_str("## AI Analysis\n\n");
            report.push_str(&self.review.overall_assessment);
            report.push_str("\n\n");
        }

        // Recommendations
        if !self.review.priority_recommendations.is_empty() {
            report.push_str("## Recommendations\n\n");
            for rec in &self.review.priority_recommendations {
                report.push_str(&format!("- {rec}\n"));
            }
            report.push('\n');
        }

        // Technology Stack
        if !self
            .review
            .technology_stack
            .programming_languages
            .is_empty()
        {
            report.push_str("## Technology Stack\n\n");
            report.push_str(&format!(
                "**Languages:** {}\n",
                self.review
                    .technology_stack
                    .programming_languages
                    .join(", ")
            ));
            if !self.review.technology_stack.tools.is_empty() {
                report.push_str(&format!(
                    "**Tools:** {}\n",
                    self.review.technology_stack.tools.join(", ")
                ));
            }
        }

        report
    }
}

// Ratatui-based TUI implementation
pub async fn run_tui(review: Review) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new(review);
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

#[derive(Debug)]
struct CodeReviewApp {
    state: AppState,
}

impl CodeReviewApp {
    fn new(review: Review) -> Self {
        Self {
            state: AppState::new(review),
        }
    }

    async fn run(&mut self) -> anyhow::Result<()> {
        let mut system = iocraft::SystemContext::new()?;
        
        loop {
            let props = AppProps {
                state: self.state.clone(),
            };
            
            let element = element! {
                App { ..props }
            };
            
            let output = system.render(element).await?;
            
            // Handle input
            if let Some(input) = system.next_input().await? {
                match input {
                    IoInput::Key(key) => {
                        if self.handle_key_input(key) {
                            break; // Exit application
                        }
                    }
                    _ => {}
                }
            }
        }
        
        Ok(())
    }
    
    fn handle_key_input(&mut self, key: IoKey) -> bool {
        match key {
            IoKey::Char('q') => return true, // Quit
            IoKey::Char('h') => self.state.view_mode = ViewMode::Help,
            IoKey::Char('o') => self.state.view_mode = ViewMode::Overview,
            IoKey::Char('i') => self.state.view_mode = ViewMode::IssuesList,
            IoKey::Char('r') => self.state.view_mode = ViewMode::ReportGeneration,
            IoKey::Char('f') => self.state.view_mode = ViewMode::Files,
            IoKey::Up | IoKey::Char('k') => self.move_up(),
            IoKey::Down | IoKey::Char('j') => self.move_down(),
            IoKey::Enter => self.handle_enter(),
            IoKey::Char('b') => self.handle_back(),
            IoKey::Char('c') => self.clear_filters(),
            IoKey::Char('1') | IoKey::Char('2') | IoKey::Char('3') => {
                self.handle_report_generation(key);
            }
            _ => {}
        }
        false
    }
    
    fn move_up(&mut self) {
        match self.state.view_mode {
            ViewMode::IssuesList => {
                if self.state.selected_issue_index > 0 {
                    self.state.selected_issue_index -= 1;
                }
            }
            ViewMode::Files => {
                if self.state.selected_file_index > 0 {
                    self.state.selected_file_index -= 1;
                }
            }
            _ => {}
        }
    }
    
    fn move_down(&mut self) {
        match self.state.view_mode {
            ViewMode::IssuesList => {
                if self.state.selected_issue_index + 1 < self.state.filtered_issues.len() {
                    self.state.selected_issue_index += 1;
                }
            }
            ViewMode::Files => {
                let file_count = self.get_files().len();
                if self.state.selected_file_index + 1 < file_count {
                    self.state.selected_file_index += 1;
                }
            }
            _ => {}
        }
    }
    
    fn handle_enter(&mut self) {
        match self.state.view_mode {
            ViewMode::IssuesList => {
                if let Some(&issue_idx) = self.state.filtered_issues.get(self.state.selected_issue_index) {
                    self.state.view_mode = ViewMode::IssueDetail(issue_idx);
                }
            }
            _ => {}
        }
    }
    
    fn handle_back(&mut self) {
        match self.state.view_mode {
            ViewMode::IssueDetail(_) => self.state.view_mode = ViewMode::IssuesList,
            ViewMode::Help => self.state.view_mode = ViewMode::Overview,
            _ => {}
        }
    }
    
    fn clear_filters(&mut self) {
        self.state.selected_category = None;
        self.state.selected_severity = None;
        self.state.apply_filters();
    }
    
    fn handle_report_generation(&mut self, key: IoKey) {
        if matches!(self.state.view_mode, ViewMode::ReportGeneration) {
            match key {
                IoKey::Char('1') => {
                    let report = self.state.generate_review_report();
                    let _ = std::fs::write("code_review_report.md", &report);
                    self.state.status_message = Some("✅ Full report saved to code_review_report.md".to_string());
                }
                IoKey::Char('2') => {
                    let summary = generate_summary_report(&self.state);
                    let _ = std::fs::write("code_review_summary.md", &summary);
                    self.state.status_message = Some("✅ Summary saved to code_review_summary.md".to_string());
                }
                IoKey::Char('3') => {
                    let critical_report = generate_critical_issues_report(&self.state);
                    let _ = std::fs::write("critical_issues.md", &critical_report);
                    self.state.status_message = Some("✅ Critical issues report saved to critical_issues.md".to_string());
                }
                _ => {}
            }
        }
    }
    
    fn get_files(&self) -> Vec<String> {
        self.state
            .review
            .branch_comparison
            .commits_analyzed
            .iter()
            .flat_map(|c| &c.files_changed)
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect()
    }
}

#[derive(Props)]
struct AppProps {
    state: AppState,
}

#[component]
fn app(props: &AppProps) -> impl Into<AnyElement> {
    let AppProps { state } = props;
    
    element! {
        Box(
            flex_direction: FlexDirection::Column,
            width: 100.percent(),
            height: 100.percent(),
        ) {
            Header { state: state.clone() }
            
            Box(flex: 1) {
                match &state.view_mode {
                    ViewMode::Overview => OverviewView { state: state.clone() },
                    ViewMode::IssuesList => IssuesListView { state: state.clone() },
                    ViewMode::IssueDetail(idx) => IssueDetailView { 
                        state: state.clone(),
                        issue_idx: *idx 
                    },
                    ViewMode::ReportGeneration => ReportView { state: state.clone() },
                    ViewMode::Files => FilesView { state: state.clone() },
                    ViewMode::Help => HelpView { state: state.clone() },
                }
            }
            
            StatusBar { state: state.clone() }
        }
    }
}

#[component]
fn header(props: &HeaderProps) -> impl Into<AnyElement> {
    let view_title = match props.state.view_mode {
        ViewMode::Overview => "📊 Overview",
        ViewMode::IssuesList => "🐛 Issues",
        ViewMode::IssueDetail(_) => "🔍 Issue Detail",
        ViewMode::ReportGeneration => "📄 Reports",
        ViewMode::Files => "📁 Files",
        ViewMode::Help => "🆘 Help",
    };
    
    element! {
        Box(
            flex_direction: FlexDirection::Row,
            padding: Padding::all(1),
            background: Color::Blue,
        ) {
            Text(content: "🤠 AI Code Buddy", style: Style::new().bold().color(Color::Yellow))
            Text(content: " | ")
            Text(content: view_title, style: Style::new().bold())
            Box(flex: 1) {}
            Text(content: format!("Issues: {}", props.state.all_issues.len()))
        }
    }
}

#[derive(Props)]
struct HeaderProps {
    state: AppState,
}

#[component]
fn overview_view(props: &OverviewProps) -> impl Into<AnyElement> {
    let state = &props.state;
    
    let critical_count = state.all_issues.iter().filter(|i| matches!(i.severity, Severity::Critical)).count();
    let high_count = state.all_issues.iter().filter(|i| matches!(i.severity, Severity::High)).count();
    let medium_count = state.all_issues.iter().filter(|i| matches!(i.severity, Severity::Medium)).count();
    let low_count = state.all_issues.iter().filter(|i| matches!(i.severity, Severity::Low)).count();
    
    element! {
        Box(
            flex_direction: FlexDirection::Column,
            padding: Padding::all(2),
        ) {
            Text(content: "📊 Code Review Overview", style: Style::new().bold().color(Color::Cyan))
            Text(content: "".to_string())
            
            // Branch info
            Text(content: format!("🌿 Branches: {} → {}", 
                state.review.branch_comparison.source_branch,
                state.review.branch_comparison.target_branch
            ))
            
            // Metrics
            Text(content: format!("📁 Files Modified: {}", state.review.metrics.files_modified))
            Text(content: format!("➕ Lines Added: +{}", state.review.metrics.lines_added))
            Text(content: format!("➖ Lines Removed: -{}", state.review.metrics.lines_removed))
            Text(content: format!("📝 Commits: {}", state.review.branch_comparison.commits_analyzed.len()))
            Text(content: "".to_string())
            
            // Issues by severity
            Text(content: "🚨 Issues by Severity:", style: Style::new().bold())
            if critical_count > 0 {
                Text(content: format!("   🔴 Critical: {}", critical_count), style: Style::new().color(Color::Red))
            }
            if high_count > 0 {
                Text(content: format!("   🟠 High: {}", high_count), style: Style::new().color(Color::Yellow))
            }
            if medium_count > 0 {
                Text(content: format!("   🟡 Medium: {}", medium_count), style: Style::new().color(Color::Yellow))
            }
            if low_count > 0 {
                Text(content: format!("   🟢 Low: {}", low_count), style: Style::new().color(Color::Green))
            }
            
            Text(content: "".to_string())
            
            // Priority recommendations
            if !state.review.priority_recommendations.is_empty() {
                Text(content: "⚡ Priority Recommendations:", style: Style::new().bold())
                for rec in &state.review.priority_recommendations {
                    Text(content: format!("   • {}", rec))
                }
            }
        }
    }
}

#[derive(Props)]
struct OverviewProps {
    state: AppState,
}

#[component]
fn issues_list_view(props: &IssuesListProps) -> impl Into<AnyElement> {
    let state = &props.state;
    
    element! {
        Box(
            flex_direction: FlexDirection::Column,
            padding: Padding::all(2),
        ) {
            Text(content: format!("🐛 Issues List ({}/{})", 
                state.filtered_issues.len(), 
                state.all_issues.len()
            ), style: Style::new().bold().color(Color::Cyan))
            Text(content: "".to_string())
            
            if state.filtered_issues.is_empty() {
                Text(content: "✅ No issues found with current filters.", style: Style::new().color(Color::Green))
            } else {
                // Show issues around the selected one
                let start = state.selected_issue_index.saturating_sub(5);
                let end = (start + 10).min(state.filtered_issues.len());
                
                for i in start..end {
                    if let Some(&issue_idx) = state.filtered_issues.get(i) {
                        if let Some(issue) = state.all_issues.get(issue_idx) {
                            let is_selected = i == state.selected_issue_index;
                            let marker = if is_selected { "► " } else { "  " };
                            
                            let (severity_icon, color) = match issue.severity {
                                Severity::Critical => ("🔴", Color::Red),
                                Severity::High => ("🟠", Color::Yellow),
                                Severity::Medium => ("🟡", Color::Yellow),
                                Severity::Low => ("🟢", Color::Green),
                                Severity::Info => ("🔵", Color::Blue),
                            };
                            
                            let mut line_info = String::new();
                            if let Some(line) = issue.line_number {
                                line_info = format!(":{}", line);
                            }
                            
                            let content = format!("{}{} [{:?}] {}{}", 
                                marker, severity_icon, issue.category, issue.file_path, line_info
                            );
                            
                            let style = if is_selected {
                                Style::new().background(Color::DarkGray).color(color)
                            } else {
                                Style::new().color(color)
                            };
                            
                            Text(content: content, style: style)
                            
                            if is_selected {
                                Text(content: format!("     💡 {}", issue.description), 
                                     style: Style::new().color(Color::Gray))
                            }
                        }
                    }
                }
                
                if state.filtered_issues.len() > 10 {
                    Text(content: format!("... showing {} of {} issues", 
                        end - start, state.filtered_issues.len()))
                }
            }
        }
    }
}

#[derive(Props)]
struct IssuesListProps {
    state: AppState,
}

#[component]
fn issue_detail_view(props: &IssueDetailProps) -> impl Into<AnyElement> {
    let state = &props.state;
    
    if let Some(issue) = state.all_issues.get(props.issue_idx) {
        let (severity_icon, color) = match issue.severity {
            Severity::Critical => ("🔴", Color::Red),
            Severity::High => ("🟠", Color::Yellow),
            Severity::Medium => ("🟡", Color::Yellow),
            Severity::Low => ("🟢", Color::Green),
            Severity::Info => ("🔵", Color::Blue),
        };
        
        element! {
            Box(
                flex_direction: FlexDirection::Column,
                padding: Padding::all(2),
            ) {
                Text(content: "🔍 Issue Details", style: Style::new().bold().color(Color::Cyan))
                Text(content: "".to_string())
                
                Text(content: format!("📁 File: {}", issue.file_path))
                if let Some(line) = issue.line_number {
                    Text(content: format!("📍 Line: {}", line))
                }
                Text(content: format!("📂 Category: {:?}", issue.category))
                Text(content: format!("{} Severity: {:?}", severity_icon, issue.severity), style: Style::new().color(color))
                Text(content: "".to_string())
                
                Text(content: "📝 Description:", style: Style::new().bold())
                Text(content: format!("   {}", issue.description))
                Text(content: "".to_string())
                
                Text(content: "💡 Suggestion:", style: Style::new().bold())
                Text(content: format!("   {}", issue.suggestion))
                
                if let Some(ref snippet) = issue.code_snippet {
                    Text(content: "".to_string())
                    Text(content: "📄 Code Snippet:", style: Style::new().bold())
                    
                    Box(
                        border: Border::single(),
                        padding: Padding::all(1),
                        margin: Margin::top(1),
                    ) {
                        for line in snippet.lines() {
                            Text(content: line.to_string(), style: Style::new().color(Color::Gray))
                        }
                    }
                }
            }
        }
    } else {
        element! {
            Box(padding: Padding::all(2)) {
                Text(content: "❌ Issue not found", style: Style::new().color(Color::Red))
            }
        }
    }
}

#[derive(Props)]
struct IssueDetailProps {
    state: AppState,
    issue_idx: usize,
}

#[component]
fn report_view(props: &ReportProps) -> impl Into<AnyElement> {
    element! {
        Box(
            flex_direction: FlexDirection::Column,
            padding: Padding::all(2),
        ) {
            Text(content: "📄 Generate Report", style: Style::new().bold().color(Color::Cyan))
            Text(content: "".to_string())
            Text(content: "Choose a report format to generate:")
            Text(content: "".to_string())
            
            Text(content: "1️⃣  Full Markdown Report", style: Style::new().bold())
            Text(content: "     • Complete analysis with all issues")
            Text(content: "     • AI assessment and recommendations")
            Text(content: "     • Technology stack information")
            Text(content: "".to_string())
            
            Text(content: "2️⃣  Executive Summary", style: Style::new().bold())
            Text(content: "     • High-level overview")
            Text(content: "     • Key metrics and priority issues")
            Text(content: "     • Quick decision-making format")
            Text(content: "".to_string())
            
            Text(content: "3️⃣  Critical Issues Only", style: Style::new().bold())
            Text(content: "     • Focus on critical and high-severity issues")
            Text(content: "     • Immediate action items")
            Text(content: "     • Risk assessment")
        }
    }
}

#[derive(Props)]
struct ReportProps {
    state: AppState,
}

#[component]
fn files_view(props: &FilesProps) -> impl Into<AnyElement> {
    let state = &props.state;
    let files: Vec<String> = state
        .review
        .branch_comparison
        .commits_analyzed
        .iter()
        .flat_map(|c| &c.files_changed)
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    element! {
        Box(
            flex_direction: FlexDirection::Column,
            padding: Padding::all(2),
        ) {
            Text(content: "📁 Changed Files", style: Style::new().bold().color(Color::Cyan))
            Text(content: "".to_string())
            
            if files.is_empty() {
                Text(content: "📭 No files found.")
            } else {
                for (idx, file) in files.iter().enumerate() {
                    let issue_count = state.all_issues.iter().filter(|i| i.file_path == *file).count();
                    
                    let (icon, color) = if issue_count > 0 {
                        match issue_count {
                            1..=2 => ("🟡", Color::Yellow),
                            3..=5 => ("🟠", Color::Yellow),
                            _ => ("🔴", Color::Red),
                        }
                    } else {
                        ("✅", Color::Green)
                    };
                    
                    let is_selected = idx == state.selected_file_index;
                    let marker = if is_selected { "► " } else { "  " };
                    
                    let content = if issue_count > 0 {
                        format!("{}{} {} ({} issues)", marker, icon, file, issue_count)
                    } else {
                        format!("{}{} {}", marker, icon, file)
                    };
                    
                    let style = if is_selected {
                        Style::new().background(Color::DarkGray).color(color)
                    } else {
                        Style::new().color(color)
                    };
                    
                    Text(content: content, style: style)
                }
                
                Text(content: "".to_string())
                Text(content: "📊 Legend: ✅ No issues  🟡 Few issues  🟠 Some issues  🔴 Many issues")
            }
        }
    }
}

#[derive(Props)]
struct FilesProps {
    state: AppState,
}

#[component]
fn help_view(props: &HelpProps) -> impl Into<AnyElement> {
    element! {
        Box(
            flex_direction: FlexDirection::Column,
            padding: Padding::all(2),
        ) {
            Text(content: "🆘 Help - AI Code Review Tool", style: Style::new().bold().color(Color::Cyan))
            Text(content: "".to_string())
            
            Text(content: "📋 Navigation Commands:", style: Style::new().bold())
            Text(content: "   o - Show overview screen")
            Text(content: "   i - Show issues list")
            Text(content: "   r - Generate reports")
            Text(content: "   f - Show changed files")
            Text(content: "   q - Exit the application")
            Text(content: "   h - Show this help")
            Text(content: "".to_string())
            
            Text(content: "🐛 Issues List Commands:", style: Style::new().bold())
            Text(content: "   ↑/k - Navigate to previous issue")
            Text(content: "   ↓/j - Navigate to next issue")
            Text(content: "   Enter - View issue details")
            Text(content: "   c - Clear filters")
            Text(content: "".to_string())
            
            Text(content: "🔍 Issue Detail Commands:", style: Style::new().bold())
            Text(content: "   b - Return to issues list")
            Text(content: "   ↑/↓ - Navigate between issues")
            Text(content: "".to_string())
            
            Text(content: "📄 Report Generation:", style: Style::new().bold())
            Text(content: "   1 - Generate full markdown report")
            Text(content: "   2 - Generate executive summary")
            Text(content: "   3 - Generate critical issues report")
            Text(content: "".to_string())
            
            Text(content: "💡 Tips:", style: Style::new().bold())
            Text(content: "   • Reports are saved as markdown files in current directory")
            Text(content: "   • Use filters in issues list to focus on specific categories")
            Text(content: "   • Critical and high-severity issues should be addressed first")
        }
    }
}

#[derive(Props)]
struct HelpProps {
    state: AppState,
}

#[component]
fn status_bar(props: &StatusBarProps) -> impl Into<AnyElement> {
    let shortcuts = match props.state.view_mode {
        ViewMode::Overview => "[i]ssues [r]eport [f]iles [q]uit [h]elp",
        ViewMode::IssuesList => "[↑↓/jk] navigate [Enter] details [c]lear [o]verview",
        ViewMode::IssueDetail(_) => "[b]ack [↑↓] navigate [o]verview",
        ViewMode::ReportGeneration => "[1/2/3] generate [o]verview [q]uit",
        ViewMode::Files => "[↑↓] navigate [o]verview [i]ssues",
        ViewMode::Help => "[b]ack [o]verview",
    };
    
    element! {
        Box(
            flex_direction: FlexDirection::Row,
            padding: Padding::all(1),
            background: Color::DarkGray,
        ) {
            if let Some(ref message) = props.state.status_message {
                Text(content: message.clone(), style: Style::new().color(Color::Green))
                Box(flex: 1) {}
            } else {
                Text(content: shortcuts, style: Style::new().color(Color::White))
                Box(flex: 1) {}
            }
            Text(content: format!("AI Code Buddy v0.1.2"), style: Style::new().color(Color::Gray))
        }
    }
}

#[derive(Props)]
struct StatusBarProps {
    state: AppState,
}

fn print_overview(state: &AppState) {
    println!("📊 Code Review Overview");
    println!("{}", "=".repeat(50));
    println!(
        "🌿 Branches: {} → {}",
        state.review.branch_comparison.source_branch, state.review.branch_comparison.target_branch
    );
    println!("📁 Files Modified: {}", state.review.metrics.files_modified);
    println!("➕ Lines Added: +{}", state.review.metrics.lines_added);
    println!("➖ Lines Removed: -{}", state.review.metrics.lines_removed);
    println!("🐛 Total Issues: {}", state.all_issues.len());
    println!(
        "📝 Commits: {}",
        state.review.branch_comparison.commits_analyzed.len()
    );

    let critical_count = state
        .all_issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Critical))
        .count();
    let high_count = state
        .all_issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::High))
        .count();
    let medium_count = state
        .all_issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Medium))
        .count();
    let low_count = state
        .all_issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Low))
        .count();

    println!("\n🚨 Issues by Severity:");
    if critical_count > 0 {
        println!("   🔴 Critical: {critical_count}");
    }
    if high_count > 0 {
        println!("   🟠 High: {high_count}");
    }
    if medium_count > 0 {
        println!("   🟡 Medium: {medium_count}");
    }
    if low_count > 0 {
        println!("   🟢 Low: {low_count}");
    }

    if !state.review.priority_recommendations.is_empty() {
        println!("\n⚡ Priority Recommendations:");
        for rec in &state.review.priority_recommendations {
            println!("   • {rec}");
        }
    }

    println!("\n📋 Commands: [i]ssues, [r]eport, [f]iles, [q]uit, [h]elp");
}

fn print_issues_list(state: &AppState) {
    println!(
        "🐛 Issues List ({}/{})",
        state.filtered_issues.len(),
        state.all_issues.len()
    );
    println!("{}", "=".repeat(50));

    if state.filtered_issues.is_empty() {
        println!("✅ No issues found with current filters.");
        println!("\n📋 Commands: [o]verview, [c]lear filters, [q]uit");
        return;
    }

    // Show up to 10 issues around the selected one
    let start = state.selected_issue_index.saturating_sub(5);
    let end = (start + 10).min(state.filtered_issues.len());

    for i in start..end {
        if let Some(&issue_idx) = state.filtered_issues.get(i) {
            if let Some(issue) = state.all_issues.get(issue_idx) {
                let marker = if i == state.selected_issue_index {
                    "► "
                } else {
                    "  "
                };
                let severity_icon = match issue.severity {
                    Severity::Critical => "🔴",
                    Severity::High => "🟠",
                    Severity::Medium => "🟡",
                    Severity::Low => "🟢",
                    Severity::Info => "🔵",
                };

                println!(
                    "{}{} [{:?}] {} {}",
                    marker,
                    severity_icon,
                    issue.category,
                    issue.file_path,
                    if let Some(line) = issue.line_number {
                        format!(":{line}")
                    } else {
                        String::new()
                    }
                );

                if i == state.selected_issue_index {
                    println!("     💡 {}", issue.description);
                }
            }
        }
    }

    if state.filtered_issues.len() > 10 {
        println!(
            "\n... showing {} of {} issues",
            end - start,
            state.filtered_issues.len()
        );
    }

    println!("\n📋 Commands: [j/k] navigate, [enter] details, [o]verview, [c]lear filters");
}

fn print_issue_detail(state: &AppState, issue_idx: usize) {
    if let Some(issue) = state.all_issues.get(issue_idx) {
        println!("🔍 Issue Details");
        println!("{}", "=".repeat(50));

        let severity_icon = match issue.severity {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🟢",
            Severity::Info => "🔵",
        };

        println!("📁 File: {}", issue.file_path);
        if let Some(line) = issue.line_number {
            println!("📍 Line: {line}");
        }
        println!("📂 Category: {:?}", issue.category);
        println!("{} Severity: {:?}", severity_icon, issue.severity);

        println!("\n📝 Description:");
        println!("   {}", issue.description);

        println!("\n💡 Suggestion:");
        println!("   {}", issue.suggestion);

        if let Some(ref snippet) = issue.code_snippet {
            println!("\n📄 Code Snippet:");
            println!("┌{}", "─".repeat(48));
            for line in snippet.lines() {
                println!("│ {line}");
            }
            println!("└{}", "─".repeat(48));
        }

        println!("\n📋 Commands: [b]ack, [j/k] navigate issues, [o]verview");
    } else {
        println!("❌ Issue not found");
    }
}

fn print_report_generation(_state: &AppState) {
    println!("📄 Generate Report");
    println!("{}", "=".repeat(50));
    println!("Choose a report format to generate:");
    println!();
    println!("1️⃣  Full Markdown Report");
    println!("     • Complete analysis with all issues");
    println!("     • AI assessment and recommendations");
    println!("     • Technology stack information");
    println!();
    println!("2️⃣  Executive Summary");
    println!("     • High-level overview");
    println!("     • Key metrics and priority issues");
    println!("     • Quick decision-making format");
    println!();
    println!("3️⃣  Critical Issues Only");
    println!("     • Focus on critical and high-severity issues");
    println!("     • Immediate action items");
    println!("     • Risk assessment");
    println!();
    println!("📋 Commands: [1/2/3] generate report, [o]verview, [q]uit");
}

fn print_files_view(state: &AppState) {
    println!("📁 Changed Files");
    println!("{}", "=".repeat(50));

    let files: Vec<String> = state
        .review
        .branch_comparison
        .commits_analyzed
        .iter()
        .flat_map(|c| &c.files_changed)
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    if files.is_empty() {
        println!("📭 No files found.");
        return;
    }

    for file in &files {
        let issue_count = state
            .all_issues
            .iter()
            .filter(|i| i.file_path == *file)
            .count();

        let icon = if issue_count > 0 {
            match issue_count {
                1..=2 => "🟡",
                3..=5 => "🟠",
                _ => "🔴",
            }
        } else {
            "✅"
        };

        if issue_count > 0 {
            println!("{icon} {file} ({issue_count} issues)");
        } else {
            println!("{icon} {file}");
        }
    }

    println!("\n📊 Legend: ✅ No issues  🟡 Few issues  🟠 Some issues  🔴 Many issues");
    println!("\n📋 Commands: [o]verview, [i]ssues, [q]uit");
}

fn print_help() {
    println!("🆘 Help - AI Code Review Tool");
    println!("{}", "=".repeat(50));
    println!("📋 Navigation Commands:");
    println!("   o, overview    - Show overview screen");
    println!("   i, issues      - Show issues list");
    println!("   r, report      - Generate reports");
    println!("   f, files       - Show changed files");
    println!("   q, quit        - Exit the application");
    println!("   h, help        - Show this help");
    println!();
    println!("🐛 Issues List Commands:");
    println!("   j, down, n, next  - Navigate to next issue");
    println!("   k, up, p, prev    - Navigate to previous issue");
    println!("   enter, details    - View issue details");
    println!("   c, clear          - Clear filters");
    println!();
    println!("🔍 Issue Detail Commands:");
    println!("   b, back          - Return to issues list");
    println!("   j, k             - Navigate between issues");
    println!();
    println!("📄 Report Generation:");
    println!("   1                - Generate full markdown report");
    println!("   2                - Generate executive summary");
    println!("   3                - Generate critical issues report");
    println!();
    println!("💡 Tips:");
    println!("   • Reports are saved as markdown files in current directory");
    println!("   • Use filters in issues list to focus on specific categories");
    println!("   • Critical and high-severity issues should be addressed first");
}

fn generate_summary_report(state: &AppState) -> String {
    format!(
        "# Code Review Executive Summary\n\n\
        **Review Date:** {}\n\
        **Branches:** {} → {}\n\
        **Files Changed:** {}\n\
        **Total Issues:** {}\n\
        **Critical Issues:** {}\n\
        **High Priority Issues:** {}\n\n\
        ## Key Metrics\n\
        - Lines Added: +{}\n\
        - Lines Removed: -{}\n\
        - Commits Analyzed: {}\n\n\
        ## Recommendation\n\
        {}\n\n\
        ## AI Assessment\n\
        {}",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        state.review.branch_comparison.source_branch,
        state.review.branch_comparison.target_branch,
        state.review.metrics.files_modified,
        state.all_issues.len(),
        state
            .all_issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Critical))
            .count(),
        state
            .all_issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::High))
            .count(),
        state.review.metrics.lines_added,
        state.review.metrics.lines_removed,
        state.review.branch_comparison.commits_analyzed.len(),
        if state
            .all_issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Critical))
        {
            "🚨 **DO NOT MERGE** - Critical issues require immediate attention."
        } else if state
            .all_issues
            .iter()
            .any(|i| matches!(i.severity, Severity::High))
        {
            "⚠️ **REVIEW REQUIRED** - Address high-priority issues before merging."
        } else {
            "✅ **APPROVED** - No critical issues found. Consider addressing minor findings."
        },
        state.review.overall_assessment
    )
}

fn generate_critical_issues_report(state: &AppState) -> String {
    let mut report = String::from("# Critical Issues Report\n\n");

    let critical_issues: Vec<_> = state
        .all_issues
        .iter()
        .filter(|i| matches!(i.severity, Severity::Critical | Severity::High))
        .collect();

    if critical_issues.is_empty() {
        report.push_str("✅ **GOOD NEWS!** No critical or high-priority issues found.\n\n");
        report.push_str("The code review found no blocking issues. You may proceed with the merge after considering any minor findings.\n");
    } else {
        report.push_str(&format!(
            "⚠️ **ACTION REQUIRED** - {} critical/high-priority issues found:\n\n",
            critical_issues.len()
        ));

        for (i, issue) in critical_issues.iter().enumerate() {
            let severity_icon = match issue.severity {
                Severity::Critical => "🚨",
                Severity::High => "⚠️",
                _ => "",
            };

            report.push_str(&format!(
                "## {}{} Issue {}: {:?}\n\n",
                severity_icon,
                severity_icon,
                i + 1,
                issue.category
            ));
            report.push_str(&format!("**File:** `{}`\n", issue.file_path));
            if let Some(line) = issue.line_number {
                report.push_str(&format!("**Line:** {line}\n"));
            }
            report.push_str(&format!("**Severity:** {:?}\n\n", issue.severity));

            report.push_str(&format!("**Problem:** {}\n\n", issue.description));
            report.push_str(&format!("**Solution:** {}\n\n", issue.suggestion));

            if let Some(ref snippet) = issue.code_snippet {
                report.push_str("**Code Location:**\n```\n");
                report.push_str(snippet);
                report.push_str("\n```\n\n");
            }

            report.push_str("---\n\n");
        }

        report.push_str("## Next Steps\n\n");
        report.push_str("1. 🔥 Address all Critical issues immediately\n");
        report.push_str("2. ⚠️ Review and fix High-priority issues\n");
        report.push_str("3. ✅ Re-run code review after fixes\n");
        report.push_str("4. 🚀 Proceed with merge only after all critical issues are resolved\n");
    }

    report
}
