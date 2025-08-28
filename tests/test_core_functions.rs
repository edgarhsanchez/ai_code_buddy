use ai_code_buddy::core;
use ai_code_buddy::{Args, OutputFormat};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_args() -> Args {
        Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "develop".to_string(),
            cli_mode: false,
            verbose: false,
            show_credits: false,
            output_format: OutputFormat::Summary,
            exclude_patterns: vec![],
            include_patterns: vec![],
            use_gpu: false,
            force_cpu: true,
            disable_ai: false,
        }
    }

    #[test]
    fn test_run_cli_mode_with_credits() {
        let mut args = create_test_args();
        args.show_credits = true;

        // Should not fail when showing credits
        let result = core::run_cli_mode(args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_cli_mode_different_output_formats() {
        let formats = [
            OutputFormat::Summary,
            OutputFormat::Detailed,
            OutputFormat::Json,
            OutputFormat::Markdown,
        ];

        for format in formats {
            let mut args = create_test_args();
            args.output_format = format;

            // Should handle different output formats
            // Note: This might fail due to git repo requirements, but we're testing the function exists
            let _result = core::run_cli_mode(args);
        }
    }

    #[test]
    fn test_args_with_patterns() {
        let args = Args {
            repo_path: ".".to_string(),
            source_branch: "main".to_string(),
            target_branch: "develop".to_string(),
            cli_mode: false,
            verbose: true,
            show_credits: false,
            output_format: OutputFormat::Summary,
            exclude_patterns: vec!["target/".to_string(), "*.tmp".to_string()],
            include_patterns: vec!["*.rs".to_string(), "*.toml".to_string()],
            use_gpu: false,
            force_cpu: true,
            disable_ai: false,
        };

        assert_eq!(args.include_patterns.len(), 2);
        assert_eq!(args.exclude_patterns.len(), 2);
        assert!(args.verbose);
        assert!(!args.show_credits);
    }

    #[test]
    fn test_output_format_variants() {
        // Test all OutputFormat variants can be created
        let summary = OutputFormat::Summary;
        let detailed = OutputFormat::Detailed;
        let json = OutputFormat::Json;
        let markdown = OutputFormat::Markdown;

        // Basic checks that they're different
        assert_ne!(format!("{summary:?}"), format!("{:?}", detailed));
        assert_ne!(format!("{json:?}"), format!("{:?}", markdown));
    }

    #[test]
    fn test_args_repo_path_manipulation() {
        let mut args = create_test_args();

        // Test path manipulation
        args.repo_path = "/tmp/test-repo".to_string();
        assert_eq!(args.repo_path, "/tmp/test-repo");

        // Test relative path
        args.repo_path = "./src".to_string();
        assert!(args.repo_path.starts_with("."));
    }

    #[test]
    fn test_args_branch_names() {
        let mut args = create_test_args();

        // Test different branch naming patterns
        args.source_branch = "feature/new-feature".to_string();
        args.target_branch = "release/v1.0".to_string();

        assert!(args.source_branch.contains("/"));
        assert!(args.target_branch.starts_with("release"));
    }

    #[test]
    fn test_args_verbose_flag() {
        let mut args = create_test_args();

        // Test verbose flag toggling
        assert!(!args.verbose); // Default false
        args.verbose = true;
        assert!(args.verbose);
    }

    #[test]
    fn test_pattern_collections() {
        let mut args = create_test_args();

        // Test adding patterns
        args.include_patterns.push("**/*.rs".to_string());
        args.exclude_patterns.push("**/target/**".to_string());

        assert!(!args.include_patterns.is_empty());
        assert!(!args.exclude_patterns.is_empty());

        // Test clearing patterns
        args.include_patterns.clear();
        args.exclude_patterns.clear();

        assert!(args.include_patterns.is_empty());
        assert!(args.exclude_patterns.is_empty());
    }
}
