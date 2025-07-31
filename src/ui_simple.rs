use crate::review::{CodeIssue, IssueCategory, Review, Severity};
use std::io::{self, Write};

#[derive(Debug, Clone)]
pub enum ViewMode {
    Overview,
    IssuesList,
    IssueDetail(usize),
    ReportGeneration,
    Files,
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
    #[allow(dead_code)]
    pub scroll_position: usize,
    #[allow(dead_code)]
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

// Simple terminal-based UI runner
pub async fn run_tui(review: Review) -> anyhow::Result<()> {
    println!("🚀 Starting interactive review interface...");

    let mut state = AppState::new(review);

    // Simple text-based UI
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
        println!("\nEnter command (h for help): ");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let command = input.trim();

        // Handle commands
        match command {
            "q" | "quit" => break,
            "h" | "help" => {
                print_help();
                println!("\nPress Enter to continue...");
                let mut _input = String::new();
                io::stdin().read_line(&mut _input)?;
            }
            "o" | "overview" => state.view_mode = ViewMode::Overview,
            "i" | "issues" => state.view_mode = ViewMode::IssuesList,
            "r" | "report" => state.view_mode = ViewMode::ReportGeneration,
            "f" | "files" => state.view_mode = ViewMode::Files,
            "j" | "down" | "n" | "next" => state.next_issue(),
            "k" | "up" | "p" | "prev" => state.prev_issue(),
            "enter" | "details" => {
                if matches!(state.view_mode, ViewMode::IssuesList) {
                    if let Some(&issue_idx) = state.filtered_issues.get(state.selected_issue_index)
                    {
                        state.view_mode = ViewMode::IssueDetail(issue_idx);
                    }
                }
            }
            "b" | "back" => {
                if matches!(state.view_mode, ViewMode::IssueDetail(_)) {
                    state.view_mode = ViewMode::IssuesList;
                }
            }
            "1" => {
                if matches!(state.view_mode, ViewMode::ReportGeneration) {
                    let report = state.generate_review_report();
                    std::fs::write("code_review_report.md", &report)?;
                    println!("✅ Full report saved to code_review_report.md");
                    println!("Press Enter to continue...");
                    let mut _input = String::new();
                    io::stdin().read_line(&mut _input)?;
                }
            }
            "2" => {
                if matches!(state.view_mode, ViewMode::ReportGeneration) {
                    let summary = generate_summary_report(&state);
                    std::fs::write("code_review_summary.md", &summary)?;
                    println!("✅ Summary saved to code_review_summary.md");
                    println!("Press Enter to continue...");
                    let mut _input = String::new();
                    io::stdin().read_line(&mut _input)?;
                }
            }
            "3" => {
                if matches!(state.view_mode, ViewMode::ReportGeneration) {
                    let critical_report = generate_critical_issues_report(&state);
                    std::fs::write("critical_issues.md", &critical_report)?;
                    println!("✅ Critical issues report saved to critical_issues.md");
                    println!("Press Enter to continue...");
                    let mut _input = String::new();
                    io::stdin().read_line(&mut _input)?;
                }
            }
            "c" | "clear" => {
                if matches!(state.view_mode, ViewMode::IssuesList) {
                    state.selected_category = None;
                    state.selected_severity = None;
                    state.apply_filters();
                }
            }
            _ => {
                println!("❌ Unknown command: '{command}'. Type 'h' for help.");
                println!("Press Enter to continue...");
                let mut _input = String::new();
                io::stdin().read_line(&mut _input)?;
            }
        }
    }

    println!("👋 Thanks for using AI Code Review!");
    Ok(())
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
