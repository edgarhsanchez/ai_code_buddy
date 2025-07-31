use iocraft::prelude::*;
use crossterm::event::{KeyCode, KeyEvent};
use crate::review::{Review, CodeIssue, IssueCategory, Severity};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ViewMode {
    Overview,
    IssuesList,
    IssueDetail(usize),
    ReportGeneration,
    Files,
}

#[derive(Default, Props)]
pub struct ReviewAppProps {
    pub review: Review,
}

#[component]
pub fn ReviewApp(props: &ReviewAppProps) -> impl Into<AnyElement> {
    let mut hooks = use_hook();
    let state = hooks.use_state(|| AppState::new(props.review.clone()));
    
    let app_state = state.get();
    
    Box::new(move |ctx: &mut ComponentContext| {
        let theme = ctx.theme();
        
        let main_content = match &app_state.view_mode {
            ViewMode::Overview => render_overview(&app_state, theme),
            ViewMode::IssuesList => render_issues_list(&app_state, theme),
            ViewMode::IssueDetail(idx) => render_issue_detail(&app_state, *idx, theme),
            ViewMode::ReportGeneration => render_report_generation(&app_state, theme),
            ViewMode::Files => render_files_view(&app_state, theme),
        };
        
        VStack::new()
            .child(render_header(&app_state, theme))
            .child(main_content)
            .child(render_status_bar(&app_state, theme))
            .child(
                if app_state.show_help {
                    render_help(theme)
                } else {
                    Text::new("Press 'h' for help, 'q' to quit").into()
                }
            )
            .into()
    })
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub review: Review,
    pub view_mode: ViewMode,
    pub selected_issue_index: usize,
    pub selected_category: Option<IssueCategory>,
    pub selected_severity: Option<Severity>,
    pub all_issues: Vec<CodeIssue>,
    pub filtered_issues: Vec<usize>, // indices into all_issues
    pub scroll_position: usize,
    pub show_help: bool,
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
        }
    }
    
    fn collect_all_issues(review: &Review) -> Vec<CodeIssue> {
        let mut issues = Vec::new();
        for category_issues in review.issues.values() {
            issues.extend(category_issues.clone());
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
        self.filtered_issues = self.all_issues
            .iter()
            .enumerate()
            .filter(|(_, issue)| {
                let category_match = self.selected_category.as_ref()
                    .map_or(true, |cat| &issue.category == cat);
                let severity_match = self.selected_severity.as_ref()
                    .map_or(true, |sev| &issue.severity == sev);
                category_match && severity_match
            })
            .map(|(idx, _)| idx)
            .collect();
        
        if self.selected_issue_index >= self.filtered_issues.len() {
            self.selected_issue_index = 0;
        }
    }
    
    pub fn get_current_issue(&self) -> Option<&CodeIssue> {
        self.filtered_issues.get(self.selected_issue_index)
            .and_then(|&idx| self.all_issues.get(idx))
    }
    
    pub fn next_issue(&mut self) {
        if !self.filtered_issues.is_empty() {
            self.selected_issue_index = (self.selected_issue_index + 1) % self.filtered_issues.len();
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
        report.push_str(&format!("**Branches Compared:** {} → {}\n", 
            self.review.branch_comparison.source_branch,
            self.review.branch_comparison.target_branch));
        report.push_str(&format!("**Generated:** {}\n\n", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        // Executive Summary
        report.push_str("## Executive Summary\n\n");
        let critical_count = self.all_issues.iter().filter(|i| matches!(i.severity, Severity::Critical)).count();
        let high_count = self.all_issues.iter().filter(|i| matches!(i.severity, Severity::High)).count();
        let total_issues = self.all_issues.len();
        
        if critical_count > 0 {
            report.push_str(&format!("⚠️ **CRITICAL:** {} critical issues require immediate attention before merge.\n", critical_count));
        }
        if high_count > 0 {
            report.push_str(&format!("🔶 **HIGH:** {} high-priority issues should be addressed.\n", high_count));
        }
        report.push_str(&format!("📊 **Total Issues:** {} findings across {} files.\n\n", 
            total_issues, self.review.metrics.files_modified));
        
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
                    report.push_str(&format!("{} **{:?}** in `{}`\n", 
                        severity_icon, issue.category, issue.file_path));
                    if let Some(line) = issue.line_number {
                        report.push_str(&format!("   Line {}: {}\n", line, issue.description));
                    } else {
                        report.push_str(&format!("   {}\n", issue.description));
                    }
                    report.push_str(&format!("   *Recommendation:* {}\n\n", issue.suggestion));
                }
            }
        }
        
        // AI Assessment
        if !self.review.overall_assessment.is_empty() {
            report.push_str("## AI Analysis\n\n");
            report.push_str(&self.review.overall_assessment);
            report.push_str("\n\n");
        }
        
        report
    }
}

