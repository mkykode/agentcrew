use std::path::PathBuf;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

/// Project configuration for agentcrew
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCrewConfig {
    /// Project name
    pub project_name: String,
    /// Project root directory
    pub project_root: PathBuf,
    /// Default agents to use
    pub default_agents: Vec<String>,
    /// Maximum number of concurrent agents
    pub max_agents: u32,
    /// Default prompt template
    pub default_prompt: Option<String>,
    /// Configuration version for future compatibility
    pub version: String,
}

impl Default for AgentCrewConfig {
    fn default() -> Self {
        Self {
            project_name: "untitled-project".to_string(),
            project_root: PathBuf::from("."),
            default_agents: vec!["claude".to_string()],
            max_agents: 5,
            default_prompt: None,
            version: "0.1.0".to_string(),
        }
    }
}

impl AgentCrewConfig {
    /// Create a new configuration for the given project
    pub fn new(project_name: String, project_root: PathBuf) -> Self {
        Self {
            project_name,
            project_root,
            ..Default::default()
        }
    }

    /// Load configuration from .agentcrew/config.toml
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;
        
        if !config_path.exists() {
            anyhow::bail!("agentcrew not initialized. Run 'agentcrew init' first.");
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;
        
        let config: AgentCrewConfig = toml::from_str(&content)
            .with_context(|| "Failed to parse config.toml")?;
        
        Ok(config)
    }

    /// Save configuration to .agentcrew/config.toml
    pub fn save(&self) -> Result<()> {
        let agentcrew_dir = Self::agentcrew_dir()?;
        
        // Create .agentcrew directory if it doesn't exist
        if !agentcrew_dir.exists() {
            fs::create_dir_all(&agentcrew_dir)
                .with_context(|| format!("Failed to create directory: {}", agentcrew_dir.display()))?;
        }

        let config_path = Self::config_file_path()?;
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;
        
        Ok(())
    }

    /// Check if agentcrew is already initialized in current directory
    pub fn is_initialized() -> bool {
        Self::config_file_path()
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    /// Get the .agentcrew directory path
    pub fn agentcrew_dir() -> Result<PathBuf> {
        let current_dir = std::env::current_dir()
            .with_context(|| "Failed to get current directory")?;
        Ok(current_dir.join(".agentcrew"))
    }

    /// Get the config file path
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::agentcrew_dir()?.join("config.toml"))
    }

    /// Get the sessions directory path
    pub fn sessions_dir() -> Result<PathBuf> {
        Ok(Self::agentcrew_dir()?.join("sessions"))
    }

    /// Get the logs directory path
    pub fn logs_dir() -> Result<PathBuf> {
        Ok(Self::agentcrew_dir()?.join("logs"))
    }

    /// Get the database file path
    pub fn database_path() -> Result<PathBuf> {
        Ok(Self::agentcrew_dir()?.join("agentcrew.db"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = AgentCrewConfig::default();
        assert_eq!(config.project_name, "untitled-project");
        assert_eq!(config.max_agents, 5);
        assert_eq!(config.version, "0.1.0");
    }

    #[test]
    fn test_config_new() {
        let config = AgentCrewConfig::new(
            "test-project".to_string(),
            PathBuf::from("/tmp/test")
        );
        assert_eq!(config.project_name, "test-project");
        assert_eq!(config.project_root, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_serialization() {
        let config = AgentCrewConfig::default();
        let toml_str = toml::to_string(&config).expect("Should serialize");
        let deserialized: AgentCrewConfig = toml::from_str(&toml_str).expect("Should deserialize");
        
        assert_eq!(config.project_name, deserialized.project_name);
        assert_eq!(config.max_agents, deserialized.max_agents);
    }
}