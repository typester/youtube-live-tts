use anyhow::Result;
use clap::Parser;

use youtube_live_tts::{config, youtube};

#[derive(Parser, Debug)]
#[clap(author, version, about = "Simple YouTube chat monitor")]
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
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "youtube_live_tts=debug".into()),
        )
        .init();

    // Parse command line arguments
    let args = Args::parse();
    tracing::info!("Starting YouTube Live Chat Monitor");

    // Load configuration
    let config = config::load_config(args.config.as_deref())?;

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
    tracing::info!("Press Ctrl+C to exit");

    while let Some(message) = chat_monitor.next_message().await? {
        println!(
            "[{}] {}: {}",
            message.timestamp, message.author, message.text
        );
    }

    Ok(())
}
