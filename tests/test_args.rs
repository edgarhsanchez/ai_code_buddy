use ai_code_buddy::args::{Args, OutputFormat};
use clap::Parser;

#[test]
fn test_args_default_values() {
    let args = Args::parse_from(&["ai-code-buddy"]);
    
    assert_eq!(args.repo_path, ".");
    assert_eq!(args.source_branch, "main");
    assert_eq!(args.target_branch, "HEAD");
    assert_eq!(args.output_format, OutputFormat::Summary);
    assert!(!args.show_credits);
    assert!(!args.force_cpu);
}

#[test]
fn test_args_custom_values() {
    let args = Args::parse_from(&[
        "ai-code-buddy",
        "/path/to/repo",
        "--source", "develop",
        "--target", "feature-branch",
        "--format", "json",
        "--cpu",
        "--credits"
    ]);
    
    assert_eq!(args.repo_path, "/path/to/repo");
    assert_eq!(args.source_branch, "develop");
    assert_eq!(args.target_branch, "feature-branch");
    assert_eq!(args.output_format, OutputFormat::Json);
    assert!(args.show_credits);
    assert!(args.force_cpu);
}

#[test]
fn test_output_format_parsing() {
    let formats = [
        ("summary", OutputFormat::Summary),
        ("detailed", OutputFormat::Detailed),
        ("json", OutputFormat::Json),
        ("markdown", OutputFormat::Markdown),
    ];
    
    for (format_str, expected) in formats {
        let args = Args::parse_from(&["ai-code-buddy", "--format", format_str]);
        assert_eq!(args.output_format, expected);
    }
}

#[test]
fn test_gpu_flag_combinations() {
    // Test --gpu flag explicitly set
    let args = Args::parse_from(&["ai-code-buddy", "--gpu"]);
    assert!(args.use_gpu);
    assert!(!args.force_cpu);
    
    // Test --cpu flag - force_cpu should be true
    let args = Args::parse_from(&["ai-code-buddy", "--cpu"]);
    // When --cpu is specified, force_cpu is true (CPU is forced)
    assert!(args.force_cpu);
    // use_gpu may still be true due to default, but force_cpu takes precedence
    
    // Test no flags (should auto-detect)
    let args = Args::parse_from(&["ai-code-buddy"]);
    // GPU availability depends on compile-time features
    // When compiled with --no-default-features, GPU should not be available
    #[cfg(not(gpu_available))]
    assert!(!args.use_gpu);
    #[cfg(gpu_available)]
    assert!(args.use_gpu);
    assert!(!args.force_cpu); // force_cpu should be false by default
}

#[test]
fn test_invalid_output_format() {
    let result = Args::try_parse_from(&["ai-code-buddy", "--format", "invalid"]);
    assert!(result.is_err());
}

#[test]
fn test_help_flag() {
    let result = Args::try_parse_from(&["ai-code-buddy", "--help"]);
    assert!(result.is_err()); // Help flag causes early exit
}

#[test]
fn test_version_flag() {
    let result = Args::try_parse_from(&["ai-code-buddy", "--version"]);
    assert!(result.is_err()); // Version flag causes early exit
}