fn render_header(state: &AppState, theme: &Theme) -> AnyElement {
    HStack::new()
        .child(Text::new("🔍 AI Code Review - ").color(Color::Blue))
        .child(Text::new(&format!("{} → {}", 
            state.review.branch_comparison.source_branch,
            state.review.branch_comparison.target_branch)).color(Color::Blue))
        .padding(Padding::all(1))
        .background_color(Color::White)
        .into()
}

fn render_overview(state: &AppState, theme: &Theme) -> AnyElement {
    let critical_count = state.all_issues.iter()
        .filter(|i| matches!(i.severity, Severity::Critical)).count();
    let high_count = state.all_issues.iter()
        .filter(|i| matches!(i.severity, Severity::High)).count();
    let medium_count = state.all_issues.iter()
        .filter(|i| matches!(i.severity, Severity::Medium)).count();
    let low_count = state.all_issues.iter()
        .filter(|i| matches!(i.severity, Severity::Low)).count();
    
    VStack::new()
        .child(Text::new("📊 Review Summary").weight(FontWeight::Bold))
        .child(Text::new(&format!("Files Modified: {}", state.review.metrics.files_modified)))
        .child(Text::new(&format!("Lines Added: +{}", state.review.metrics.lines_added)))
        .child(Text::new(&format!("Lines Removed: -{}", state.review.metrics.lines_removed)))
        .child(Text::new(&format!("Total Issues: {}", state.all_issues.len())))
        .child(Text::new(""))
        .child(Text::new("🚨 Issues by Severity").weight(FontWeight::Bold))
        .child(if critical_count > 0 {
            Text::new(&format!("Critical: {}", critical_count)).color(Color::Red)
        } else {
            Text::new("")
        })
        .child(if high_count > 0 {
            Text::new(&format!("High: {}", high_count)).color(Color::Yellow)
        } else {
            Text::new("")
        })
        .child(Text::new(&format!("Medium: {}", medium_count)))
        .child(Text::new(&format!("Low: {}", low_count)))
        .child(Text::new(""))
        .child(Text::new("Navigation:"))
        .child(Text::new("  'i' - View Issues List"))
        .child(Text::new("  'r' - Generate Report"))
        .child(Text::new("  'f' - View Files"))
        .child(Text::new("  'q' - Quit"))
        .into()
}

fn render_issues_list(state: &AppState, theme: &Theme) -> AnyElement {
    let mut stack = VStack::new()
        .child(Text::new(&format!("🐛 Issues ({}/{})", 
            state.filtered_issues.len(), state.all_issues.len())).weight(FontWeight::Bold))
        .child(Text::new("Filters: 'c' - Category, 's' - Severity, 'x' - Clear"))
        .child(Text::new(""));
    
    for (display_idx, &issue_idx) in state.filtered_issues.iter().enumerate() {
        let issue = &state.all_issues[issue_idx];
        let is_selected = display_idx == state.selected_issue_index;
        
        let severity_text = match issue.severity {
            Severity::Critical => Text::new("Critical").color(Color::Red),
            Severity::High => Text::new("High").color(Color::Yellow),
            Severity::Medium => Text::new("Medium").color(Color::Blue),
            Severity::Low => Text::new("Low").color(Color::Green),
            Severity::Info => Text::new("Info").color(Color::Cyan),
        };
        
        let issue_line = HStack::new()
            .child(severity_text)
            .child(Text::new(" "))
            .child(Text::new(&format!("{:?}", issue.category)))
            .child(Text::new(" "))
            .child(Text::new(&issue.file_path))
            .child(if let Some(line) = issue.line_number {
                Text::new(&format!(":{}", line))
            } else {
                Text::new("")
            });
        
        if is_selected {
            stack = stack.child(issue_line.background_color(Color::White));
        } else {
            stack = stack.child(issue_line);
        }
    }
    
    stack
        .child(Text::new(""))
        .child(Text::new("↑/↓ Navigate, Enter - Details, 'o' - Overview"))
        .into()
}

