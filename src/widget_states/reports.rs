use crate::core::review::Review;
use bevy::prelude::*;
use serde_json;

#[derive(Debug, Clone, Resource)]
pub struct ReportsWidgetState {
    pub review: Option<Review>,
    pub selected_format: ReportFormat,
    pub export_status: ExportStatus,
    pub generated_report: Option<String>,
    pub view_mode: ViewMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Selection, // Format selection and preview
    Report,    // Full report display
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReportFormat {
    Summary,
    Detailed,
    Json,
    Markdown,
}

#[derive(Debug, Clone)]
pub enum ExportStatus {
    None,
    Exporting(String), // format
    Success(String),   // path
}

impl Default for ReportsWidgetState {
    fn default() -> Self {
        Self {
            review: None,
            selected_format: ReportFormat::Summary,
            export_status: ExportStatus::None,
            generated_report: None,
            view_mode: ViewMode::Selection,
        }
    }
}

impl ReportsWidgetState {
    pub fn set_review(&mut self, review: Review) {
        self.review = Some(review);
    }

    pub fn next_format(&mut self) {
        self.selected_format = match self.selected_format {
            ReportFormat::Summary => ReportFormat::Detailed,
            ReportFormat::Detailed => ReportFormat::Json,
            ReportFormat::Json => ReportFormat::Markdown,
            ReportFormat::Markdown => ReportFormat::Summary,
        };
    }

    pub fn previous_format(&mut self) {
        self.selected_format = match self.selected_format {
            ReportFormat::Summary => ReportFormat::Markdown,
            ReportFormat::Detailed => ReportFormat::Summary,
            ReportFormat::Json => ReportFormat::Detailed,
            ReportFormat::Markdown => ReportFormat::Json,
        };
    }

    pub fn start_export(&mut self, format: String) {
        self.export_status = ExportStatus::Exporting(format);
    }

    pub fn complete_export(&mut self, path: String) {
        self.export_status = ExportStatus::Success(path);
    }

    pub fn generate_report(&mut self) -> Option<String> {
        if let Some(review) = &self.review {
            let report_content = match self.selected_format {
                ReportFormat::Summary => self.generate_summary_report(review),
                ReportFormat::Detailed => self.generate_detailed_report(review),
                ReportFormat::Json => self.generate_json_report(review),
                ReportFormat::Markdown => self.generate_markdown_report(review),
            };
            self.generated_report = Some(report_content.clone());
            self.view_mode = ViewMode::Report;
            Some(report_content)
        } else {
            None
        }
    }

    pub fn back_to_selection(&mut self) {
        self.view_mode = ViewMode::Selection;
    }

    fn generate_summary_report(&self, review: &Review) -> String {
        format!(
            "🤖 AI Code Review Summary\n\
             ========================\n\n\
             📊 Analysis Results:\n\
             • Files analyzed: {}\n\
             • Total issues found: {}\n\n\
             🚨 Issue Breakdown:\n\
             • Critical: {} issues\n\
             • High: {} issues\n\
             • Medium: {} issues\n\
             • Low: {} issues\n\n\
             📋 Recommendations:\n\
             {} Focus on addressing Critical and High severity issues first.\n\
             {} Review Medium issues for code quality improvements.\n\
             {} Low severity issues can be addressed as time permits.\n\n\
             🎯 Next Steps:\n\
             1. Review each Critical issue immediately\n\
             2. Plan fixes for High severity issues\n\
             3. Consider Medium issues for future iterations\n\
             4. Use the detailed report for specific guidance",
            review.files_count,
            review.issues_count,
            review.critical_issues,
            review.high_issues,
            review.medium_issues,
            review.low_issues,
            if review.critical_issues > 0 {
                "⚠️"
            } else {
                "✅"
            },
            if review.medium_issues > 0 {
                "📝"
            } else {
                "✅"
            },
            if review.low_issues > 0 { "💡" } else { "✅" }
        )
    }

