use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("YouTube API error: {0}")]
    YouTube(String),

    #[error("TTS engine error: {0}")]
    TTS(String),

    #[error("I/O error: {0}")]
    IO(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Windows API error: {0}")]
    Windows(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}
