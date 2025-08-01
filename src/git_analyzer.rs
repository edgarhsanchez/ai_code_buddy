use crate::review::{CodeMetrics, CommitInfo};
use anyhow::{Context, Result};
use git2::{Commit, DiffOptions, Oid, Repository};
use std::collections::HashSet;

pub struct GitAnalyzer {
    repo: Repository,
}

impl GitAnalyzer {
    pub fn new(repo_path: &str) -> Result<Self> {
        let repo = Repository::open(repo_path).context("Failed to open git repository")?;
        Ok(Self { repo })
    }

    pub fn get_commits_between_branches(
        &self,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<Vec<CommitInfo>> {
        let source_commit = self.get_branch_commit(source_branch)?;
        let target_commit = self.get_branch_commit(target_branch)?;

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(source_commit.id())?;
        revwalk.hide(target_commit.id())?;

        let mut commits = Vec::new();
        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            commits.push(self.commit_to_info(&commit)?);
        }

        Ok(commits)
    }

    pub fn analyze_changes_between_branches(
        &self,
        source_branch: &str,
        target_branch: &str,
    ) -> Result<(CodeMetrics, Vec<String>)> {
        let source_commit = self.get_branch_commit(source_branch)?;
        let target_commit = self.get_branch_commit(target_branch)?;

        let source_tree = source_commit.tree()?;
        let target_tree = target_commit.tree()?;

        let diff = self.repo.diff_tree_to_tree(
            Some(&target_tree),
            Some(&source_tree),
            Some(&mut DiffOptions::new()),
        )?;

        let stats = diff.stats()?;
        let mut changed_files = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    changed_files.push(path.to_string_lossy().to_string());
                }
                true
            },
            None,
            None,
            None,
        )?;

        let metrics = CodeMetrics {
            lines_added: stats.insertions() as u32,
            lines_removed: stats.deletions() as u32,
            files_modified: stats.files_changed() as u32,
            complexity_score: None,
            test_coverage: None,
        };

        Ok((metrics, changed_files))
    }

    pub fn get_file_content_at_commit(&self, file_path: &str, branch_name: &str) -> Result<String> {
        let commit = self.get_branch_commit(branch_name)?;
        let tree = commit.tree()?;
        let entry = tree.get_path(std::path::Path::new(file_path))?;
        let blob = self.repo.find_blob(entry.id())?;

        Ok(String::from_utf8_lossy(blob.content()).to_string())
    }

    #[allow(dead_code)]
    pub fn get_file_content_at_commit_id(
        &self,
        commit_id: &str,
        file_path: &str,
    ) -> Result<String> {
        let oid = Oid::from_str(commit_id)?;
        let commit = self.repo.find_commit(oid)?;
        let tree = commit.tree()?;
        let entry = tree.get_path(std::path::Path::new(file_path))?;
        let blob = self.repo.find_blob(entry.id())?;

        Ok(String::from_utf8_lossy(blob.content()).to_string())
    }

    pub fn get_available_branches(&self) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        
        // Get local branches
        let local_branches = self.repo.branches(Some(git2::BranchType::Local))?;
        for branch in local_branches {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }

        // Get remote branches
        let remote_branches = self.repo.branches(Some(git2::BranchType::Remote))?;
        for branch in remote_branches {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                // Remove the remote prefix (e.g., "origin/main" -> "main")
                if let Some(short_name) = name.split('/').last() {
                    if !branches.contains(&short_name.to_string()) {
                        branches.push(short_name.to_string());
                    }
                }
            }
        }

        // Sort branches, putting common ones first
        branches.sort_by(|a, b| {
            let common_branches = ["main", "master", "develop", "dev"];
            let a_priority = common_branches.iter().position(|&x| x == a).unwrap_or(usize::MAX);
            let b_priority = common_branches.iter().position(|&x| x == b).unwrap_or(usize::MAX);
            
            match (a_priority, b_priority) {
                (usize::MAX, usize::MAX) => a.cmp(b), // Both not in common list, sort alphabetically
                (usize::MAX, _) => std::cmp::Ordering::Greater, // a not common, b is common
                (_, usize::MAX) => std::cmp::Ordering::Less, // a is common, b not common
                _ => a_priority.cmp(&b_priority), // Both common, sort by priority
            }
        });

        Ok(branches)
    }

    fn get_branch_commit(&self, branch_name: &str) -> Result<Commit<'_>> {
        let branch = self
            .repo
            .find_branch(branch_name, git2::BranchType::Local)
            .or_else(|_| self.repo.find_branch(branch_name, git2::BranchType::Remote))?;

        let target = branch.get().target().context("Branch has no target")?;
        self.repo
            .find_commit(target)
            .context("Failed to find commit")
    }

    fn commit_to_info(&self, commit: &Commit) -> Result<CommitInfo> {
        let signature = commit.author();
        let message = commit.message().unwrap_or("").to_string();

        // Get list of files changed in this commit
        let mut files_changed = Vec::new();
        if let Ok(parent) = commit.parent(0) {
            let tree = commit.tree()?;
            let parent_tree = parent.tree()?;
            let diff = self
                .repo
                .diff_tree_to_tree(Some(&parent_tree), Some(&tree), None)?;

            diff.foreach(
                &mut |delta, _| {
                    if let Some(path) = delta.new_file().path() {
                        files_changed.push(path.to_string_lossy().to_string());
                    }
                    true
                },
                None,
                None,
                None,
            )?;
        }

        Ok(CommitInfo {
            hash: commit.id().to_string(),
            author: signature.name().unwrap_or("Unknown").to_string(),
            message,
            timestamp: chrono::DateTime::from_timestamp(signature.when().seconds(), 0)
                .unwrap_or_default()
                .to_rfc3339(),
            files_changed,
        })
    }

    pub fn detect_technology_stack(
        &self,
        files: &[String],
    ) -> Result<crate::review::TechnologyStack> {
        let mut languages = HashSet::new();
        let frameworks = HashSet::new();
        let mut tools = HashSet::new();
        let databases = HashSet::new();

        for file in files {
            // Detect programming languages by file extension
            if let Some(ext) = std::path::Path::new(file).extension() {
                match ext.to_str() {
                    Some("rs") => {
                        languages.insert("Rust".to_string());
                    }
                    Some("py") => {
                        languages.insert("Python".to_string());
                    }
                    Some("js") => {
                        languages.insert("JavaScript".to_string());
                    }
                    Some("ts") => {
                        languages.insert("TypeScript".to_string());
                    }
                    Some("java") => {
                        languages.insert("Java".to_string());
                    }
                    Some("cpp" | "cc" | "cxx") => {
                        languages.insert("C++".to_string());
                    }
                    Some("c") => {
                        languages.insert("C".to_string());
                    }
                    Some("go") => {
                        languages.insert("Go".to_string());
                    }
                    Some("rb") => {
                        languages.insert("Ruby".to_string());
                    }
                    Some("php") => {
                        languages.insert("PHP".to_string());
                    }
                    Some("swift") => {
                        languages.insert("Swift".to_string());
                    }
                    Some("kt") => {
                        languages.insert("Kotlin".to_string());
                    }
                    _ => {}
                }
            }

            // Detect frameworks and tools by filename
            let filename = std::path::Path::new(file)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            match filename {
                "Cargo.toml" | "Cargo.lock" => {
                    tools.insert("Cargo".to_string());
                }
                "package.json" | "package-lock.json" => {
                    tools.insert("npm".to_string());
                }
                "requirements.txt" | "pyproject.toml" => {
                    tools.insert("pip".to_string());
                }
                "Dockerfile" => {
                    tools.insert("Docker".to_string());
                }
                "docker-compose.yml" | "docker-compose.yaml" => {
                    tools.insert("Docker Compose".to_string());
                }
                _ => {}
            }
        }

        Ok(crate::review::TechnologyStack {
            programming_languages: languages.into_iter().collect(),
            frameworks: frameworks.into_iter().collect(),
            tools: tools.into_iter().collect(),
            databases: databases.into_iter().collect(),
        })
    }
}
