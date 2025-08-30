use clap::Parser;
use bevy::prelude::Resource;

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
    version,
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
        help = "Source branch to compare from (default: auto-detected default branch)"
    )]
    pub source_branch: Option<String>,

    /// Target branch for comparison
    #[arg(
        short = 't',
        long = "target",
        value_name = "BRANCH",
        help = "Target branch to compare to (default: current branch/HEAD)"
    )]
    pub target_branch: Option<String>,

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

    /// Enable parallel file analysis using Rayon
    #[arg(
        long = "parallel",
        help = "Enable parallel file analysis using all available CPU cores",
        default_value = "true"
    )]
    pub parallel: bool,

    /// AI model to use for analysis
    #[arg(
        long = "model",
        help = "AI model to use for analysis (default: auto-selected based on system)",
        value_name = "MODEL"
    )]
    pub model: Option<String>,

    /// Show available AI models and installation options
    #[arg(
        long = "list-models",
        help = "Show available AI models and installation commands",
        action = clap::ArgAction::SetTrue
    )]
    pub list_models: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
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

impl Args {
    /// Get the source branch, auto-detecting default if not provided
    pub fn get_source_branch(&self, repo_path: &str) -> String {
        self.source_branch.clone().unwrap_or_else(|| {
            Self::detect_default_branch(repo_path).unwrap_or_else(|| "main".to_string())
        })
    }

    /// Get the target branch, defaulting to HEAD if not provided
    pub fn get_target_branch(&self) -> String {
        self.target_branch.clone().unwrap_or_else(|| "HEAD".to_string())
    }

    /// Detect the default branch of a git repository
    fn detect_default_branch(repo_path: &str) -> Option<String> {
        use std::process::Command;
        
        // Try to get the default branch from git
        let output = Command::new("git")
            .args(["-C", repo_path, "symbolic-ref", "refs/remotes/origin/HEAD"])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Extract branch name from "refs/remotes/origin/main"
            if let Some(branch) = stdout.trim().strip_prefix("refs/remotes/origin/") {
                return Some(branch.to_string());
            }
        }

        // Fallback: try to get the current branch
        let output = Command::new("git")
            .args(["-C", repo_path, "rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let branch = stdout.trim();
            if !branch.is_empty() && branch != "HEAD" {
                return Some(branch.to_string());
            }
        }

        None
    }

    /// Display available AI models and installation commands
    pub fn print_model_help() {
        println!("🤖 Available AI Models for Code Analysis\n");
        
        println!("📋 Quick Setup Commands:");
        println!("  cargo install ai-code-buddy                    # Basic installation with auto-GPU detection");
        println!("  cargo install ai-code-buddy --features llama   # Explicit Llama support");
        println!("  cargo install ai-code-buddy --features gpu-metal  # Force Metal GPU (macOS)");
        println!("  cargo install ai-code-buddy --features gpu-cuda   # Force CUDA GPU (Linux/Windows)");
        println!("  cargo install ai-code-buddy --features gpu-mkl    # Force Intel MKL acceleration\n");
        
        println!("🔧 Model Categories:");
        println!("  Default: Auto-selected based on system capabilities");
        println!("  Small:   1-4B parameters (fast, good for quick analysis)");
        println!("  Medium:  7-13B parameters (balanced speed/quality)");
        println!("  Large:   22B+ parameters (highest quality, slower)\n");
        
        println!("📦 Available Models (use with --model <name>):");
        
        println!("  🚀 Fast Models (Good for CI/CD):");
        println!("    tiny_llama_1_1b_chat     - TinyLlama 1.1B Chat (fastest)");
        println!("    phi_3_mini_4k_instruct   - Microsoft Phi-3 Mini 4K");
        println!("    qwen_2_5_0_5b_instruct   - Qwen 2.5 0.5B (ultra-fast)");
        println!("    qwen_2_5_1_5b_instruct   - Qwen 2.5 1.5B");
        
        println!("\n  ⚡ Balanced Models (Recommended):");
        println!("    llama_3_2_3b_chat        - Llama 3.2 3B Chat");
        println!("    qwen_2_5_3b_instruct     - Qwen 2.5 3B Instruct");
        println!("    phi_3_1_mini_4k_instruct - Microsoft Phi-3.1 Mini");
        println!("    qwen_2_5_7b_instruct     - Qwen 2.5 7B Instruct");
        
        println!("\n  🧠 Quality Models (Best Analysis):");
        println!("    llama_8b_chat            - Llama 3 8B Chat");
        println!("    llama_3_1_8b_chat        - Llama 3.1 8B Chat");
        println!("    mistral_7b_instruct      - Mistral 7B Instruct");
        println!("    phi_4                    - Microsoft Phi-4 14B");
        
        println!("\n  🔒 Code-Specialized Models:");
        println!("    llama_7b_code            - Llama 2 7B Code");
        println!("    llama_13b_code           - Llama 2 13B Code");
        println!("    codestral_22b            - Mistral Codestral 22B");
        
        println!("\n💡 Usage Examples:");
        println!("  ai-code-buddy --model phi_3_mini_4k_instruct --gpu .");
        println!("  ai-code-buddy --model llama_8b_chat --parallel .");
        println!("  ai-code-buddy --model codestral_22b --gpu --format json .");
        
        println!("\n🔧 GPU Acceleration:");
        println!("  • Metal: Automatically enabled on macOS with Apple Silicon");
        println!("  • CUDA:  Automatically enabled with NVIDIA GPUs");
        println!("  • MKL:   Automatically enabled with Intel processors");
        println!("  • Use --cpu to force CPU-only mode");
        
        println!("\n📊 Performance Tips:");
        println!("  • Smaller models (1-4B): Great for CI/CD pipelines");
        println!("  • Medium models (7-8B): Best balance of speed and quality");
        println!("  • Large models (13B+): Highest quality but slower");
        println!("  • Code models: Specialized for programming languages");
        println!("  • Use --parallel for faster multi-file analysis");
    }
}
