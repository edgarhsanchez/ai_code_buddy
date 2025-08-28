use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::sync::mpsc;

use crate::core::review::{CommitStatus, Issue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisRequest {
    pub file_path: String,
    pub content: String,
    pub language: String,
    pub commit_status: CommitStatus,
}

#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub current_file: String,
    pub progress: f64,
    pub stage: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpuBackend {
    Metal,
    Cuda,
    Mkl,
    Cpu,
}

impl std::fmt::Display for GpuBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuBackend::Metal => write!(f, "Metal"),
            GpuBackend::Cuda => write!(f, "CUDA"),
            GpuBackend::Mkl => write!(f, "MKL"),
            GpuBackend::Cpu => write!(f, "CPU"),
        }
    }
}

pub struct AIAnalyzer {
    backend: GpuBackend,
}

impl AIAnalyzer {
    pub async fn new(use_gpu: bool) -> Result<Self> {
        println!("🧠 Initializing AI analyzer...");

        // Detect and configure GPU backend
        let backend = if use_gpu {
            Self::detect_gpu_backend()
        } else {
            GpuBackend::Cpu
        };

        println!("🔧 Using backend: {backend:?}");

        println!("🔍 AI inference currently disabled due to token sampling issues");
        println!("🔧 Using enhanced rule-based analysis for comprehensive code review");

        let analyzer = AIAnalyzer { backend };

        // Display the configured backend for diagnostics
        println!(
            "🔧 AI Analyzer initialized with {} backend",
            analyzer.get_backend()
        );

        Ok(analyzer)
    }

    /// Get the GPU backend being used by this analyzer
    pub fn get_backend(&self) -> &GpuBackend {
        &self.backend
    }

    fn detect_gpu_backend() -> GpuBackend {
        // Check if we're on Apple Silicon (Metal support)
        if cfg!(target_os = "macos") && Self::is_apple_silicon() {
            println!("🍎 Apple Silicon detected, using Metal backend");
            GpuBackend::Metal
        }
        // Check for CUDA support (NVIDIA)
        else if Self::has_cuda_support() {
            println!("🟢 NVIDIA CUDA detected, using CUDA backend");
            GpuBackend::Cuda
        }
        // Check for Intel MKL support
        else if Self::has_mkl_support() {
            println!("🔵 Intel MKL detected, using MKL backend");
            GpuBackend::Mkl
        }
        // Fallback to CPU
        else {
            println!("💻 No GPU acceleration detected, falling back to CPU");
            GpuBackend::Cpu
        }
    }

    fn is_apple_silicon() -> bool {
        // Check if we're running on Apple Silicon
        cfg!(target_arch = "aarch64") && cfg!(target_os = "macos")
    }

