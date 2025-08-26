use clap::Parser;

/// Check if GPU acceleration is available at compile time
fn is_gpu_available() -> bool {
    #[cfg(gpu_available)]
    {
        true
    }
    #[cfg(not(gpu_available))]
    {
        false
    }
}

#[derive(Parser, Clone, Debug, Resource)]
#[command(
    name = "ai-code-buddy",
    version = "0.2.0",
    about = "🤖 AI-powered code review tool with elegant TUI",
    long_about = "AI Code Buddy is an intelligent code analysis tool that compares branches, \
                  detects security vulnerabilities, performance issues, and code quality problems. \
                  Features a modern Bevy-powered TUI with real-time analysis and reporting."
)]
pub struct Args {
    /// Git repository path to analyze
    #[arg(
        value_name = "REPO_PATH",
        default_value = ".",
        help = "Path to the Git repository (default: current directory)"
    )]
    pub repo_path: String,

    /// Source branch for comparison
    #[arg(
        short = 's',
        long = "source",
        value_name = "BRANCH",
        default_value = "main",
        help = "Source branch to compare from"
    )]
    pub source_branch: String,

    /// Target branch for comparison
    #[arg(
        short = 't',
        long = "target",
        value_name = "BRANCH",
        default_value = "HEAD",
        help = "Target branch to compare to (default: HEAD)"
    )]
    pub target_branch: String,

    /// Use CLI mode instead of interactive TUI
    #[arg(
        long = "cli",
        help = "Run in CLI mode with text output instead of interactive interface"
    )]
    pub cli_mode: bool,

    /// Enable verbose output
    #[arg(
        short = 'v',
        long = "verbose",
        help = "Enable verbose output for debugging"
    )]
    pub verbose: bool,

    /// Show credits and contributors
    #[arg(
        long = "credits",
        help = "Show credits and list all contributors to the project"
    )]
    pub show_credits: bool,

    /// Output format for results
    #[arg(
        short = 'f',
        long = "format",
        value_enum,
        default_value = "summary",
        help = "Output format for results"
    )]
    pub output_format: OutputFormat,

    /// Exclude files matching pattern
    #[arg(
        long = "exclude",
        value_name = "PATTERN",
        help = "Exclude files matching glob pattern (can be used multiple times)",
        action = clap::ArgAction::Append
    )]
    pub exclude_patterns: Vec<String>,

    /// Include only files matching pattern
    #[arg(
        long = "include",
        value_name = "PATTERN",
        help = "Include only files matching glob pattern (can be used multiple times)",
        action = clap::ArgAction::Append
    )]
    pub include_patterns: Vec<String>,

    /// Enable GPU acceleration for AI analysis
    #[arg(
        long = "gpu",
        help = "Enable GPU acceleration (Metal on Apple, CUDA on NVIDIA, auto-detected)",
        default_value_t = is_gpu_available()
    )]
    pub use_gpu: bool,

    /// Force CPU mode (disable GPU acceleration)
    #[arg(
        long = "cpu",
        help = "Force CPU mode (disable GPU acceleration even if available)",
        conflicts_with = "use_gpu"
    )]
    pub force_cpu: bool,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum OutputFormat {
    /// Summary output with key findings
    Summary,
    /// Detailed output with all issues
    Detailed,
    /// JSON format for programmatic use
    Json,
    /// Markdown format for documentation
    Markdown,
}

use bevy::prelude::Resource;
