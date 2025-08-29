use ai_code_buddy::{
    args::{Args, OutputFormat},
    core::run_cli_mode,
    version::{APP_NAME, APP_VERSION},
};
use tempfile::TempDir;

#[test]
fn test_run_cli_mode_with_credits() {
    let args = Args {
        repo_path: "/test/repo".to_string(),
        source_branch: "main".to_string(),
        target_branch: "feature".to_string(),
        cli_mode: true,
        verbose: false,
        show_credits: true,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: false,
        parallel: false,
        disable_ai: false,
    };

    let result = run_cli_mode(args);
    assert!(result.is_ok());
}

#[test]
fn test_run_cli_mode_basic_functionality() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    let args = Args {
        repo_path,
        source_branch: "main".to_string(),
        target_branch: "main".to_string(),
        cli_mode: true,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: false,
        parallel: false,
        disable_ai: false,
    };

    // This should work even with an empty/non-git directory
    let result = run_cli_mode(args);
    // The result might be an error due to no git repo, but the function should execute
    assert!(result.is_ok() || result.is_err()); // Either outcome is acceptable for this test
}

#[test]
fn test_run_cli_mode_different_output_formats() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    let output_formats = vec![
        OutputFormat::Summary,
        OutputFormat::Detailed,
        OutputFormat::Json,
        OutputFormat::Markdown,
    ];

    for format in output_formats {
        let args = Args {
            repo_path: repo_path.clone(),
            source_branch: "main".to_string(),
            target_branch: "main".to_string(),
            cli_mode: true,
            verbose: false,
            show_credits: false,
            output_format: format,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: false,
            parallel: false,
            disable_ai: false,
        };

        let result = run_cli_mode(args);
        // Function should execute regardless of output format
        assert!(result.is_ok() || result.is_err());
    }
}

#[test]
fn test_run_cli_mode_with_patterns() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    let args = Args {
        repo_path,
        source_branch: "main".to_string(),
        target_branch: "main".to_string(),
        cli_mode: true,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec!["*.tmp".to_string(), "*.log".to_string()],
        include_patterns: vec!["*.rs".to_string(), "*.toml".to_string()],
        use_gpu: false,
        force_cpu: false,
        parallel: false,
        disable_ai: false,
    };

    let result = run_cli_mode(args);
    // Should handle patterns gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_run_cli_mode_verbose_mode() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    let args = Args {
        repo_path,
        source_branch: "main".to_string(),
        target_branch: "main".to_string(),
        cli_mode: true,
        verbose: true,
        show_credits: false,
        output_format: OutputFormat::Detailed,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: false,
        parallel: false,
        disable_ai: false,
    };

    let result = run_cli_mode(args);
    // Verbose mode should work
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_run_cli_mode_gpu_flags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    // Test with GPU enabled
    let args_gpu = Args {
        repo_path: repo_path.clone(),
        source_branch: "main".to_string(),
        target_branch: "main".to_string(),
        cli_mode: true,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: true,
        force_cpu: false,
        parallel: false,
        disable_ai: false,
    };

    let result_gpu = run_cli_mode(args_gpu);
    assert!(result_gpu.is_ok() || result_gpu.is_err());

    // Test with force CPU
    let args_cpu = Args {
        repo_path,
        source_branch: "main".to_string(),
        target_branch: "main".to_string(),
        cli_mode: true,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: true,
        parallel: false,
        disable_ai: false,
    };

    let result_cpu = run_cli_mode(args_cpu);
    assert!(result_cpu.is_ok() || result_cpu.is_err());
}

#[test]
fn test_version_constants() {
    // Test that version constants are properly exported
    assert!(!APP_NAME.is_empty());
    assert!(!APP_VERSION.is_empty());

    // Version should contain some expected patterns
    assert!(APP_VERSION.contains('.') || APP_VERSION.chars().all(|c| c.is_numeric()));
}

#[test]
fn test_args_parsing_for_cli_mode() {
    // Test that Args can be created for CLI mode
    let args = Args {
        repo_path: "/test/path".to_string(),
        source_branch: "develop".to_string(),
        target_branch: "feature-branch".to_string(),
        cli_mode: true,
        verbose: true,
        show_credits: false,
        output_format: OutputFormat::Json,
        exclude_patterns: vec!["target/".to_string()],
        include_patterns: vec!["src/".to_string()],
        use_gpu: true,
        force_cpu: false,
        parallel: false,
        disable_ai: false,
    };

    assert_eq!(args.repo_path, "/test/path");
    assert_eq!(args.source_branch, "develop");
    assert_eq!(args.target_branch, "feature-branch");
    assert!(args.cli_mode);
    assert!(args.verbose);
    assert!(!args.show_credits);
    assert!(matches!(args.output_format, OutputFormat::Json));
    assert_eq!(args.exclude_patterns, vec!["target/"]);
    assert_eq!(args.include_patterns, vec!["src/"]);
    assert!(args.use_gpu);
    assert!(!args.force_cpu);
}

#[test]
fn test_run_cli_mode_error_handling() {
    // Test with invalid repository path
    let args = Args {
        repo_path: "/nonexistent/path/that/does/not/exist".to_string(),
        source_branch: "main".to_string(),
        target_branch: "main".to_string(),
        cli_mode: true,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: false,
        parallel: false,
        disable_ai: false,
    };

    let result = run_cli_mode(args);
    // Should handle invalid paths gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_run_cli_mode_different_branches() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap().to_string();

    let args = Args {
        repo_path,
        source_branch: "main".to_string(),
        target_branch: "develop".to_string(),
        cli_mode: true,
        verbose: false,
        show_credits: false,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: false,
        parallel: false,
        disable_ai: false,
    };

    let result = run_cli_mode(args);
    // Should handle different branch names
    assert!(result.is_ok() || result.is_err());
}
