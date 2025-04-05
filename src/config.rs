use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    #[serde(default = "default_voice")]
    pub voice_name: String,
}

fn default_poll_interval() -> u64 {
    3000 // 3 seconds
}

fn default_voice() -> String {
    "Microsoft David".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            poll_interval_ms: default_poll_interval(),
            voice_name: default_voice(),
        }
    }
}

pub fn load_config(config_path: Option<&str>) -> Result<Config> {
    // If config path is provided, load from there
    if let Some(path) = config_path {
        return parse_config_file(path);
    }
    
    // Otherwise check default locations
    if let Some(config_dir) = dirs::config_dir() {
        let default_path = config_dir.join("youtube-live-tts/config.toml");
        if default_path.exists() {
            return parse_config_file(default_path);
        }
    }
    
    // Fallback to current directory
    let local_config = Path::new("config.toml");
    if local_config.exists() {
        return parse_config_file(local_config);
    }
    
    // If no config found, return error
    Err(AppError::Config("No configuration file found. Please provide API key".to_string()).into())
}

fn parse_config_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    let content = fs::read_to_string(path)?
        .parse::<toml::Table>()?;
    
    let config = toml::from_str(&content.to_string())?;
    Ok(config)
}