    fn generate_detailed_report(&self, review: &Review) -> String {
        let mut report = format!(
            "🤖 AI Code Review - Detailed Report\n\
             ===================================\n\n\
             📊 Overview:\n\
             • Repository analyzed\n\
             • Files processed: {}\n\
             • Total issues: {}\n\n",
            review.files_count, review.issues_count
        );

        if review.issues.is_empty() {
            report.push_str("🎉 No issues found! Your code looks great!\n");
            return report;
        }

        // Group issues by severity
        let mut critical_issues = Vec::new();
        let mut high_issues = Vec::new();
        let mut medium_issues = Vec::new();
        let mut low_issues = Vec::new();

        for issue in &review.issues {
            match issue.severity.as_str() {
                "Critical" => critical_issues.push(issue),
                "High" => high_issues.push(issue),
                "Medium" => medium_issues.push(issue),
                "Low" => low_issues.push(issue),
                _ => low_issues.push(issue),
            }
        }

        // Critical issues
        if !critical_issues.is_empty() {
            report.push_str("🚨 CRITICAL ISSUES (Immediate Action Required):\n");
            report.push_str("=====================================================\n\n");
            for (i, issue) in critical_issues.iter().enumerate() {
                report.push_str(&format!(
                    "{}. File: {}\n   Line: {}\n   Category: {}\n   Issue: {}\n\n",
                    i + 1,
                    issue.file,
                    issue.line,
                    issue.category,
                    issue.description
                ));
            }
        }

        // High issues
        if !high_issues.is_empty() {
            report.push_str("⚠️  HIGH PRIORITY ISSUES:\n");
            report.push_str("=========================\n\n");
            for (i, issue) in high_issues.iter().enumerate() {
                report.push_str(&format!(
                    "{}. File: {}\n   Line: {}\n   Category: {}\n   Issue: {}\n\n",
                    i + 1,
                    issue.file,
                    issue.line,
                    issue.category,
                    issue.description
                ));
            }
        }

        // Medium issues
        if !medium_issues.is_empty() {
            report.push_str("🔶 MEDIUM PRIORITY ISSUES:\n");
            report.push_str("==========================\n\n");
            for (i, issue) in medium_issues.iter().enumerate() {
                report.push_str(&format!(
                    "{}. File: {}\n   Line: {}\n   Category: {}\n   Issue: {}\n\n",
                    i + 1,
                    issue.file,
                    issue.line,
                    issue.category,
                    issue.description
                ));
            }
        }

        // Low issues
        if !low_issues.is_empty() {
            report.push_str("ℹ️  LOW PRIORITY ISSUES:\n");
            report.push_str("========================\n\n");
            for (i, issue) in low_issues.iter().enumerate() {
                report.push_str(&format!(
                    "{}. File: {}\n   Line: {}\n   Category: {}\n   Issue: {}\n\n",
                    i + 1,
                    issue.file,
                    issue.line,
                    issue.category,
                    issue.description
                ));
            }
        }

        report.push_str("\n📝 End of Report\n");
        report
    }

    fn generate_json_report(&self, review: &Review) -> String {
        // Use serde to generate proper JSON
        match serde_json::to_string_pretty(review) {
            Ok(json) => json,
            Err(_) => "Error generating JSON report".to_string(),
        }
    }

    fn generate_markdown_report(&self, review: &Review) -> String {
        let mut report = format!(
            "# 🤖 AI Code Review Report\n\n\
             ## 📊 Summary\n\n\
             - **Files analyzed:** {}\n\
             - **Total issues:** {}\n\
             - **Critical issues:** {}\n\
             - **High priority:** {}\n\
             - **Medium priority:** {}\n\
             - **Low priority:** {}\n\n",
            review.files_count,
            review.issues_count,
            review.critical_issues,
            review.high_issues,
            review.medium_issues,
            review.low_issues
        );

        if review.issues.is_empty() {
            report.push_str("## 🎉 Results\n\nNo issues found! Your code looks great!\n");
            return report;
        }

        report.push_str("## 📋 Issues by Severity\n\n");

        // Group and display issues by severity
        for severity in ["Critical", "High", "Medium", "Low"] {
            let severity_issues: Vec<_> = review
                .issues
                .iter()
                .filter(|issue| issue.severity == severity)
                .collect();

            if !severity_issues.is_empty() {
                let icon = match severity {
                    "Critical" => "🚨",
                    "High" => "⚠️",
                    "Medium" => "🔶",
                    "Low" => "ℹ️",
                    _ => "📝",
                };

                report.push_str(&format!("### {icon} {severity} Priority Issues\n\n"));

                for issue in severity_issues {
                    report.push_str(&format!(
                        "- **File:** `{}`\n  **Line:** {}\n  **Category:** {}\n  **Issue:** {}\n\n",
                        issue.file, issue.line, issue.category, issue.description
                    ));
                }
            }
        }

        report.push_str("---\n\n*Report generated by AI Code Buddy*\n");
        report
    }
}