fn render_issue_detail(state: &AppState, issue_idx: usize, theme: &Theme) -> AnyElement {
    if let Some(issue) = state.all_issues.get(issue_idx) {
        let mut stack = VStack::new()
            .child(Text::new("🔍 Issue Details").weight(FontWeight::Bold))
            .child(Text::new(""))
            .child(Text::new(&format!("File: {}", issue.file_path)))
            .child(if let Some(line) = issue.line_number {
                Text::new(&format!("Line: {}", line))
            } else {
                Text::new("")
            })
            .child(Text::new(&format!("Category: {:?}", issue.category)))
            .child(Text::new(&format!("Severity: {:?}", issue.severity)))
            .child(Text::new(""))
            .child(Text::new("Description:").weight(FontWeight::Bold))
            .child(Text::new(&issue.description))
            .child(Text::new(""))
            .child(Text::new("Suggestion:").weight(FontWeight::Bold))
            .child(Text::new(&issue.suggestion));
        
        if let Some(ref snippet) = issue.code_snippet {
            stack = stack
                .child(Text::new(""))
                .child(Text::new("Code Snippet:").weight(FontWeight::Bold))
                .child(Text::new(snippet).background_color(Color::Black).color(Color::White));
        }
        
        stack
            .child(Text::new(""))
            .child(Text::new("'b' - Back to list, ←/→ Navigate issues"))
            .into()
    } else {
        Text::new("Issue not found").into()
    }
}

fn render_report_generation(state: &AppState, theme: &Theme) -> AnyElement {
    VStack::new()
        .child(Text::new("📄 Generate Report").weight(FontWeight::Bold))
        .child(Text::new(""))
        .child(Text::new("Choose report format:"))
        .child(Text::new("  '1' - Markdown Report"))
        .child(Text::new("  '2' - Summary Report"))
        .child(Text::new("  '3' - Critical Issues Only"))
        .child(Text::new("  'o' - Back to Overview"))
        .into()
}

fn render_files_view(state: &AppState, theme: &Theme) -> AnyElement {
    let files: Vec<String> = state.review.branch_comparison.commits_analyzed
        .iter()
        .flat_map(|c| &c.files_changed)
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    let mut stack = VStack::new()
        .child(Text::new("📁 Changed Files").weight(FontWeight::Bold))
        .child(Text::new(""));
    
    for file in &files {
        let issue_count = state.all_issues.iter()
            .filter(|i| i.file_path == *file)
            .count();
        
        let file_line = HStack::new()
            .child(Text::new(file))
            .child(if issue_count > 0 {
                Text::new(&format!(" ({} issues)", issue_count))
            } else {
                Text::new("")
            });
        
        stack = stack.child(file_line);
    }
    
    stack
        .child(Text::new(""))
        .child(Text::new("'o' - Back to Overview"))
        .into()
}

fn render_status_bar(state: &AppState, theme: &Theme) -> AnyElement {
    HStack::new()
        .child(Text::new(&format!("Mode: {:?} | Issues: {} | ", 
            state.view_mode, state.filtered_issues.len())))
        .child(if !state.filtered_issues.is_empty() {
            Text::new(&format!("Selected: {}/{}", 
                state.selected_issue_index + 1, 
                state.filtered_issues.len()))
        } else {
            Text::new("")
        })
        .padding(Padding::all(1))
        .background_color(Color::Black)
        .color(Color::White)
        .into()
}

fn render_help(theme: &Theme) -> AnyElement {
    VStack::new()
        .child(Text::new("🆘 Help"))
        .child(Text::new("Global: 'q' - Quit, 'h' - Toggle help"))
        .child(Text::new("Navigation: 'o' - Overview, 'i' - Issues, 'r' - Report, 'f' - Files"))
        .child(Text::new("Issues List: ↑/↓ - Navigate, Enter - Details, 'c' - Filter category, 's' - Filter severity"))
        .child(Text::new("Issue Detail: 'b' - Back, ←/→ - Previous/Next issue"))
        .child(Text::new("Report: '1'/'2'/'3' - Generate different report types"))
        .padding(Padding::all(1))
        .background_color(Color::Black)
        .color(Color::White)
        .into()
}

