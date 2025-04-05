use anyhow::Result;
use clap::Parser;
use std::io::Write;
use youtube_live_tts::{config, tts};

#[derive(Parser, Debug)]
#[clap(author, version, about = "Simple TTS text speaker")]
struct Args {
    /// Text to speak (optional, reads from stdin if not provided)
    #[clap(short, long)]
    text: Option<String>,

    /// Voice name to use
    #[clap(short, long)]
    voice: Option<String>,

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
    tracing::info!("Starting Text-to-Speech Test Tool");

    // Try to load configuration for default voice
    let config = config::load_config(args.config.as_deref()).unwrap_or_default();

    // Initialize TTS engine
    let mut tts_engine = tts::TtsEngine::new()?;

    // Set voice (command line takes precedence over config)
    let voice_name = args.voice.as_deref().unwrap_or(&config.voice_name);
    if let Err(e) = tts_engine.set_voice(voice_name) {
        tracing::warn!("Failed to set voice '{}': {}", voice_name, e);
        tracing::info!("Using default voice instead");
    } else {
        tracing::info!("Using voice: {}", voice_name);
    }

    // Get text to speak
    let text = if let Some(text) = args.text {
        text
    } else {
        // Read from stdin if no text provided
        tracing::info!("Enter text to speak (Ctrl+D to exit):");
        let mut buffer = String::new();
        loop {
            print!("> ");
            std::io::stdout().flush()?;

            let mut line = String::new();
            if std::io::stdin().read_line(&mut line)? == 0 {
                break; // EOF
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            buffer.push_str(line);

            // Speak the line
            tracing::info!("Speaking: {}", line);
            tts_engine.speak(line)?;

            // Wait for speaking to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
        buffer
    };

    // Speak the text if provided via command line
    if !text.is_empty() && args.text.is_some() {
        tracing::info!("Speaking: {}", text);
        tts_engine.speak(&text)?;

        // Give time for speech to complete
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }

    Ok(())
}
