use anyhow::Result;
use kalosm::language::{Llama, ChatModelExt, TextStream};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::sync::mpsc;
use futures::StreamExt;

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
    enable_ai: bool,
    model_initialized: bool,
    model: Option<Llama>, // Store the model instance for reuse
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

        println!("🤖 AI inference enabled - creating Kalosm model...");
        // Create and store the model instance for reuse across all analyses
        let (model_initialized, model) = match Llama::new_chat().await {
            Ok(model) => {
                println!("✅ Kalosm Llama model created successfully");
                (true, Some(model))
            }
            Err(e) => {
                println!("⚠️  Failed to create Kalosm model: {}", e);
                println!("   Continuing with basic analysis");
                (false, None)
            }
        };

        let analyzer = AIAnalyzer { 
            backend, 
            enable_ai: model_initialized,
            model_initialized,
            model,
        };

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
        // If specific GPU features are enabled, use them in order of preference
        #[cfg(feature = "gpu-metal")]
        {
            println!("🍎 Metal backend available (compiled)");
            return GpuBackend::Metal;
        }
        
        #[cfg(all(feature = "gpu-cuda", not(feature = "gpu-metal")))]
        {
            println!("🟢 CUDA backend available (compiled)");
            return GpuBackend::Cuda;
        }
        
        #[cfg(all(feature = "gpu-mkl", not(feature = "gpu-metal"), not(feature = "gpu-cuda")))]
        {
            println!("🔵 MKL backend available (compiled)");
            return GpuBackend::Mkl;
        }
        
        // If auto-gpu is enabled, provide recommendations based on hardware detection
        #[cfg(all(feature = "auto-gpu", not(any(feature = "gpu-metal", feature = "gpu-cuda", feature = "gpu-mkl"))))]
        {
            println!("� Auto-GPU enabled - detecting optimal GPU backend...");
            
            // Provide platform-specific recommendations
            #[cfg(all(target_os = "macos", metal_gpu_available))]
            {
                println!("💡 Metal GPU detected! Enable with: cargo build --features gpu-metal");
                return GpuBackend::Cpu;
            }
            
            #[cfg(all(not(target_os = "macos"), nvidia_gpu_available))]
            {
                println!("💡 NVIDIA GPU detected! Enable with: cargo build --features gpu-cuda");
                return GpuBackend::Cpu;
            }
            
            #[cfg(intel_mkl_available)]
            {
                println!("💡 Intel MKL detected! Enable with: cargo build --features gpu-mkl");
                return GpuBackend::Cpu;
            }
            
            println!("💻 No GPU acceleration detected - using CPU");
        }
        
        #[cfg(not(any(feature = "gpu-metal", feature = "gpu-cuda", feature = "gpu-mkl", feature = "auto-gpu")))]
        {
            println!("💻 No GPU backends compiled - using CPU");
        }
        
        // Fallback to CPU
        GpuBackend::Cpu
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

        // Use AI-powered analysis with Kalosm (create model per analysis for thread safety)
        if self.enable_ai && self.model_initialized {
            println!("🤖 Running AI-powered analysis with Kalosm...");
            issues.extend(self.ai_analysis(&request).await?);
        } else {
            println!("⚠️ AI analysis not available - no model loaded");
            // Return empty issues if no AI model is available
        }

        if let Some(ref tx) = progress_tx {
            let _ = tx.send(ProgressUpdate {
                current_file: request.file_path.clone(),
                progress: 100.0,
                stage: "Analysis complete".to_string(),
            });
        }

        Ok(issues)
    }

    /// Helper function to extract code snippet with context
    fn get_code_snippet_with_context(
        &self,
        content: &str,
        target_line: u32,
        context_lines: usize,
    ) -> (String, Option<Vec<String>>) {
        let lines: Vec<&str> = content.lines().collect();
        let target_idx = target_line.saturating_sub(1) as usize; // Convert to 0-based index
        
        // Clamp target_idx to be within valid range
        let target_idx = std::cmp::min(target_idx, lines.len().saturating_sub(1));
        
        // Get the main offending line
        let code_snippet = lines.get(target_idx)
            .map(|&line| line.to_string())
            .unwrap_or_else(|| "Line not found".to_string());
        
        // Get context lines if requested
        let context = if context_lines > 0 {
            let start_idx = target_idx.saturating_sub(context_lines);
            let end_idx = std::cmp::min(target_idx + context_lines + 1, lines.len());
            
            // Ensure start_idx <= end_idx
            let start_idx = std::cmp::min(start_idx, end_idx);
            
            let mut context_vec = Vec::new();
            for (idx, &line) in lines[start_idx..end_idx].iter().enumerate() {
                let actual_line_num = start_idx + idx + 1;
                let marker = if actual_line_num == target_line as usize { ">>> " } else { "    " };
                context_vec.push(format!("{}{:3}: {}", marker, actual_line_num, line));
            }
            Some(context_vec)
        } else {
            None
        };
        
        (code_snippet, context)
    }

    /// Helper function to create an issue with code snippet
    fn create_issue_with_snippet(
        &self,
        request: &AnalysisRequest,
        line_number: u32,
        severity: &str,
        category: &str,
        description: &str,
    ) -> Issue {
        let (code_snippet, context_lines) = self.get_code_snippet_with_context(&request.content, line_number, 2);
        
        Issue {
            file: request.file_path.clone(),
            line: line_number,
            severity: severity.to_string(),
            category: category.to_string(),
            description: description.to_string(),
            commit_status: request.commit_status.clone(),
            code_snippet,
            context_lines,
        }
    }

    /// AI-powered analysis using Kalosm for comprehensive code review
    pub async fn ai_analysis(&self, request: &AnalysisRequest) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        
        // Use the stored model instance instead of creating a new one
        let model = match &self.model {
            Some(model) => {
                println!("🤖 Using shared AI model for analysis...");
                model
            }
            None => {
                println!("⚠️  No AI model available for analysis");
                return Ok(issues);
            }
        };
        
        // Analyze code in smaller chunks to reduce memory pressure and avoid token limits
        let lines: Vec<&str> = request.content.lines().collect();
        let chunk_size = 30; // Reduced from 50 to 30 lines for better stability
        
        for (chunk_start, chunk) in lines.chunks(chunk_size).enumerate() {
            let chunk_start_line = chunk_start * chunk_size + 1;
            let chunk_content = chunk.join("\n");
            
            // Skip empty chunks or very small chunks
            if chunk_content.trim().is_empty() || chunk_content.trim().lines().count() < 3 {
                continue;
            }
            
            // Add a small delay between chunks to prevent overwhelming the model
            if chunk_start > 0 {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            }
            
            match self.analyze_code_chunk(
                &model,
                &chunk_content,
                &request.language,
                &request.file_path,
                chunk_start_line,
            ).await {
                Ok(analysis_results) => {
                    issues.extend(analysis_results.into_iter().map(|mut issue| {
                        issue.commit_status = request.commit_status.clone();
                        issue
                    }));
                }
                Err(e) => {
                    println!("⚠️  Skipping chunk {} due to analysis error: {}", chunk_start + 1, e);
                    // Continue with next chunk instead of failing entire analysis
                }
            }
        }
        
        Ok(issues)
    }

    /// Analyze a chunk of code using structured generation with clear formatting
    async fn analyze_code_chunk_structured(
        &self,
        model: &Llama,
        code_chunk: &str,
        language: &str,
        file_path: &str,
        start_line: usize,
    ) -> Result<Vec<Issue>> {
        let analysis_prompt = format!(r#"Analyze this {} code for issues. Return findings in this exact structured format:

Code to analyze:
{}

For each issue found, use this exact format:
ISSUE_START
Line: [number]
Severity: [Critical|High|Medium|Low]
Category: [Security|Performance|Code Quality|Best Practices]
Description: [brief description of the issue]
Code: [the problematic code snippet]
Suggestion: [improved code suggestion]
Explanation: [why this improvement is better]
ISSUE_END

If no issues found, respond with: NO_ISSUES_FOUND

Separate multiple issues with ISSUE_START/ISSUE_END blocks."#, language, code_chunk);

        println!("🤖 Running structured analysis with clear formatting...");

        let task = model.task("You are a code review expert. Analyze code and return structured findings using the exact format specified.");

        let stream = task.run(&analysis_prompt);

        // Stream the response and get the complete text
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            self.time_stream_silent(stream)
        ).await
        .map_err(|_| anyhow::anyhow!("AI analysis timeout after 60 seconds"))?
        .map_err(|e| anyhow::anyhow!("AI streaming error: {}", e))?;

        println!("✅ Got structured response: {} chars", response.len());

        // Parse the structured response
        self.parse_structured_response(&response, file_path, start_line, code_chunk)
    }

    /// Parse structured response with clear ISSUE_START/ISSUE_END markers
    fn parse_structured_response(
        &self,
        response: &str,
        file_path: &str,
        start_line: usize,
        code_chunk: &str,
    ) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();

        // Check for no issues response
        if response.trim().contains("NO_ISSUES_FOUND") {
            println!("ℹ️  No issues found in this code chunk");
            return Ok(issues);
        }

        // Split response into issue blocks
        let issue_blocks: Vec<&str> = response
            .split("ISSUE_START")
            .skip(1) // Skip content before first ISSUE_START
            .filter_map(|block| {
                block.split("ISSUE_END").next() // Take content before ISSUE_END
            })
            .collect();

        println!("📋 Found {} issue blocks to parse", issue_blocks.len());

        for (i, block) in issue_blocks.iter().enumerate() {
            if let Some(issue) = self.parse_issue_block(block, file_path, start_line, code_chunk) {
                let severity = issue.severity.clone();
                let category = issue.category.clone();
                issues.push(issue);
                println!("✅ Parsed issue {}: {} - {}", i + 1, severity, category);
            } else {
                println!("⚠️  Failed to parse issue block {}", i + 1);
            }
        }

        Ok(issues)
    }

    /// Parse a single issue block
    fn parse_issue_block(
        &self,
        block: &str,
        file_path: &str,
        start_line: usize,
        code_chunk: &str,
    ) -> Option<Issue> {
        let mut line_num = 1u32;
        let mut severity = "Medium".to_string();
        let mut category = "Code Quality".to_string();
        let mut description = "Code quality issue detected".to_string();
        let mut code_snippet = "".to_string();
        let mut suggestion = "".to_string();
        let mut explanation = "".to_string();

        for line in block.lines() {
            let line = line.trim();
            if line.starts_with("Line:") {
                if let Some(num_str) = line.strip_prefix("Line:").map(str::trim) {
                    line_num = num_str.parse().unwrap_or(1);
                }
            } else if line.starts_with("Severity:") {
                if let Some(sev) = line.strip_prefix("Severity:").map(str::trim) {
                    severity = sev.to_string();
                }
            } else if line.starts_with("Category:") {
                if let Some(cat) = line.strip_prefix("Category:").map(str::trim) {
                    category = cat.to_string();
                }
            } else if line.starts_with("Description:") {
                if let Some(desc) = line.strip_prefix("Description:").map(str::trim) {
                    description = desc.to_string();
                }
            } else if line.starts_with("Code:") {
                if let Some(code) = line.strip_prefix("Code:").map(str::trim) {
                    code_snippet = code.to_string();
                }
            } else if line.starts_with("Suggestion:") {
                if let Some(sugg) = line.strip_prefix("Suggestion:").map(str::trim) {
                    suggestion = sugg.to_string();
                }
            } else if line.starts_with("Explanation:") {
                if let Some(exp) = line.strip_prefix("Explanation:").map(str::trim) {
                    explanation = exp.to_string();
                }
            }
        }

        // Skip issues with empty descriptions
        if description.trim().is_empty() {
            return None;
        }

        // Adjust line number to absolute position in file
        let absolute_line = start_line as u32 + line_num.saturating_sub(1);

        // Get code snippet with context if not provided
        let actual_code_snippet = if code_snippet.is_empty() {
            let (snippet, _) = self.get_code_snippet_with_context(code_chunk, line_num, 2);
            snippet
        } else {
            code_snippet
        };

        // Enhanced description with improvement suggestions
        let enhanced_description = if !suggestion.is_empty() && !explanation.is_empty() {
            format!("{}\n\n**Suggested Improvement:**\n```\n{}\n```\n\n**Explanation:** {}",
                    description, suggestion, explanation)
        } else {
            description
        };

        Some(Issue {
            file: file_path.to_string(),
            line: absolute_line,
            severity,
            category,
            description: enhanced_description,
            commit_status: CommitStatus::Modified, // Will be overridden
            code_snippet: actual_code_snippet,
            context_lines: None,
        })
    }

    /// Analyze a chunk of code using AI with comprehensive prompts
    async fn analyze_code_chunk(
        &self,
        model: &Llama,
        code_chunk: &str,
        language: &str,
        file_path: &str,
        start_line: usize,
    ) -> Result<Vec<Issue>> {
        // Try structured analysis first, fall back to JSON parsing if it fails
        match self.analyze_code_chunk_structured(model, code_chunk, language, file_path, start_line).await {
            Ok(issues) => Ok(issues),
            Err(e) => {
                println!("⚠️  Structured analysis failed ({}), trying JSON parsing approach...", e);
                // Fallback to the original approach
                match self.try_ai_analysis(model, code_chunk, language, file_path, start_line, false).await {
                    Ok(issues) => Ok(issues),
                    Err(e2) => {
                        println!("⚠️  JSON parsing also failed ({}), trying simplified analysis...", e2);
                        match self.try_ai_analysis(model, code_chunk, language, file_path, start_line, true).await {
                            Ok(issues) => Ok(issues),
                            Err(e3) => {
                                println!("⚠️  All analysis methods failed ({}), skipping AI analysis for this chunk", e3);
                                Ok(Vec::new()) // Return empty instead of failing
                            }
                        }
                    }
                }
            }
        }
    }

    /// Try AI analysis with optional simplified prompt
    async fn try_ai_analysis(
        &self,
        model: &Llama,
        code_chunk: &str,
        language: &str,
        file_path: &str,
        start_line: usize,
        simplified: bool,
    ) -> Result<Vec<Issue>> {
        let analysis_prompt = if simplified {
            self.create_simple_analysis_prompt(code_chunk, language)
        } else {
            self.create_comprehensive_analysis_prompt(code_chunk, language)
        };
        
        println!("🤖 Sending prompt to model (simplified={})", simplified);
        
        // Create a task for AI analysis
        let task = model.task("You are a code review expert. Analyze code and return findings in valid JSON format only.");
        let stream = task.run(&analysis_prompt);
        
        // Stream the response and get the complete text
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            self.time_stream(stream)
        ).await
        .map_err(|_| anyhow::anyhow!("AI analysis timeout after 60 seconds"))?
        .map_err(|e| anyhow::anyhow!("AI streaming error: {}", e))?;
        
        println!("✅ Got response from model: {} chars", response.len());
        
        // Check response quality
        if response.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty response from AI model"));
        }
        
        if response.contains("No token sampled") {
            return Err(anyhow::anyhow!("AI model token sampling failed"));
        }
        
        // Parse the AI response to extract issues
        self.parse_ai_response(&response, file_path, start_line, code_chunk)
    }

    /// Create a comprehensive analysis prompt covering OWASP and best practices
    fn create_comprehensive_analysis_prompt(&self, code: &str, language: &str) -> String {
        format!(r#"Perform a comprehensive code review of this {} code. Check for OWASP security vulnerabilities, performance issues, and coding best practices. Include specific code examples.

Return your analysis as a JSON object with this exact structure:
{{
    "issues": [
        {{
            "line": 1,
            "severity": "High|Medium|Low",
            "category": "Security|Performance|Code Quality|Best Practices",
            "description": "Brief description of the issue",
            "code_snippet": "The problematic code snippet",
            "improvement_example": "Suggested improved code",
            "explanation": "Why this improvement is better"
        }}
    ]
}}

Code to analyze:
{}"#, language, code)
    }

    /// Create a simplified analysis prompt for fallback when comprehensive analysis fails
    fn create_simple_analysis_prompt(&self, code: &str, language: &str) -> String {
        format!(r#"Do a code review of this {} code including OWASP and other security concerns.

Return your analysis as a JSON object with this exact structure:
{{
    "issues": [
        {{
            "line": 1,
            "severity": "High|Medium|Low",
            "category": "Security|Performance|Code Quality|Best Practices",
            "description": "Brief description of the issue",
            "code_snippet": "The problematic code snippet",
            "improvement_example": "Suggested improved code",
            "explanation": "Why this improvement is better"
        }}
    ]
}}

Code to analyze:
{}"#, language, code)
    }

    /// Create a mock AI response for testing the analysis pipeline
    fn create_mock_ai_response(&self, _code_chunk: &str, language: &str, _file_path: &str) -> String {
        format!(r#"{{
            "issues": [
                {{
                    "category": "Security",
                    "description": "Potential buffer overflow vulnerability detected in {} code",
                    "severity": "High",
                    "line_number": 42,
                    "suggestion": "Consider using bounds checking or safe string functions to prevent buffer overflows",
                    "improvement_example": "Replace strcpy() with strncpy() or use std::string for safer memory management"
                }},
                {{
                    "category": "Code Quality", 
                    "description": "Function complexity is high, consider refactoring",
                    "severity": "Medium",
                    "line_number": 15,
                    "suggestion": "Break down large functions into smaller, more focused functions following the Single Responsibility Principle",
                    "improvement_example": "Extract validation logic into separate validate_input() function and error handling into handle_error() function"
                }},
                {{
                    "category": "OWASP",
                    "description": "Input validation missing for user data",
                    "severity": "High", 
                    "line_number": 8,
                    "suggestion": "Implement proper input validation and sanitization to prevent injection attacks",
                    "improvement_example": "Use parameterized queries for database operations and escape special characters in user input"
                }}
            ]
        }}"#, language)
    }

    /// Parse AI response and convert to Issue structs
    fn parse_ai_response(
        &self,
        response: &str,
        file_path: &str,
        start_line: usize,
        code_chunk: &str,
    ) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        
        // Clean up response text
        let cleaned_response = response.trim();
        
        // Check for error indicators
        if cleaned_response.is_empty() {
            println!("⚠️  Empty response from AI model");
            return Ok(issues);
        }
        
        if cleaned_response.contains("No token sampled") || cleaned_response.len() < 10 {
            println!("⚠️  Invalid or too short response from AI model");
            return Ok(issues);
        }
        
        // Try to extract JSON from response (handle markdown code blocks and extra text)
        let json_content = if let Some(json_start) = cleaned_response.find('{') {
            if let Some(json_end) = cleaned_response.rfind('}') {
                let extracted = &cleaned_response[json_start..=json_end];
                println!("📄 Extracted JSON content: {} chars", extracted.len());
                extracted
            } else {
                println!("⚠️  Found JSON start but no end brace");
                cleaned_response
            }
        } else {
            println!("⚠️  No JSON object found in response, trying raw response");
            cleaned_response
        };
        
        // Try to parse JSON response
        match serde_json::from_str::<serde_json::Value>(json_content) {
            Ok(parsed) => {
                println!("✅ Successfully parsed JSON response");
                if let Some(issues_array) = parsed.get("issues").and_then(|v| v.as_array()) {
                    println!("📋 Found {} issues in response", issues_array.len());
                    for issue_value in issues_array {
                        if let Some(issue) = self.parse_single_issue(issue_value, file_path, start_line, code_chunk) {
                            issues.push(issue);
                        }
                    }
                } else {
                    println!("⚠️  AI response missing 'issues' array");
                    println!("🔍 Available keys: {:?}", parsed.as_object().map(|o| o.keys().collect::<Vec<_>>()));
                }
            }
            Err(e) => {
                println!("⚠️  Failed to parse JSON response ({}), trying text fallback", e);
                println!("🔍 First 200 chars of response: {}", &json_content.chars().take(200).collect::<String>());
                // Fallback: parse non-JSON response
                issues.extend(self.parse_text_response(response, file_path, start_line, code_chunk));
            }
        }
        
        Ok(issues)
    }

    /// Parse a single issue from JSON
    fn parse_single_issue(
        &self,
        issue_value: &serde_json::Value,
        file_path: &str,
        start_line: usize,
        code_chunk: &str,
    ) -> Option<Issue> {
        // Get required fields with fallbacks
        let line = issue_value.get("line")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as u32;
            
        let severity = issue_value.get("severity")
            .and_then(|v| v.as_str())
            .unwrap_or("Medium")
            .to_string();
            
        let category = issue_value.get("category")
            .and_then(|v| v.as_str())
            .unwrap_or("Code Quality")
            .to_string();
            
        let description = issue_value.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("Code quality issue detected")
            .to_string();
            
        // Skip issues with empty descriptions
        if description.trim().is_empty() {
            return None;
        }
            
        let code_snippet = issue_value.get("code_snippet")
            .and_then(|v| v.as_str())
            .unwrap_or("(code snippet not provided)")
            .to_string();
            
        let improvement_example = issue_value.get("improvement_example")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let explanation = issue_value.get("explanation")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Adjust line number to absolute position in file
        let absolute_line = start_line as u32 + line.saturating_sub(1);
        
        // Get code snippet with context
        let (actual_code_snippet, context_lines) = self.get_code_snippet_with_context(code_chunk, line, 2);
        
        // Enhanced description with improvement suggestions
        let enhanced_description = if !improvement_example.is_empty() && !explanation.is_empty() {
            format!("{}\n\n**Suggested Improvement:**\n```\n{}\n```\n\n**Explanation:** {}", 
                    description, improvement_example, explanation)
        } else {
            description
        };

        Some(Issue {
            file: file_path.to_string(),
            line: absolute_line,
            severity,
            category,
            description: enhanced_description,
            commit_status: crate::core::review::CommitStatus::Modified, // Will be overridden
            code_snippet: if code_snippet.is_empty() { actual_code_snippet } else { code_snippet },
            context_lines,
        })
    }

    /// Fallback parser for non-JSON responses
    fn parse_text_response(
        &self,
        response: &str,
        file_path: &str,
        start_line: usize,
        code_chunk: &str,
    ) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // Try to extract meaningful issues from text response
        let lower_response = response.to_lowercase();
        
        // Look for security-related issues
        if lower_response.contains("security") || lower_response.contains("vulnerability") 
            || lower_response.contains("password") || lower_response.contains("injection") {
            let (code_snippet, context_lines) = self.get_code_snippet_with_context(code_chunk, 1u32, 2);
            
            // Extract the first line that mentions security as description
            let description = response.lines()
                .find(|line| {
                    let lower_line = line.to_lowercase();
                    lower_line.contains("security") || lower_line.contains("vulnerability") 
                        || lower_line.contains("password") || lower_line.contains("injection")
                })
                .map(|line| line.trim())
                .unwrap_or("Security concern detected by AI analysis")
                .to_string();
            
            issues.push(Issue {
                file: file_path.to_string(),
                line: start_line as u32,
                severity: "High".to_string(),
                category: "Security".to_string(),
                description,
                commit_status: crate::core::review::CommitStatus::Modified,
                code_snippet,
                context_lines,
            });
        }
        
        // Look for performance issues
        if lower_response.contains("performance") || lower_response.contains("inefficient") 
            || lower_response.contains("optimization") || lower_response.contains("slow") {
            let (code_snippet, context_lines) = self.get_code_snippet_with_context(code_chunk, 1u32, 2);
            
            let description = response.lines()
                .find(|line| {
                    let lower_line = line.to_lowercase();
                    lower_line.contains("performance") || lower_line.contains("inefficient") 
                        || lower_line.contains("optimization") || lower_line.contains("slow")
                })
                .map(|line| line.trim())
                .unwrap_or("Performance issue detected by AI analysis")
                .to_string();
            
            issues.push(Issue {
                file: file_path.to_string(),
                line: start_line as u32,
                severity: "Medium".to_string(),
                category: "Performance".to_string(),
                description,
                commit_status: crate::core::review::CommitStatus::Modified,
                code_snippet,
                context_lines,
            });
        }
        
        // Look for error handling issues
        if lower_response.contains("error") || lower_response.contains("unwrap") 
            || lower_response.contains("panic") || lower_response.contains("exception") {
            let (code_snippet, context_lines) = self.get_code_snippet_with_context(code_chunk, 1u32, 2);
            
            let description = response.lines()
                .find(|line| {
                    let lower_line = line.to_lowercase();
                    lower_line.contains("error") || lower_line.contains("unwrap") 
                        || lower_line.contains("panic") || lower_line.contains("exception")
                })
                .map(|line| line.trim())
                .unwrap_or("Error handling issue detected by AI analysis")
                .to_string();
            
            issues.push(Issue {
                file: file_path.to_string(),
                line: start_line as u32,
                severity: "Medium".to_string(),
                category: "Error Handling".to_string(),
                description,
                commit_status: crate::core::review::CommitStatus::Modified,
                code_snippet,
                context_lines,
            });
        }
        
        // If we found no specific issues but got a response, create a generic issue
        if issues.is_empty() && response.len() > 50 {
            let (code_snippet, context_lines) = self.get_code_snippet_with_context(code_chunk, 1u32, 2);
            
            // Take the first meaningful line as description
            let description = response.lines()
                .find(|line| line.trim().len() > 10 && !line.starts_with('{') && !line.starts_with('}'))
                .map(|line| line.trim())
                .unwrap_or("Code quality issue detected by AI analysis")
                .to_string();
            
            issues.push(Issue {
                file: file_path.to_string(),
                line: start_line as u32,
                severity: "Low".to_string(),
                category: "Code Quality".to_string(),
                description,
                commit_status: crate::core::review::CommitStatus::Modified,
                code_snippet,
                context_lines,
            });
        }
        
        issues
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

    /// Stream and time the AI response, returning the complete text
    async fn time_stream(&self, mut stream: impl TextStream + Unpin) -> Result<String> {
        let start_time = std::time::Instant::now();
        let mut tokens = 0;
        let mut result = String::new();
        
        while let Some(token) = stream.next().await {
            tokens += 1;
            result.push_str(&token);
            // Removed real-time printing to avoid output during processing
        }
        
        let elapsed = start_time.elapsed();
        println!("\n\nGenerated {tokens} tokens ({result_len} characters) in {elapsed:?}", 
                result_len = result.len());
        println!(
            "Tokens per second: {:.2}",
            tokens as f64 / elapsed.as_secs_f64()
        );
        
        Ok(result)
    }

    /// Stream tokens silently without any output for structured analysis
    async fn time_stream_silent(&self, mut stream: impl TextStream + Unpin) -> Result<String> {
        let mut result = String::new();
        
        while let Some(token) = stream.next().await {
            result.push_str(&token);
        }
        
        Ok(result)
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
            enable_ai: false,
            model_initialized: false,
        };
        assert_eq!(analyzer.detect_language("src/main.rs"), "rust");
        assert_eq!(analyzer.detect_language("a/b/c.py"), "python");
        assert_eq!(analyzer.detect_language("index.ts"), "typescript");
        assert_eq!(analyzer.detect_language("script.js"), "javascript");
        assert_eq!(analyzer.detect_language("unknown.foo"), "unknown");
    }

    #[test]
    fn test_ai_analyzer_initialization() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
            enable_ai: true,
            model_initialized: true,
        };
        
        // Test that analyzer is properly configured
        assert!(analyzer.enable_ai);
        assert!(analyzer.model_initialized);
        assert_eq!(analyzer.backend, GpuBackend::Cpu);
    }

    #[test]
    fn test_create_comprehensive_analysis_prompt() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
            enable_ai: true,
            model_initialized: true,
        };
        
        let prompt = analyzer.create_comprehensive_analysis_prompt(
            "let password = \"secret\";",
            "rust"
        );
        
        assert!(prompt.contains("OWASP"));
        assert!(prompt.contains("security"));
        assert!(prompt.contains("rust"));
        assert!(prompt.contains("let password = \"secret\""));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_parse_ai_response_json() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
            enable_ai: true,
            model_initialized: true,
        };
        
        let json_response = r#"
        {
            "issues": [
                {
                    "line": 1,
                    "severity": "Critical",
                    "category": "Security",
                    "description": "Hardcoded credentials detected",
                    "code_snippet": "let password = \"secret\";",
                    "improvement_example": "let password = env::var(\"PASSWORD\")?;",
                    "explanation": "Use environment variables for sensitive data"
                }
            ]
        }
        "#;
        
        let code = "let password = \"secret\";";
        let issues = analyzer.parse_ai_response(json_response, "test.rs", 1, code).unwrap();
        
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].severity, "Critical");
        assert_eq!(issues[0].category, "Security");
        assert!(issues[0].description.contains("Hardcoded credentials"));
        assert!(issues[0].description.contains("detected"));
    }

    #[test]
    fn test_parse_ai_response_fallback() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
            enable_ai: true,
            model_initialized: true,
        };
        
        let text_response = "This code has a security vulnerability in the password handling.";
        let code = "let password = \"secret\";";
        let issues = analyzer.parse_ai_response(text_response, "test.rs", 1, code).unwrap();
        
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].category, "Security");
        assert!(issues[0].description.contains("security concern"));
    }

    #[test]
    fn test_get_code_snippet_with_context() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
            enable_ai: true,
            model_initialized: true,
        };
        
        let code = "line1\nline2\nline3\nline4\nline5";
        let (snippet, context) = analyzer.get_code_snippet_with_context(code, 3u32, 1);
        
        assert_eq!(snippet, "line3");
        assert!(context.is_some());
        let context_lines = context.unwrap();
        assert!(context_lines.len() > 0);
        assert!(context_lines.iter().any(|line| line.contains(">>> ")));
    }

    #[tokio::test]
    async fn test_analyze_file_without_ai() {
        let analyzer = AIAnalyzer {
            backend: GpuBackend::Cpu,
            enable_ai: false,
            model_initialized: false,
        };
        
        let req = make_request("file.rs", "let password = \"secret\";", "rust");
        let issues = analyzer.analyze_file(req, None).await.unwrap();
        assert!(issues.is_empty()); // Should be empty without AI
    }
}
