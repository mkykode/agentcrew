use std::path::Path;
use anyhow::{Context, Result};
use git2::Repository;

/// Git utilities for agentcrew
pub struct GitUtils;

impl GitUtils {
    /// Check if the current directory is in a git repository
    pub fn is_git_repository() -> Result<bool> {
        let current_dir = std::env::current_dir()
            .with_context(|| "Failed to get current directory")?;
        
        Ok(Self::find_git_repository(&current_dir).is_ok())
    }

    /// Find the git repository starting from the given path
    pub fn find_git_repository(path: &Path) -> Result<Repository> {
        Repository::discover(path)
            .with_context(|| "Not in a git repository")
    }

    /// Get the root directory of the git repository
    pub fn get_repository_root() -> Result<std::path::PathBuf> {
        let current_dir = std::env::current_dir()
            .with_context(|| "Failed to get current directory")?;
        
        let repo = Self::find_git_repository(&current_dir)?;
        let workdir = repo.workdir()
            .ok_or_else(|| anyhow::anyhow!("Repository has no working directory"))?;
        
        Ok(workdir.to_path_buf())
    }

    /// Get the name of the current repository
    pub fn get_repository_name() -> Result<String> {
        let repo_root = Self::get_repository_root()?;
        let name = repo_root
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Could not determine repository name"))?;
        
        Ok(name.to_string())
    }

    /// Check if there are uncommitted changes
    pub fn has_uncommitted_changes() -> Result<bool> {
        let current_dir = std::env::current_dir()
            .with_context(|| "Failed to get current directory")?;
        
        let repo = Self::find_git_repository(&current_dir)?;
        let statuses = repo.statuses(None)
            .with_context(|| "Failed to get repository status")?;
        
        Ok(!statuses.is_empty())
    }

    /// Get the current branch name
    pub fn get_current_branch() -> Result<String> {
        let current_dir = std::env::current_dir()
            .with_context(|| "Failed to get current directory")?;
        
        let repo = Self::find_git_repository(&current_dir)?;
        let head = repo.head()
            .with_context(|| "Failed to get HEAD reference")?;
        
        let branch_name = head
            .shorthand()
            .ok_or_else(|| anyhow::anyhow!("Could not get branch name"))?;
        
        Ok(branch_name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_is_git_repository_false() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let original_dir = std::env::current_dir().expect("Should get current dir");
        
        std::env::set_current_dir(temp_dir.path()).expect("Should change dir");
        
        let result = GitUtils::is_git_repository().expect("Should not error");
        assert!(!result);
        
        std::env::set_current_dir(original_dir).expect("Should restore dir");
    }

    #[test]
    fn test_repository_name() {
        // This test only runs if we're actually in a git repository
        if GitUtils::is_git_repository().unwrap_or(false) {
            let name = GitUtils::get_repository_name();
            assert!(name.is_ok());
            assert!(!name.unwrap().is_empty());
        }
    }
}