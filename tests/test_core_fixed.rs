use ai_code_buddy::{
    args::Args,
    core::{
        ai_analyzer::{AnalysisRequest, GpuBackend, ProgressUpdate},
        git::GitAnalyzer,
        review::{CommitStatus, Issue, Review},
    },
};
use clap::Parser;
use tempfile::TempDir;

#[cfg(test)]
mod core_ai_analyzer_tests {
    use super::*;

    #[test]
    fn test_analysis_request_structure() {
        let request = AnalysisRequest {
            file_path: "src/test.rs".to_string(),
            content: "fn main() { println!(\"Hello\"); }".to_string(),
            language: "rust".to_string(),
            commit_status: CommitStatus::Modified,
        };

        assert_eq!(request.file_path, "src/test.rs");
        assert_eq!(request.language, "rust");
        assert!(request.content.contains("Hello"));
    }

    #[test]
    fn test_analysis_request_serialization() {
        let request = AnalysisRequest {
            file_path: "test.rs".to_string(),
            content: "test content".to_string(),
            language: "rust".to_string(),
            commit_status: CommitStatus::Modified,
        };

        let json = serde_json::to_string(&request).expect("Should serialize");
        let deserialized: AnalysisRequest =
            serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(request.file_path, deserialized.file_path);
        assert_eq!(request.content, deserialized.content);
        assert_eq!(request.language, deserialized.language);
    }

    #[test]
    fn test_progress_update_structure() {
        let progress = ProgressUpdate {
            current_file: "src/main.rs".to_string(),
            progress: 0.5,
            stage: "analyzing".to_string(),
        };

        assert_eq!(progress.current_file, "src/main.rs");
        assert_eq!(progress.progress, 0.5);
        assert_eq!(progress.stage, "analyzing");
    }

