mod config;
mod error;
mod tts;
mod youtube;

use anyhow::Result;
use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// YouTube Live video ID
    #[clap(short, long, group = "target")]
    video_id: Option<String>,

    /// YouTube channel ID or username
    #[clap(short, long, group = "target")]
    channel_id: Option<String>,

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

    // Get video ID either directly or by finding the live stream for a channel
    let video_id = match (args.video_id, args.channel_id) {
        (Some(vid), _) => {
            info!("Using provided video ID: {}", vid);
            vid
        }
        (_, Some(channel)) => {
            info!("Searching for live stream for channel: {}", channel);
            let client = reqwest::Client::new();
            youtube::ChatMonitor::find_live_video_id_by_channel(&client, &channel, &config.api_key)
                .await?
        }
        _ => {
            return Err(
                anyhow::anyhow!("Either --video-id or --channel-id must be provided").into(),
            );
        }
    };

    // Start chat monitor
    let mut chat_monitor = youtube::ChatMonitor::new(&video_id, &config.api_key)?;
    chat_monitor.set_poll_interval(config.poll_interval_ms);

    // Main processing loop
    info!("Monitoring chat for video ID: {}", video_id);
    while let Some(message) = chat_monitor.next_message().await? {
        info!("New message from {}: {}", message.author, message.text);
        tts_engine.speak(&format!("{}さん: {}", message.author, message.text))?;
    }

    Ok(())
}
