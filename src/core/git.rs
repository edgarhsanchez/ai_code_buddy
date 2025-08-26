use anyhow::{anyhow, Result};
use git2::{Repository, Status};

pub struct GitAnalyzer {
    repo: Repository,
}

impl GitAnalyzer {
    pub fn new(repo_path: &str) -> Result<Self> {
        let repo = Repository::open(repo_path)
            .map_err(|e| anyhow!("Failed to open repository: {}", e))?;
        
        Ok(Self { repo })
    }
    
    pub fn get_changed_files(&self, source_branch: &str, target_branch: &str) -> Result<Vec<String>> {
        let mut all_files = Vec::new();
        let mut committed_files = std::collections::HashSet::new();

        // Get committed changes between branches
        if source_branch != target_branch {
            let source_commit = self.get_commit(source_branch)?;
            let target_commit = self.get_commit(target_branch)?;
            
            let source_tree = source_commit.tree()?;
            let target_tree = target_commit.tree()?;
            
            let diff = self.repo.diff_tree_to_tree(Some(&source_tree), Some(&target_tree), None)?;
            
            diff.foreach(
                &mut |delta, _progress| {
                    if let Some(file) = delta.new_file().path() {
                        let file_path = file.to_string_lossy().to_string();
                        committed_files.insert(file_path.clone());
                        all_files.push(file_path);
                    }
                    true
                },
                None,
                None,
                None,
            )?;
        }

        // Get uncommitted changes (staged and modified)
        let uncommitted_files = self.get_uncommitted_files()?;
        for file in uncommitted_files {
            if !committed_files.contains(&file) {
                all_files.push(file);
            }
        }
        
        Ok(all_files)
    }

    pub fn get_uncommitted_files(&self) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let statuses = self.repo.statuses(None)?;
        
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                files.push(path.to_string());
            }
        }
        
        Ok(files)
    }

    pub fn get_file_status(&self, file_path: &str) -> Result<super::review::CommitStatus> {
        let statuses = self.repo.statuses(None)?;
        
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                if path == file_path {
                    let status = entry.status();
                    
                    if status.contains(Status::INDEX_NEW) 
                        || status.contains(Status::INDEX_MODIFIED) 
                        || status.contains(Status::INDEX_DELETED) {
                        return Ok(super::review::CommitStatus::Staged);
                    }
                    
                    if status.contains(Status::WT_NEW) {
                        return Ok(super::review::CommitStatus::Untracked);
                    }
                    
                    if status.contains(Status::WT_MODIFIED) 
                        || status.contains(Status::WT_DELETED) {
                        return Ok(super::review::CommitStatus::Modified);
                    }
                }
            }
        }
        
        // If not in status, assume it's committed
        Ok(super::review::CommitStatus::Committed)
    }
    
    pub fn get_file_content(&self, file_path: &str, branch: &str) -> Result<String> {
        // First check if file has uncommitted changes
        let file_status = self.get_file_status(file_path)?;
        
        match file_status {
            super::review::CommitStatus::Untracked | 
            super::review::CommitStatus::Modified => {
                // Read from working directory
                let full_path = self.repo.workdir()
                    .ok_or_else(|| anyhow!("Repository has no working directory"))?
                    .join(file_path);
                
                std::fs::read_to_string(&full_path)
                    .map_err(|e| anyhow!("Failed to read file from working directory: {}", e))
            },
            super::review::CommitStatus::Staged => {
                // Try to read from index first, fall back to working directory
                match self.get_file_content_from_index(file_path) {
                    Ok(content) => Ok(content),
                    Err(_) => {
                        let full_path = self.repo.workdir()
                            .ok_or_else(|| anyhow!("Repository has no working directory"))?
                            .join(file_path);
                        
                        std::fs::read_to_string(&full_path)
                            .map_err(|e| anyhow!("Failed to read file from working directory: {}", e))
                    }
                }
            },
            super::review::CommitStatus::Committed => {
                // Read from commit
                let commit = self.get_commit(branch)?;
                let tree = commit.tree()?;
                
                let entry = tree.get_path(std::path::Path::new(file_path))?;
                let object = self.repo.find_object(entry.id(), Some(git2::ObjectType::Blob))?;
                let blob = object.as_blob()
                    .ok_or_else(|| anyhow!("Object is not a blob"))?;
                
                let content = std::str::from_utf8(blob.content())
                    .map_err(|e| anyhow!("Invalid UTF-8 in file: {}", e))?;
                
                Ok(content.to_string())
            }
        }
    }

    fn get_file_content_from_index(&self, file_path: &str) -> Result<String> {
        let index = self.repo.index()?;
        let entry = index.get_path(std::path::Path::new(file_path), 0)
            .ok_or_else(|| anyhow!("File not found in index"))?;
        
        let object = self.repo.find_object(entry.id, Some(git2::ObjectType::Blob))?;
        let blob = object.as_blob()
            .ok_or_else(|| anyhow!("Object is not a blob"))?;
        
        let content = std::str::from_utf8(blob.content())
            .map_err(|e| anyhow!("Invalid UTF-8 in file: {}", e))?;
        
        Ok(content.to_string())
    }
    
    fn get_commit(&self, branch_name: &str) -> Result<git2::Commit<'_>> {
        let reference = if branch_name == "HEAD" {
            self.repo.head()?
        } else {
            self.repo.find_reference(&format!("refs/heads/{}", branch_name))?
        };
        
        let oid = reference.target()
            .ok_or_else(|| anyhow!("Invalid reference"))?;
        
        self.repo.find_commit(oid)
            .map_err(|e| anyhow!("Failed to find commit: {}", e))
    }
}