use std::{fs, path::Path};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TtsEngine {
    Windows,
    OpenAI,
}

impl Default for TtsEngine {
    fn default() -> Self {
        TtsEngine::Windows
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_key: String,
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,

    // For backward compatibility
    #[serde(default = "default_voice")]
    pub voice_name: String,

    // TTS engine selection
    #[serde(default)]
    pub tts_engine: TtsEngine,

    // Windows TTS config
    #[serde(default = "default_voice")]
    pub windows_voice: String,

    // OpenAI TTS config
    pub openai_api_key: Option<String>,
    #[serde(default = "default_openai_model")]
    pub openai_model: String,
    #[serde(default = "default_openai_voice")]
    pub openai_voice: String,
}

fn default_poll_interval() -> u64 {
    3000 // 3 seconds
}

fn default_voice() -> String {
    "Microsoft David".to_string()
}

fn default_openai_model() -> String {
    "tts-1".to_string()
}

fn default_openai_voice() -> String {
    "alloy".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            poll_interval_ms: default_poll_interval(),
            voice_name: default_voice(),
            tts_engine: TtsEngine::default(),
            windows_voice: default_voice(),
            openai_api_key: None,
            openai_model: default_openai_model(),
            openai_voice: default_openai_voice(),
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
    let content = fs::read_to_string(path)?.parse::<toml::Table>()?;

    let mut config: Config = toml::from_str(&content.to_string())?;

    // For backward compatibility: if voice_name is set but windows_voice isn't,
    // copy the value to windows_voice
    if config.windows_voice.is_empty() && !config.voice_name.is_empty() {
        config.windows_voice = config.voice_name.clone();
    }

    Ok(config)
}
