use ai_code_buddy::core::ai_analyzer::{AnalysisRequest, GpuBackend, ProgressUpdate};
use ai_code_buddy::core::review::CommitStatus;
use anyhow::Result;
use pretty_assertions::assert_eq;

#[test]
fn test_gpu_backend_display() {
    assert_eq!(format!("{}", GpuBackend::Metal), "Metal");
    assert_eq!(format!("{}", GpuBackend::Cuda), "CUDA");
    assert_eq!(format!("{}", GpuBackend::Mkl), "MKL");
    assert_eq!(format!("{}", GpuBackend::Cpu), "CPU");
}

#[test]
fn test_gpu_backend_equality() {
    assert_eq!(GpuBackend::Metal, GpuBackend::Metal);
    assert_eq!(GpuBackend::Cuda, GpuBackend::Cuda);
    assert_eq!(GpuBackend::Mkl, GpuBackend::Mkl);
    assert_eq!(GpuBackend::Cpu, GpuBackend::Cpu);
    
    assert_ne!(GpuBackend::Metal, GpuBackend::Cuda);
    assert_ne!(GpuBackend::Cpu, GpuBackend::Metal);
}

#[test]
fn test_analysis_request_creation() {
    let request = AnalysisRequest {
        file_path: "test.rs".to_string(),
        content: "fn main() {}".to_string(),
        language: "rust".to_string(),
        commit_status: CommitStatus::Modified,
    };
    
    assert_eq!(request.file_path, "test.rs");
    assert_eq!(request.content, "fn main() {}");
    assert_eq!(request.language, "rust");
    assert!(matches!(request.commit_status, CommitStatus::Modified));
}

#[test]
fn test_progress_update_creation() {
    let update = ProgressUpdate {
        current_file: "src/main.rs".to_string(),
        progress: 0.5,
        stage: "analyzing".to_string(),
    };
    
    assert_eq!(update.current_file, "src/main.rs");
    assert_eq!(update.progress, 0.5);
    assert_eq!(update.stage, "analyzing");
}

#[test]
fn test_gpu_backend_debug_format() {
    let backend = GpuBackend::Metal;
    let debug_str = format!("{:?}", backend);
    assert!(debug_str.contains("Metal"));
}

#[test]
fn test_analysis_request_serialization() -> Result<()> {
    let request = AnalysisRequest {
        file_path: "test.rs".to_string(),
        content: "fn main() {}".to_string(),
        language: "rust".to_string(),
        commit_status: CommitStatus::Modified,
    };
    
    // Test serialization
    let json = serde_json::to_string(&request)?;
    assert!(json.contains("test.rs"));
    assert!(json.contains("rust"));
    
    // Test deserialization
    let deserialized: AnalysisRequest = serde_json::from_str(&json)?;
    assert_eq!(deserialized.file_path, request.file_path);
    assert_eq!(deserialized.content, request.content);
    assert_eq!(deserialized.language, request.language);
    assert!(matches!(deserialized.commit_status, CommitStatus::Modified));
    
    Ok(())
}

#[test]
fn test_analysis_request_with_different_statuses() {
    let statuses = vec![
        CommitStatus::Committed,
        CommitStatus::Staged,
        CommitStatus::Modified,
        CommitStatus::Untracked,
    ];
    
    for status in statuses {
        let request = AnalysisRequest {
            file_path: "test.rs".to_string(),
            content: "fn main() {}".to_string(),
            language: "rust".to_string(),
            commit_status: status.clone(),
        };
        
        // Use pattern matching instead of equality
        match (&request.commit_status, &status) {
            (CommitStatus::Committed, CommitStatus::Committed) => assert!(true),
            (CommitStatus::Staged, CommitStatus::Staged) => assert!(true),
            (CommitStatus::Modified, CommitStatus::Modified) => assert!(true),
            (CommitStatus::Untracked, CommitStatus::Untracked) => assert!(true),
            _ => assert!(false, "Status mismatch"),
        }
    }
}

#[test]
fn test_progress_update_with_different_values() {
    let progress_values = vec![0.0, 0.25, 0.5, 0.75, 1.0];
    
    for progress in progress_values {
        let update = ProgressUpdate {
            current_file: format!("file_{}.rs", (progress * 100.0) as i32),
            progress,
            stage: "analyzing".to_string(),
        };
        
        assert_eq!(update.progress, progress);
        assert!(update.current_file.contains("file_"));
    }
}

#[test]
fn test_analysis_request_with_empty_content() {
    let request = AnalysisRequest {
        file_path: "empty.rs".to_string(),
        content: "".to_string(),
        language: "rust".to_string(),
        commit_status: CommitStatus::Untracked,
    };
    
    assert!(request.content.is_empty());
    assert_eq!(request.file_path, "empty.rs");
}

#[test]
fn test_analysis_request_with_large_content() {
    let large_content = "fn main() {\n".to_string() + &"    println!(\"Hello\");\n".repeat(1000) + "}";
    
    let request = AnalysisRequest {
        file_path: "large.rs".to_string(),
        content: large_content.clone(),
        language: "rust".to_string(),
        commit_status: CommitStatus::Modified,
    };
    
    assert_eq!(request.content, large_content);
    assert!(request.content.len() > 1000);
}

#[test]
fn test_analysis_request_different_languages() {
    let languages = vec!["rust", "python", "javascript", "typescript", "go", "java"];
    
    for language in languages {
        let request = AnalysisRequest {
            file_path: format!("test.{}", language),
            content: "// test content".to_string(),
            language: language.to_string(),
            commit_status: CommitStatus::Untracked,
        };
        
        assert_eq!(request.language, language);
        assert!(request.file_path.contains(language));
    }
}

#[test]
fn test_progress_update_stages() {
    let stages = vec!["initializing", "analyzing", "processing", "finalizing", "complete"];
    
    for stage in stages {
        let update = ProgressUpdate {
            current_file: "test.rs".to_string(),
            progress: 0.5,
            stage: stage.to_string(),
        };
        
        assert_eq!(update.stage, stage);
    }
}

#[test]
fn test_gpu_backend_clone() {
    let backend = GpuBackend::Metal;
    let cloned = backend.clone();
    assert_eq!(backend, cloned);
}

#[test]
fn test_analysis_request_clone() {
    let request = AnalysisRequest {
        file_path: "test.rs".to_string(),
        content: "fn main() {}".to_string(),
        language: "rust".to_string(),
        commit_status: CommitStatus::Modified,
    };
    
    let cloned = request.clone();
    assert_eq!(request.file_path, cloned.file_path);
    assert_eq!(request.content, cloned.content);
    assert_eq!(request.language, cloned.language);
    
    // Use pattern matching for comparison
    match (&request.commit_status, &cloned.commit_status) {
        (CommitStatus::Modified, CommitStatus::Modified) => assert!(true),
        _ => assert!(false, "Status mismatch"),
    }
}

#[test]
fn test_progress_update_clone() {
    let update = ProgressUpdate {
        current_file: "src/main.rs".to_string(),
        progress: 0.75,
        stage: "analyzing".to_string(),
    };
    
    let cloned = update.clone();
    assert_eq!(update.current_file, cloned.current_file);
    assert_eq!(update.progress, cloned.progress);
    assert_eq!(update.stage, cloned.stage);
}
