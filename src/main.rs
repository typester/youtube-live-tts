mod config;
mod error;
mod tts;
mod youtube;

use anyhow::Result;
use clap::Parser;
use config::TtsEngine;

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

    /// TTS engine to use (windows or openai)
    #[clap(long)]
    tts_engine: Option<String>,

    /// OpenAI voice to use (if tts-engine is openai)
    #[clap(long)]
    openai_voice: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with INFO level by default
    // Use RUST_LOG env var if set, otherwise default to info for this crate
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "youtube_live_tts=info".into()),
        )
        .init();

    // Parse command line arguments
    let args = Args::parse();
    tracing::info!("Starting YouTube Live TTS Bot");

    // Load configuration
    let mut config = config::load_config(args.config.as_deref())?;

    // Override config with command line arguments if provided
    if let Some(engine) = args.tts_engine {
        match engine.to_lowercase().as_str() {
            "windows" => config.tts_engine = TtsEngine::Windows,
            "openai" => config.tts_engine = TtsEngine::OpenAI,
            _ => {
                return Err(anyhow::anyhow!(
                    "Invalid TTS engine: {}. Supported engines: windows, openai",
                    engine
                ));
            }
        }
    }

    if let Some(voice) = args.openai_voice {
        config.openai_voice = voice;
    }

    // Initialize appropriate TTS engine
    tracing::info!("Initializing TTS engine: {:?}", config.tts_engine);
    let tts_engine = tts::create_tts_engine(&config)?;

    // Get video ID either directly or by finding the live stream for a channel
    let video_id = match (args.video_id, args.channel_id) {
        (Some(vid), _) => {
            tracing::info!("Using provided video ID: {}", vid);
            vid
        }
        (_, Some(channel)) => {
            tracing::info!("Searching for live stream for channel: {}", channel);
            let client = reqwest::Client::new();
            youtube::ChatMonitor::find_live_video_id_by_channel(&client, &channel, &config.api_key)
                .await?
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Either --video-id or --channel-id must be provided"
            ));
        }
    };

    // Start chat monitor
    let mut chat_monitor = youtube::ChatMonitor::new(&video_id, &config.api_key)?;
    chat_monitor.set_poll_interval(config.poll_interval_ms);

    // Main processing loop
    tracing::info!("Monitoring chat for video ID: {}", video_id);
    while let Some(message) = chat_monitor.next_message().await? {
        tracing::info!("New message from {}: {}", message.author, message.text);
        tts_engine.speak(&format!("{}さん: {}", message.author, message.text))?;
    }

    Ok(())
}
