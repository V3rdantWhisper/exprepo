use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppResult;

/// Persistent application configuration, stored as JSON in the app config dir.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Working directory of the exp git repository.
    pub repo_path: PathBuf,
    /// Remote URL (https) for GitHub sync, if configured.
    pub remote_url: Option<String>,
    /// Branch used for push/pull.
    pub default_branch: String,
    /// Author identity used for commits.
    pub git_name: String,
    pub git_email: String,
}

impl AppConfig {
    pub fn default_for(data_dir: &Path) -> Self {
        AppConfig {
            repo_path: data_dir.join("repo"),
            remote_url: None,
            default_branch: "main".to_string(),
            git_name: "ExpRepo".to_string(),
            git_email: "exprepo@localhost".to_string(),
        }
    }

    pub fn load_or_default(config_path: &Path, data_dir: &Path) -> AppResult<Self> {
        if config_path.exists() {
            let raw = std::fs::read_to_string(config_path)?;
            let cfg: AppConfig = serde_json::from_str(&raw)?;
            Ok(cfg)
        } else {
            let cfg = AppConfig::default_for(data_dir);
            cfg.save(config_path)?;
            Ok(cfg)
        }
    }

    pub fn save(&self, config_path: &Path) -> AppResult<()> {
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, raw)?;
        Ok(())
    }
}