    fn has_cuda_support() -> bool {
        // Check for NVIDIA GPU presence
        // This is a simplified check - in production you might want to check for actual CUDA runtime
        std::process::Command::new("nvidia-smi")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn has_mkl_support() -> bool {
        // Check for Intel processor
        // This is a simplified check
        cfg!(target_arch = "x86_64")
    }

    pub async fn analyze_file(
        &self,
        request: AnalysisRequest,
        progress_tx: Option<mpsc::UnboundedSender<ProgressUpdate>>,
    ) -> Result<Vec<Issue>> {
        let _language = self.detect_language(&request.file_path);

        if let Some(ref tx) = progress_tx {
            let _ = tx.send(ProgressUpdate {
                current_file: request.file_path.clone(),
                progress: 0.0,
                stage: "Starting analysis".to_string(),
            });
        }

        let mut issues = Vec::new();

        // AI inference is currently disabled due to token sampling issues
        // Using enhanced rule-based analysis which provides comprehensive coverage
        issues.extend(self.rule_based_analysis(&request)?);

        // TODO: Re-enable AI analysis once token sampling issues are resolved
        // The AI methods are preserved below for future use

        if let Some(ref tx) = progress_tx {
            let _ = tx.send(ProgressUpdate {
                current_file: request.file_path.clone(),
                progress: 100.0,
                stage: "Analysis complete".to_string(),
            });
        }

        Ok(issues)
    }

    pub fn rule_based_analysis(&self, request: &AnalysisRequest) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();

        for (line_num, line) in request.content.lines().enumerate() {
            let line_number = line_num + 1;
            let line_lower = line.to_lowercase();

            // SECURITY PATTERNS

            // Hardcoded credentials
            if (line_lower.contains("password")
                || line_lower.contains("api_key")
                || line_lower.contains("secret"))
                && line.contains("=")
                && (line.contains("\"") || line.contains("'"))
            {
                issues.push(Issue {
                    file: request.file_path.clone(),
                    line: line_number,
                    severity: "Critical".to_string(),
                    category: "Security".to_string(),
                    description: "Hardcoded credentials detected - use environment variables"
                        .to_string(),
                    commit_status: request.commit_status.clone(),
                });
            }

            // Code injection
            if line.contains("eval(") || line.contains("exec(") {
                issues.push(Issue {
                    file: request.file_path.clone(),
                    line: line_number,
                    severity: "Critical".to_string(),
                    category: "Security".to_string(),
                    description: "Code injection vulnerability - avoid eval/exec".to_string(),
                    commit_status: request.commit_status.clone(),
                });
            }

            // SQL injection patterns
            if line.contains("query")
                && line.contains("format!")
                && (line.contains("SELECT") || line.contains("INSERT") || line.contains("UPDATE"))
            {
                issues.push(Issue {
                    file: request.file_path.clone(),
                    line: line_number,
                    severity: "Critical".to_string(),
                    category: "Security".to_string(),
                    description: "Potential SQL injection - use parameterized queries".to_string(),
                    commit_status: request.commit_status.clone(),
                });
            }

            // Command injection patterns
            if (line.contains("Command::new")
                || line.contains("subprocess")
                || line.contains("system("))
                && (line.contains("format!")
                    || line.contains("user_input")
                    || line.contains("args"))
            {
                issues.push(Issue {
                    file: request.file_path.clone(),
                    line: line_number,
                    severity: "Critical".to_string(),
                    category: "Security".to_string(),
                    description: "Command injection vulnerability - sanitize inputs".to_string(),
                    commit_status: request.commit_status.clone(),
                });
            }

            // Path traversal patterns
            if line.contains("../")
                && (line.contains("read") || line.contains("open") || line.contains("file"))
            {
                issues.push(Issue {
                    file: request.file_path.clone(),
                    line: line_number,
                    severity: "High".to_string(),
                    category: "Security".to_string(),
                    description: "Path traversal vulnerability - validate file paths".to_string(),
                    commit_status: request.commit_status.clone(),
                });
            }

            // PERFORMANCE PATTERNS

            // Nested loops (O(n²) complexity)
            if line.contains("for") && line.trim().starts_with("for") {
                // Check if there's another for loop nearby (simple heuristic)
                let lines: Vec<&str> = request.content.lines().collect();
                for (idx, _) in lines
                    .iter()
                    .enumerate()
                    .take(std::cmp::min(line_num + 10, lines.len()))
                    .skip(line_num + 1)
                {
                    if lines[idx].trim().starts_with("for") {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "Medium".to_string(),
                            category: "Performance".to_string(),
                            description: "Nested loops detected - consider optimization"
                                .to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                        break;
                    }
                }
            }

            // Language-specific analysis
            match request.language.as_str() {
                "rust" => {
                    // Security
                    if line.contains("unsafe") {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "High".to_string(),
                            category: "Security".to_string(),
                            description: "Unsafe code block - requires justification and review"
                                .to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                    }

                    if line.contains("std::ptr::null") {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "Critical".to_string(),
                            category: "Security".to_string(),
                            description: "Null pointer dereference - will cause segfault"
                                .to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                    }

                    // Error handling
                    if line.contains("unwrap()") && !line.contains("expect(") {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "Medium".to_string(),
                            category: "Error Handling".to_string(),
                            description:
                                "Use expect() or proper error handling instead of unwrap()"
                                    .to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                    }

                    // Performance
                    if line.contains(".clone()") && line.contains("&") {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "Low".to_string(),
                            category: "Performance".to_string(),
                            description: "Unnecessary clone - consider borrowing instead"
                                .to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                    }
                }
                "python" => {
                    // Security
                    if line.contains("pickle.loads") && !line.contains("trusted") {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "Critical".to_string(),
                            category: "Security".to_string(),
                            description: "Unsafe deserialization - pickle.loads is dangerous"
                                .to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                    }

                    if line.contains("yaml.load") && !line.contains("safe_load") {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "High".to_string(),
                            category: "Security".to_string(),
                            description: "Use yaml.safe_load instead of yaml.load".to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                    }

                    // Performance
                    if line.contains("+=") && (line.contains("\"") || line.contains("'")) {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "Medium".to_string(),
                            category: "Performance".to_string(),
                            description:
                                "String concatenation in loop - use join() for better performance"
                                    .to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                    }
                }
                "javascript" | "typescript" => {
                    // Security
                    if line.contains("innerHTML") && line.contains("+") {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "High".to_string(),
                            category: "Security".to_string(),
                            description: "XSS vulnerability - validate before setting innerHTML"
                                .to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                    }

                    // Performance
                    if line.contains("document.getElementById") && line.contains("for") {
                        issues.push(Issue {
                            file: request.file_path.clone(),
                            line: line_number,
                            severity: "Medium".to_string(),
                            category: "Performance".to_string(),
                            description: "DOM query in loop - cache the element reference"
                                .to_string(),
                            commit_status: request.commit_status.clone(),
                        });
                    }
                }
                _ => {}
            }

            // CODE QUALITY PATTERNS

            if line.contains("TODO") || line.contains("FIXME") || line.contains("HACK") {
                issues.push(Issue {
                    file: request.file_path.clone(),
                    line: line_number,
                    severity: "Low".to_string(),
                    category: "Code Quality".to_string(),
                    description: "Code comment indicates incomplete implementation".to_string(),
                    commit_status: request.commit_status.clone(),
                });
            }

            // Long line detection
            if line.len() > 120 {
                issues.push(Issue {
                    file: request.file_path.clone(),
                    line: line_number,
                    severity: "Low".to_string(),
                    category: "Code Quality".to_string(),
                    description: format!(
                        "Line too long ({} chars) - consider breaking into multiple lines",
                        line.len()
                    ),
                    commit_status: request.commit_status.clone(),
                });
            }
        }

        Ok(issues)
    }