    #[test]
    fn test_gpu_backend_variants() {
        let backends = [
            GpuBackend::Cpu,
            GpuBackend::Cuda,
            GpuBackend::Metal,
            GpuBackend::Mkl,
        ];

        for backend in backends {
            let display_str = format!("{backend}");
            assert!(!display_str.is_empty());

            let debug_str = format!("{backend:?}");
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_gpu_backend_display() {
        assert_eq!(format!("{}", GpuBackend::Metal), "Metal");
        assert_eq!(format!("{}", GpuBackend::Cuda), "CUDA");
        assert_eq!(format!("{}", GpuBackend::Mkl), "MKL");
        assert_eq!(format!("{}", GpuBackend::Cpu), "CPU");
    }

    #[test]
    fn test_gpu_backend_equality() {
        assert_eq!(GpuBackend::Cpu, GpuBackend::Cpu);
        assert_eq!(GpuBackend::Cuda, GpuBackend::Cuda);
        assert_ne!(GpuBackend::Cpu, GpuBackend::Cuda);
        assert_ne!(GpuBackend::Metal, GpuBackend::Mkl);
    }

    #[test]
    fn test_analysis_request_with_different_commit_statuses() {
        let statuses = [
            CommitStatus::Committed,
            CommitStatus::Staged,
            CommitStatus::Modified,
            CommitStatus::Untracked,
        ];

        for status in statuses {
            let request = AnalysisRequest {
                file_path: "test.rs".to_string(),
                content: "test".to_string(),
                language: "rust".to_string(),
                commit_status: status,
            };

            // Test that the request was created successfully
            assert_eq!(request.language, "rust");
        }
    }

    #[test]
    fn test_analysis_request_with_different_languages() {
        let languages = ["rust", "javascript", "python", "go", "typescript"];

        for lang in languages {
            let request = AnalysisRequest {
                file_path: format!("test.{lang}"),
                content: "test content".to_string(),
                language: lang.to_string(),
                commit_status: CommitStatus::Modified,
            };

            assert_eq!(request.language, lang);
            assert!(request.file_path.contains(lang));
        }
    }

    #[test]
    fn test_progress_update_bounds() {
        let mut progress = ProgressUpdate {
            current_file: "test.rs".to_string(),
            progress: 0.0,
            stage: "start".to_string(),
        };

        // Test valid progress values
        progress.progress = 0.0;
        assert_eq!(progress.progress, 0.0);

        progress.progress = 0.5;
        assert_eq!(progress.progress, 0.5);

        progress.progress = 1.0;
        assert_eq!(progress.progress, 1.0);
    }

    #[test]
    fn test_analysis_request_large_content() {
        let large_content = "a".repeat(100_000);
        let request = AnalysisRequest {
            file_path: "large_file.txt".to_string(),
            content: large_content.clone(),
            language: "text".to_string(),
            commit_status: CommitStatus::Modified,
        };

        assert_eq!(request.content.len(), 100_000);
        assert_eq!(request.content, large_content);
    }

    #[test]
    fn test_analysis_request_unicode_content() {
        let unicode_content = "Hello, 世界! 🚀 Rust is awesome!";
        let request = AnalysisRequest {
            file_path: "unicode_test.rs".to_string(),
            content: unicode_content.to_string(),
            language: "rust".to_string(),
            commit_status: CommitStatus::Modified,
        };

        assert!(request.content.contains("世界"));
        assert!(request.content.contains("🚀"));
        assert!(request.content.contains("Rust"));
    }
}

#[cfg(test)]
mod core_git_tests {
    use super::*;

    #[test]
    fn test_git_analyzer_with_invalid_repo() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path().to_str().unwrap();

        // Test with non-git directory
        let result = GitAnalyzer::new(repo_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_git_analyzer_with_valid_repo() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");

        // Test with valid git directory
        let result = GitAnalyzer::new(repo_path.to_str().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_analyzer_file_operations() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Create and commit a test file
        let test_content = "Hello, Git!";
        std::fs::write(repo_path.join("test.txt"), test_content).unwrap();

        std::process::Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["commit", "-m", "Add test file"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let analyzer = GitAnalyzer::new(repo_path.to_str().unwrap()).unwrap();

        // Test getting changed files (should work even with no changes between branches)
        let result = analyzer.get_changed_files("HEAD", "HEAD");
        assert!(result.is_ok());
    }

    #[test]
    fn test_git_analyzer_error_handling() {
        // Test with non-existent path
        let result = GitAnalyzer::new("/non/existent/path");
        assert!(result.is_err());

        // Test with regular file instead of directory
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("not_a_directory.txt");
        std::fs::write(&file_path, "content").unwrap();

        let result = GitAnalyzer::new(file_path.to_str().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn test_git_analyzer_branch_scenarios() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let repo_path = temp_dir.path();

        // Initialize git repo
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(repo_path)
            .output()
            .expect("Failed to init git repo");

        std::process::Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Create initial commit
        std::fs::write(repo_path.join("README.md"), "# Test Repo").unwrap();
        std::process::Command::new("git")
            .args(["add", "README.md"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let analyzer = GitAnalyzer::new(repo_path.to_str().unwrap()).unwrap();

        // Test various branch comparison scenarios
        let result = analyzer.get_changed_files("HEAD", "HEAD");
        assert!(result.is_ok());

        // Test with non-existent branches (should return error or empty result)
        let result = analyzer.get_changed_files("non-existent-branch", "HEAD");
        // This might succeed with empty changes or fail - both are valid behaviors
        println!("Non-existent branch result: {result:?}");
    }
}

#[cfg(test)]
mod core_review_tests {
    use super::*;

    #[test]
    fn test_review_structure() {
        let review = Review {
            files_count: 5,
            issues_count: 10,
            critical_issues: 1,
            high_issues: 2,
            medium_issues: 3,
            low_issues: 4,
            issues: Vec::new(),
        };

        assert_eq!(review.files_count, 5);
        assert_eq!(review.issues_count, 10);
        assert_eq!(
            review.critical_issues + review.high_issues + review.medium_issues + review.low_issues,
            10
        );
    }

    #[test]
    fn test_commit_status_variants() {
        let statuses = [
            CommitStatus::Committed,
            CommitStatus::Staged,
            CommitStatus::Modified,
            CommitStatus::Untracked,
        ];

        for status in statuses {
            // Test that each variant can be cloned
            let _cloned_status = status.clone();

            // Test debug formatting
            let debug_str = format!("{status:?}");
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_issue_creation() {
        let issue = Issue {
            file: "src/main.rs".to_string(),
            line: 42,
            severity: "high".to_string(),
            category: "security".to_string(),
            description: "Potential security vulnerability".to_string(),
            commit_status: CommitStatus::Modified,
        };

        assert_eq!(issue.file, "src/main.rs");
        assert_eq!(issue.line, 42);
        assert_eq!(issue.severity, "high");
        assert_eq!(issue.category, "security");
    }

    #[test]
    fn test_review_with_issues() {
        let issues = vec![
            Issue {
                file: "src/main.rs".to_string(),
                line: 10,
                severity: "critical".to_string(),
                category: "bug".to_string(),
                description: "Critical issue".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                file: "src/lib.rs".to_string(),
                line: 20,
                severity: "medium".to_string(),
                category: "style".to_string(),
                description: "Medium issue".to_string(),
                commit_status: CommitStatus::Staged,
            },
        ];

        let review = Review {
            files_count: 2,
            issues_count: issues.len(),
            critical_issues: 1,
            high_issues: 0,
            medium_issues: 1,
            low_issues: 0,
            issues,
        };

        assert_eq!(review.issues.len(), 2);
        assert_eq!(review.issues[0].severity, "critical");
        assert_eq!(review.issues[1].severity, "medium");
    }

    #[test]
    fn test_review_serialization() {
        let review = Review {
            files_count: 1,
            issues_count: 1,
            critical_issues: 0,
            high_issues: 0,
            medium_issues: 1,
            low_issues: 0,
            issues: vec![Issue {
                file: "test.rs".to_string(),
                line: 1,
                severity: "medium".to_string(),
                category: "style".to_string(),
                description: "Test issue".to_string(),
                commit_status: CommitStatus::Modified,
            }],
        };

        let json = serde_json::to_string(&review).expect("Should serialize");
        let deserialized: Review = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(review.files_count, deserialized.files_count);
        assert_eq!(review.issues_count, deserialized.issues_count);
        assert_eq!(review.issues.len(), deserialized.issues.len());
    }
}

#[cfg(test)]
mod args_parsing_tests {
    use super::*;

    #[test]
    fn test_args_parsing_variants() {
        // Test basic parsing
        let args = Args::parse_from(["test", "."]);
        assert_eq!(args.repo_path, ".");

        // Test with GPU options
        let args = Args::parse_from(["test", ".", "--cpu"]);
        assert!(args.force_cpu);

        // Test with output format
        let args = Args::parse_from(["test", ".", "--format", "json"]);
        assert!(matches!(
            args.output_format,
            ai_code_buddy::args::OutputFormat::Json
        ));
    }

    #[test]
    fn test_args_default_values() {
        let args = Args::parse_from(["test", "test_repo"]);

        assert_eq!(args.repo_path, "test_repo");
        assert_eq!(args.source_branch, "main");
        assert_eq!(args.target_branch, "HEAD");
        assert!(!args.force_cpu);
        assert!(!args.show_credits);
    }

    #[test]
    fn test_args_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let args = Arc::new(Args::parse_from(["test", "."]));
        let mut handles = vec![];

        for _ in 0..5 {
            let args_clone = args.clone();
            let handle = thread::spawn(move || {
                // Test configuration access from multiple threads
                let _repo_path = &args_clone.repo_path;
                let _source_branch = &args_clone.source_branch;
                let _target_branch = &args_clone.target_branch;
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

#[cfg(test)]
mod comprehensive_integration_tests {
    use super::*;

    #[test]
    fn test_full_data_structures_integration() {
        // Create a complete review with real data structures
        let issues = vec![
            Issue {
                file: "src/main.rs".to_string(),
                line: 15,
                severity: "high".to_string(),
                category: "performance".to_string(),
                description: "Inefficient algorithm detected".to_string(),
                commit_status: CommitStatus::Modified,
            },
            Issue {
                file: "src/lib.rs".to_string(),
                line: 32,
                severity: "medium".to_string(),
                category: "style".to_string(),
                description: "Code formatting inconsistency".to_string(),
                commit_status: CommitStatus::Staged,
            },
        ];

        let review = Review {
            files_count: 10,
            issues_count: issues.len(),
            critical_issues: 0,
            high_issues: 1,
            medium_issues: 1,
            low_issues: 0,
            issues,
        };

        let analysis_request = AnalysisRequest {
            file_path: "src/main.rs".to_string(),
            content: "fn main() { /* some code */ }".to_string(),
            language: "rust".to_string(),
            commit_status: CommitStatus::Modified,
        };

        let progress = ProgressUpdate {
            current_file: "src/main.rs".to_string(),
            progress: 0.75,
            stage: "finalizing".to_string(),
        };

        // Test that all structures integrate properly
        assert_eq!(review.issues.len(), 2);
        assert_eq!(analysis_request.language, "rust");
        assert_eq!(progress.progress, 0.75);

        // Test serialization of the complete review
        let json = serde_json::to_string(&review).expect("Should serialize complete review");
        let _deserialized: Review =
            serde_json::from_str(&json).expect("Should deserialize complete review");
    }

    #[test]
    fn test_gpu_backend_comprehensive() {
        let all_backends = [
            GpuBackend::Cpu,
            GpuBackend::Cuda,
            GpuBackend::Metal,
            GpuBackend::Mkl,
        ];

        // Test that all backends can be formatted and compared
        for (i, backend) in all_backends.iter().enumerate() {
            let display = format!("{backend}");
            let debug = format!("{backend:?}");

            assert!(!display.is_empty());
            assert!(!debug.is_empty());

            // Test equality with itself
            assert_eq!(backend, &all_backends[i]);

            // Test inequality with others
            for (j, other) in all_backends.iter().enumerate() {
                if i != j {
                    assert_ne!(backend, other);
                }
            }
        }
    }

    #[test]
    fn test_commit_status_comprehensive() {
        let all_statuses = [
            CommitStatus::Committed,
            CommitStatus::Staged,
            CommitStatus::Modified,
            CommitStatus::Untracked,
        ];

        for status in &all_statuses {
            // Test cloning
            let _cloned = status.clone();

            // Test debug formatting
            let debug_str = format!("{status:?}");
            assert!(!debug_str.is_empty());

            // Test serialization
            let json = serde_json::to_string(status).expect("Should serialize status");
            let _deserialized: CommitStatus =
                serde_json::from_str(&json).expect("Should deserialize status");
        }
    }
}
