mod config;
mod youtube;
mod tts;
mod error;

use clap::Parser;
use anyhow::Result;
use tracing::info;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// YouTube Live video ID
    #[clap(short, long)]
    video_id: String,

    /// Path to config file (optional)
    #[clap(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Parse command line arguments
    let args = Args::parse();
    info!("Starting YouTube Live TTS Bot");
    
    // Load configuration
    let config = config::load_config(args.config.as_deref())?;
    
    // Initialize TTS engine
    let mut tts_engine = tts::TtsEngine::new()?;
    if let Err(e) = tts_engine.set_voice(&config.voice_name) {
        info!("Failed to set voice '{}': {}", config.voice_name, e);
        info!("Using default voice instead");
    }
    
    // Start chat monitor
    let mut chat_monitor = youtube::ChatMonitor::new(&args.video_id, &config.api_key)?;
    chat_monitor.set_poll_interval(config.poll_interval_ms);
    
    // Main processing loop
    info!("Monitoring chat for video ID: {}", args.video_id);
    while let Some(message) = chat_monitor.next_message().await? {
        info!("New message from {}: {}", message.author, message.text);
        tts_engine.speak(&format!("{}さん: {}", message.author, message.text))?;
    }
    
    Ok(())
}
