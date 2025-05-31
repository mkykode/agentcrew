use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::config::AgentCrewConfig;
use crate::database::Database;
use crate::git::GitUtils;

/// CLI command implementations
pub struct CommandHandler;

impl CommandHandler {
    /// Initialize agentcrew in the current project
    pub async fn init() -> Result<()> {
        println!("🚀 Initializing agentcrew in current project...");

        // Step 1: Check if we're in a git repository
        if !GitUtils::is_git_repository()? {
            anyhow::bail!(
                "❌ Not in a git repository. Please run 'git init' first or navigate to a git repository."
            );
        }

        // Step 2: Check if already initialized
        if AgentCrewConfig::is_initialized() {
            anyhow::bail!(
                "❌ agentcrew is already initialized in this project.\n   Use 'agentcrew status' to check current state."
            );
        }

        // Step 3: Get project information
        let repo_name = GitUtils::get_repository_name()
            .with_context(|| "Failed to determine repository name")?;
        let repo_root = GitUtils::get_repository_root()
            .with_context(|| "Failed to determine repository root")?;

        println!("  📁 Project: {}", repo_name);
        println!("  📍 Location: {}", repo_root.display());

        // Step 4: Create configuration
        let config = AgentCrewConfig::new(repo_name, repo_root);

        // Step 5: Save configuration and create directories
        config
            .save()
            .with_context(|| "Failed to save configuration")?;

        println!("  ✅ Created .agentcrew directory");

        // Step 6: Create additional directories
        let logs_dir = AgentCrewConfig::logs_dir()?;

        std::fs::create_dir_all(&logs_dir)
            .with_context(|| format!("Failed to create logs directory: {}", logs_dir.display()))?;

        println!("  ✅ Created logs directory");
        println!("  ✅ Generated config.toml");

        // Step 7: Initialize SQLite database
        let db_path = AgentCrewConfig::database_path()?;
        println!("  🗃️  Initializing database...");
        
        let db = Database::new(&db_path).await
            .with_context(|| "Failed to initialize database")?;
        
        // Run initial cleanup (won't delete anything on first run)
        db.cleanup_old_sessions(30).await
            .with_context(|| "Failed to run database cleanup")?;
        
        let stats = db.get_stats().await?;
        println!("  ✅ Database initialized (schema version {})", stats.schema_version);
        
        // Close the database connection
        db.close().await;

        // Step 8: Create .gitignore entry
        Self::update_gitignore()?;

        println!("  🎉 agentcrew initialized successfully!");
        println!();
        println!("💡 Next steps:");
        println!("   • Run 'agentcrew list' to see available agents");
        println!("   • Run 'agentcrew deploy --agents claude:1 --prompt \"your task\"' to start");
        println!("   • Run 'agentcrew status' to check current state");

        Ok(())
    }

    /// Update .gitignore to exclude agentcrew temporary files
    fn update_gitignore() -> Result<()> {
        let gitignore_path = PathBuf::from(".gitignore");
        let agentcrew_entries = [
            "",
            "# agentcrew",
            ".agentcrew/logs/",
            ".agentcrew/temp/",
            ".agentcrew/agentcrew.db*",
        ];

        if gitignore_path.exists() {
            let content = std::fs::read_to_string(&gitignore_path)
                .with_context(|| "Failed to read .gitignore")?;

            // Check if agentcrew entries already exist
            if content.contains("# agentcrew") {
                return Ok(());
            }

            // Append agentcrew entries
            let mut new_content = content;
            if !new_content.ends_with('\n') {
                new_content.push('\n');
            }
            new_content.push_str(&agentcrew_entries.join("\n"));
            new_content.push('\n');

            std::fs::write(&gitignore_path, new_content)
                .with_context(|| "Failed to update .gitignore")?;
        } else {
            // Create new .gitignore
            let content = agentcrew_entries.join("\n") + "\n";
            std::fs::write(&gitignore_path, content)
                .with_context(|| "Failed to create .gitignore")?;
        }

        println!("  ✅ Updated .gitignore");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_init_not_in_git_repo() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let original_dir = std::env::current_dir().expect("Should get current dir");

        std::env::set_current_dir(temp_dir.path()).expect("Should change dir");

        let result = CommandHandler::init().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a git repository"));

        std::env::set_current_dir(original_dir).expect("Should restore dir");
    }

    #[test]
    fn test_gitignore_update() {
        let temp_dir = TempDir::new().expect("Should create temp dir");
        let original_dir = std::env::current_dir().expect("Should get current dir");

        std::env::set_current_dir(temp_dir.path()).expect("Should change dir");

        // Test creating new .gitignore
        CommandHandler::update_gitignore().expect("Should update gitignore");
        assert!(fs::metadata(".gitignore").is_ok());

        let content = fs::read_to_string(".gitignore").expect("Should read gitignore");
        assert!(content.contains("# agentcrew"));
        assert!(content.contains(".agentcrew/logs/"));

        std::env::set_current_dir(original_dir).expect("Should restore dir");
    }
}