    fn detect_language(&self, file_path: &str) -> String {
        let path = Path::new(file_path);
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("rs") => "rust".to_string(),
            Some("js") => "javascript".to_string(),
            Some("ts") => "typescript".to_string(),
            Some("py") => "python".to_string(),
            Some("java") => "java".to_string(),
            Some("cpp") | Some("cc") | Some("cxx") => "cpp".to_string(),
            Some("c") => "c".to_string(),
            Some("go") => "go".to_string(),
            Some("php") => "php".to_string(),
            Some("rb") => "ruby".to_string(),
            Some("cs") => "csharp".to_string(),
            _ => "unknown".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::review::CommitStatus;

    fn make_request(file: &str, content: &str, language: &str) -> AnalysisRequest {
        AnalysisRequest {
            file_path: file.to_string(),
            content: content.to_string(),
            language: language.to_string(),
            commit_status: CommitStatus::Modified,
        }
    }

    #[test]
    fn test_detect_language_variants() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
        };
        assert_eq!(analyzer.detect_language("src/main.rs"), "rust");
        assert_eq!(analyzer.detect_language("a/b/c.py"), "python");
        assert_eq!(analyzer.detect_language("index.ts"), "typescript");
        assert_eq!(analyzer.detect_language("script.js"), "javascript");
        assert_eq!(analyzer.detect_language("unknown.foo"), "unknown");
    }

    #[test]
    fn test_rule_based_analysis_rust_patterns() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
        };
        let content = r#"
            // SECURITY
            let password = "secret";
            let _ = eval("2+2");
            let query = format!("SELECT * FROM users");
            std::process::Command::new("sh").arg(format!("{}", user_input));
            let _ = std::fs::read("../etc/passwd");
            // PERFORMANCE
            for i in 0..10 {
                for j in 0..10 {}
            }
            // RUST SPECIFIC
            unsafe { /* do unsafe things */ }
            let p = std::ptr::null();
            let _ = something.unwrap();
            let _y = &x.clone();
            // QUALITY
            // TODO: fix
            // Long line next
            aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        "#;
        let req = make_request("file.rs", content, "rust");
        let issues = analyzer.rule_based_analysis(&req).unwrap();
        assert!(!issues.is_empty());
        // Ensure we hit multiple categories
        assert!(issues.iter().any(|i| i.category == "Security"));
        assert!(issues.iter().any(|i| i.category == "Performance"));
        assert!(issues.iter().any(|i| i.category == "Code Quality"));
    }

    #[test]
    fn test_rule_based_analysis_python_patterns() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
        };
        let content = r#"
            import pickle
            data = pickle.loads(b"...")
            import yaml
            result = yaml.load("x: 1")
            s = "";
            for i in range(10): s += "x"
        "#;
        let req = make_request("script.py", content, "python");
        let issues = analyzer.rule_based_analysis(&req).unwrap();
        assert!(issues.iter().any(|i| i.category == "Security"));
        assert!(issues.iter().any(|i| i.category == "Performance"));
    }

    #[test]
    fn test_rule_based_analysis_js_patterns() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
        };
        let content = r#"
            let x = "user";
            element.innerHTML = "<div>" + x;
            for (let i = 0; i < 10; i++) { document.getElementById("id"); }
        "#;
        let req = make_request("script.js", content, "javascript");
        let issues = analyzer.rule_based_analysis(&req).unwrap();
        assert!(issues.iter().any(|i| i.category == "Security"));
        assert!(issues.iter().any(|i| i.category == "Performance"));
    }

    #[test]
    fn test_analyze_file_emits_progress_and_issues() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let analyzer = AIAnalyzer::new(false).await.unwrap();
            let (tx, mut rx) = mpsc::unbounded_channel::<ProgressUpdate>();
            let req = make_request("file.rs", "let password = \"x\";", "rust");
            let issues = analyzer.analyze_file(req, Some(tx)).await.unwrap();
            assert!(!issues.is_empty());
            // Try receive up to a couple of progress messages (non-blocking)
            let mut got_any = false;
            for _ in 0..4 {
                if rx.try_recv().is_ok() {
                    got_any = true;
                    break;
                }
            }
            assert!(got_any, "expected at least one progress message");
        });
    }
}
