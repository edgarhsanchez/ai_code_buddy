use ai_code_buddy::{Args, OutputFormat};

#[test]
fn test_output_format_debug() {
    // Test Debug format since Display is not implemented
    assert_eq!(format!("{:?}", OutputFormat::Summary), "Summary");
    assert_eq!(format!("{:?}", OutputFormat::Detailed), "Detailed");
    assert_eq!(format!("{:?}", OutputFormat::Json), "Json");
    assert_eq!(format!("{:?}", OutputFormat::Markdown), "Markdown");
}

#[test]
fn test_args_construction() {
    let args = Args {
        repo_path: "/test/path".to_string(),
        source_branch: "dev".to_string(),
        target_branch: "main".to_string(),
        cli_mode: true,
        verbose: true,
        show_credits: false,
        output_format: OutputFormat::Json,
        exclude_patterns: vec!["*.tmp".to_string()],
        include_patterns: vec!["*.rs".to_string()],
        use_gpu: true,
        force_cpu: false,
    };

    assert_eq!(args.repo_path, "/test/path");
    assert_eq!(args.source_branch, "dev");
    assert_eq!(args.target_branch, "main");
    assert!(args.cli_mode);
    assert!(args.verbose);
    assert_eq!(args.output_format, OutputFormat::Json);
    assert_eq!(args.exclude_patterns, vec!["*.tmp".to_string()]);
    assert_eq!(args.include_patterns, vec!["*.rs".to_string()]);
    assert!(args.use_gpu);
    assert!(!args.force_cpu);
}

#[test]
fn test_args_defaults() {
    let args = Args {
        repo_path: ".".to_string(),
        source_branch: "main".to_string(),
        target_branch: "HEAD".to_string(),
        cli_mode: false,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: true,
    };

    assert_eq!(args.repo_path, ".");
    assert_eq!(args.source_branch, "main");
    assert_eq!(args.target_branch, "HEAD");
    assert!(!args.cli_mode);
    assert!(!args.verbose);
    assert!(!args.show_credits);
    assert_eq!(args.output_format, OutputFormat::Summary);
    assert!(args.exclude_patterns.is_empty());
    assert!(args.include_patterns.is_empty());
    assert!(!args.use_gpu);
    assert!(args.force_cpu);
}

#[test]
fn test_output_format_equality() {
    assert_eq!(OutputFormat::Summary, OutputFormat::Summary);
    assert_eq!(OutputFormat::Detailed, OutputFormat::Detailed);
    assert_eq!(OutputFormat::Json, OutputFormat::Json);
    assert_eq!(OutputFormat::Markdown, OutputFormat::Markdown);

    assert_ne!(OutputFormat::Summary, OutputFormat::Detailed);
    assert_ne!(OutputFormat::Json, OutputFormat::Markdown);
}

#[test]
fn test_args_patterns() {
    let args = Args {
        repo_path: "/project".to_string(),
        source_branch: "feature".to_string(),
        target_branch: "develop".to_string(),
        cli_mode: true,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Detailed,
        exclude_patterns: vec!["*.log".to_string(), "*.tmp".to_string()],
        include_patterns: vec!["*.rs".to_string(), "*.toml".to_string()],
        use_gpu: false,
        force_cpu: true,
    };

    assert_eq!(args.exclude_patterns.len(), 2);
    assert_eq!(args.include_patterns.len(), 2);
    assert!(args.exclude_patterns.contains(&"*.log".to_string()));
    assert!(args.include_patterns.contains(&"*.rs".to_string()));
}
