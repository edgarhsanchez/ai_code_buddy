use crate::review::{CodeIssue, IssueCategory, Review, Severity};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap,
    },
    Frame, Terminal,
};
use std::io::{stdout};

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
        let filtered_issues: Vec<usize> = (0..all_issues.len()).collect();
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
                            AppTab::Files => {
                                if app.selected_file_index > 0 {
                                    app.selected_file_index -= 1;
                                }
                            },
                            _ => {}
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        match app.current_tab {
                            AppTab::Issues => app.next_issue(),
                            AppTab::Files => {
                                let files = app.get_files();
                                if app.selected_file_index + 1 < files.len() {
                                    app.selected_file_index += 1;
                                }
                            },
                            _ => {}
                        }
                    }
                    KeyCode::Enter => {
                        if app.current_tab == AppTab::Issues {
                            app.show_issue_detail = !app.show_issue_detail;
                        }
                    }
                    KeyCode::Char('b') => {
                        if app.current_tab == AppTab::Issues && app.show_issue_detail {
                            app.show_issue_detail = false;
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
fn ui(f: &mut Frame, app: &mut App) {
    let size = f.area();
    
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

fn render_header(f: &mut Frame, area: Rect, app: &App) {
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

fn render_overview(f: &mut Frame, area: Rect, app: &App) {
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

fn render_issues(f: &mut Frame, area: Rect, app: &mut App) {
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

        f.render_stateful_widget(issues_list, area, &mut app.issue_list_state);
    }
}

fn render_issue_detail(f: &mut Frame, area: Rect, issue: &CodeIssue) {
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

fn render_files(f: &mut Frame, area: Rect, app: &mut App) {
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

    // Update file list state
    if !files.is_empty() {
        if app.selected_file_index >= files.len() {
            app.selected_file_index = 0;
        }
        app.file_list_state.select(Some(app.selected_file_index));
    } else {
        app.file_list_state.select(None);
    }

    let files_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Changed Files"))
        .highlight_style(Style::default().bg(Color::DarkGray))
        .highlight_symbol("► ");

    f.render_stateful_widget(files_list, area, &mut app.file_list_state);
}

fn render_reports(f: &mut Frame, area: Rect, _app: &App) {
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

fn render_help(f: &mut Frame, area: Rect, _app: &App) {
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

fn render_status_bar(f: &mut Frame, area: Rect, app: &App) {
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

fn generate_summary_report(app: &App) -> String {
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
        app.review.branch_comparison.source_branch,
        app.review.branch_comparison.target_branch,
        app.review.metrics.files_modified,
        app.all_issues.len(),
        app.all_issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::Critical))
            .count(),
        app.all_issues
            .iter()
            .filter(|i| matches!(i.severity, Severity::High))
            .count(),
        app.review.metrics.lines_added,
        app.review.metrics.lines_removed,
        app.review.branch_comparison.commits_analyzed.len(),
        if app.all_issues
            .iter()
            .any(|i| matches!(i.severity, Severity::Critical))
        {
            "🚨 **DO NOT MERGE** - Critical issues require immediate attention."
        } else if app.all_issues
            .iter()
            .any(|i| matches!(i.severity, Severity::High))
        {
            "⚠️ **REVIEW REQUIRED** - Address high-priority issues before merging."
        } else {
            "✅ **APPROVED** - No critical issues found. Consider addressing minor findings."
        },
        app.review.overall_assessment
    )
}

fn generate_critical_issues_report(app: &App) -> String {
    let mut report = String::from("# Critical Issues Report\n\n");

    let critical_issues: Vec<_> = app.all_issues
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