// Simple TUI runner - IOCraft has its own app system
pub async fn run_tui(review: Review) -> anyhow::Result<()> {
    // For now, let's create a simpler CLI-based interactive interface
    // until we can properly integrate IOCraft
    println!("🚀 Starting interactive review interface...");
    
    let mut state = AppState::new(review);
    
    // Simple text-based UI for now
    loop {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");
        
        // Print current view
        match state.view_mode {
            ViewMode::Overview => print_overview(&state),
            ViewMode::IssuesList => print_issues_list(&state),
            ViewMode::IssueDetail(idx) => print_issue_detail(&state, idx),
            ViewMode::ReportGeneration => print_report_generation(&state),
            ViewMode::Files => print_files_view(&state),
        }
        
        // Get user input
        println!("\nEnter command: ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let command = input.trim();
        
        // Handle commands
        match command {
            "q" => break,
            "h" => state.show_help = !state.show_help,
            "o" => state.view_mode = ViewMode::Overview,
            "i" => state.view_mode = ViewMode::IssuesList,
            "r" => state.view_mode = ViewMode::ReportGeneration,
            "f" => state.view_mode = ViewMode::Files,
            "j" | "down" => state.next_issue(),
            "k" | "up" => state.prev_issue(),
            "enter" => {
                if matches!(state.view_mode, ViewMode::IssuesList) {
                    if let Some(&issue_idx) = state.filtered_issues.get(state.selected_issue_index) {
                        state.view_mode = ViewMode::IssueDetail(issue_idx);
                    }
                }
            }
            "b" => {
                if matches!(state.view_mode, ViewMode::IssueDetail(_)) {
                    state.view_mode = ViewMode::IssuesList;
                }
            }
            "1" => {
                if matches!(state.view_mode, ViewMode::ReportGeneration) {
                    let report = state.generate_review_report();
                    std::fs::write("code_review_report.md", &report)?;
                    println!("Report saved to code_review_report.md");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
            "2" => {
                if matches!(state.view_mode, ViewMode::ReportGeneration) {
                    let summary = generate_summary_report(&state);
                    std::fs::write("code_review_summary.md", &summary)?;
                    println!("Summary saved to code_review_summary.md");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
            "3" => {
                if matches!(state.view_mode, ViewMode::ReportGeneration) {
                    let critical_report = generate_critical_issues_report(&state);
                    std::fs::write("critical_issues.md", &critical_report)?;
                    println!("Critical issues report saved to critical_issues.md");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
            _ => {}
        }
    }
    
    Ok(())
}

fn print_overview(state: &AppState) {
    println!("📊 Code Review Overview");
    println!("======================");
    println!("Branches: {} → {}", 
        state.review.branch_comparison.source_branch,
        state.review.branch_comparison.target_branch);
    println!("Files Modified: {}", state.review.metrics.files_modified);
    println!("Lines Added: +{}", state.review.metrics.lines_added);
    println!("Lines Removed: -{}", state.review.metrics.lines_removed);
    println!("Total Issues: {}", state.all_issues.len());
    
    let critical_count = state.all_issues.iter().filter(|i| matches!(i.severity, Severity::Critical)).count();
    let high_count = state.all_issues.iter().filter(|i| matches!(i.severity, Severity::High)).count();
    
    if critical_count > 0 {
        println!("🚨 Critical Issues: {}", critical_count);
    }
    if high_count > 0 {
        println!("⚠️ High Priority Issues: {}", high_count);
    }
    
    println!("\nCommands: o=Overview, i=Issues, r=Report, f=Files, q=Quit");
}

fn print_issues_list(state: &AppState) {
    println!("🐛 Issues List ({}/{})", state.filtered_issues.len(), state.all_issues.len());
    println!("=================");
    
    for (display_idx, &issue_idx) in state.filtered_issues.iter().enumerate() {
        let issue = &state.all_issues[issue_idx];
        let marker = if display_idx == state.selected_issue_index { ">" } else { " " };
        
        println!("{} [{:?}] {:?} - {} {}",
            marker,
            issue.severity,
            issue.category,
            issue.file_path,
            if let Some(line) = issue.line_number { format!(":{}", line) } else { String::new() }
        );
    }
    
    println!("\nCommands: j/k=Navigate, enter=Details, o=Overview");
}

fn print_issue_detail(state: &AppState, issue_idx: usize) {
    if let Some(issue) = state.all_issues.get(issue_idx) {
        println!("🔍 Issue Details");
        println!("================");
        println!("File: {}", issue.file_path);
        if let Some(line) = issue.line_number {
            println!("Line: {}", line);
        }
        println!("Category: {:?}", issue.category);
        println!("Severity: {:?}", issue.severity);
        println!("\nDescription:");
        println!("{}", issue.description);
        println!("\nSuggestion:");
        println!("{}", issue.suggestion);
        
        if let Some(ref snippet) = issue.code_snippet {
            println!("\nCode Snippet:");
            println!("{}", snippet);
        }
        
        println!("\nCommands: b=Back, j/k=Navigate issues");
    }
}

fn print_report_generation(state: &AppState) {
    println!("📄 Generate Report");
    println!("==================");
    println!("1 - Full Markdown Report");
    println!("2 - Summary Report");
    println!("3 - Critical Issues Only");
    println!("\nCommands: 1/2/3=Generate, o=Overview");
}

fn print_files_view(state: &AppState) {
    println!("📁 Changed Files");
    println!("================");
    
    let files: Vec<String> = state.review.branch_comparison.commits_analyzed
        .iter()
        .flat_map(|c| &c.files_changed)
        .cloned()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    
    for file in &files {
        let issue_count = state.all_issues.iter()
            .filter(|i| i.file_path == *file)
            .count();
        
        if issue_count > 0 {
            println!("{} ({} issues)", file, issue_count);
        } else {
            println!("{}", file);
        }
    }
    
    println!("\nCommands: o=Overview");
}

fn generate_summary_report(state: &AppState) -> String {
    format!("# Code Review Summary\n\n\
        **Branches:** {} → {}\n\
        **Files Changed:** {}\n\
        **Total Issues:** {}\n\
        **Critical Issues:** {}\n\
        **High Priority Issues:** {}\n\n\
        {}",
        state.review.branch_comparison.source_branch,
        state.review.branch_comparison.target_branch,
        state.review.metrics.files_modified,
        state.all_issues.len(),
        state.all_issues.iter().filter(|i| matches!(i.severity, Severity::Critical)).count(),
        state.all_issues.iter().filter(|i| matches!(i.severity, Severity::High)).count(),
        state.review.overall_assessment
    )
}

fn generate_critical_issues_report(state: &AppState) -> String {
    let mut report = String::from("# Critical Issues Report\n\n");
    
    let critical_issues: Vec<_> = state.all_issues.iter()
        .filter(|i| matches!(i.severity, Severity::Critical | Severity::High))
        .collect();
    
    if critical_issues.is_empty() {
        report.push_str("✅ No critical or high-priority issues found.\n");
    } else {
        report.push_str(&format!("⚠️ {} critical/high-priority issues found:\n\n", critical_issues.len()));
        
        for (i, issue) in critical_issues.iter().enumerate() {
            report.push_str(&format!("## Issue {}\n\n", i + 1));
            report.push_str(&format!("**File:** `{}`\n", issue.file_path));
            if let Some(line) = issue.line_number {
                report.push_str(&format!("**Line:** {}\n", line));
            }
            report.push_str(&format!("**Severity:** {:?}\n", issue.severity));
            report.push_str(&format!("**Category:** {:?}\n\n", issue.category));
            report.push_str(&format!("**Issue:** {}\n\n", issue.description));
            report.push_str(&format!("**Recommendation:** {}\n\n", issue.suggestion));
            
            if let Some(ref snippet) = issue.code_snippet {
                report.push_str("**Code:**\n```\n");
                report.push_str(snippet);
                report.push_str("\n```\n\n");
            }
            
            report.push_str("---\n\n");
        }
    }
    
    report
}
