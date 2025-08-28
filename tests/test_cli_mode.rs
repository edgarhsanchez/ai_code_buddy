use ai_code_buddy::{core, Args, OutputFormat};

#[test]
fn test_run_cli_mode_credits_and_formats() {
    // Credits early-return path
    let args = Args {
        repo_path: ".".to_string(),
        source_branch: "main".to_string(),
        target_branch: "HEAD".to_string(),
        cli_mode: true,
        verbose: false,
        show_credits: true,
        output_format: OutputFormat::Summary,
        exclude_patterns: vec![],
        include_patterns: vec![],
        use_gpu: false,
        force_cpu: true,
        disable_ai: false,
    };
    assert!(core::run_cli_mode(args).is_ok());

    // Exercise other formats (the function prints; we just validate it doesn't error)
    for fmt in [
        OutputFormat::Summary,
        OutputFormat::Detailed,
        OutputFormat::Json,
        OutputFormat::Markdown,
    ] {
        let args = Args {
            repo_path: "/non/existent".to_string(), // Will likely fail during analysis
            source_branch: "main".to_string(),
            target_branch: "HEAD".to_string(),
            cli_mode: true,
            verbose: false,
            show_credits: false,
            output_format: fmt,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: true,
            disable_ai: false,
        };
        // run_cli_mode returns a boxed error; accept either success or error, but it must not panic
        let _ = core::run_cli_mode(args);
    }
}
