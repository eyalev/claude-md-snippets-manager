use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use crate::publish::get_app_dir;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub default_repo: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;
        
        if !config_path.exists() {
            // Create default config if it doesn't exist
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }
        
        let content = fs::read_to_string(config_path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;
        
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }
    
    pub fn set_default_repo(&mut self, repo_name: String) -> Result<()> {
        self.default_repo = Some(repo_name);
        self.save()
    }
    
    pub fn get_default_repo(&self) -> Option<&str> {
        self.default_repo.as_deref()
    }
}

fn get_config_path() -> Result<std::path::PathBuf> {
    let app_dir = get_app_dir()?;
    Ok(app_dir.join("config.json"))
}

pub fn get_default_repo_name() -> Result<String> {
    let config = Config::load()?;
    
    if let Some(repo_name) = config.get_default_repo() {
        return Ok(repo_name.to_string());
    }
    
    // Fallback: try to detect first available repository
    let repos_dir = crate::publish::get_repos_dir()?;
    if repos_dir.exists() {
        for entry in fs::read_dir(repos_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    // Auto-set this as default and save
                    let mut config = Config::load()?;
                    config.set_default_repo(name.to_string())?;
                    return Ok(name.to_string());
                }
            }
        }
    }
    
    // Ultimate fallback
    Ok("default".to_string())
}