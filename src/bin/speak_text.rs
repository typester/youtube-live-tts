use anyhow::Result;
use clap::Parser;
use config::TtsEngine;
use std::io::Write;
use youtube_live_tts::{config, tts};

#[derive(Parser, Debug)]
#[clap(author, version, about = "Simple TTS text speaker")]
struct Args {
    /// Text to speak (optional, reads from stdin if not provided)
    #[clap(short, long)]
    text: Option<String>,

    /// Windows voice name to use (when using Windows TTS)
    #[clap(short, long)]
    voice: Option<String>,

    /// TTS engine to use (windows or openai)
    #[clap(long)]
    tts_engine: Option<String>,

    /// OpenAI voice to use (when using OpenAI TTS)
    #[clap(long)]
    openai_voice: Option<String>,

    /// OpenAI model to use (when using OpenAI TTS)
    #[clap(long)]
    openai_model: Option<String>,

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
    let mut config = config::load_config(args.config.as_deref()).unwrap_or_default();

    // Override config with command line arguments if provided
    if let Some(engine) = &args.tts_engine {
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

    if let Some(voice) = &args.voice {
        config.windows_voice = voice.clone();
    }

    if let Some(voice) = &args.openai_voice {
        config.openai_voice = voice.clone();
    }

    if let Some(model) = &args.openai_model {
        config.openai_model = model.clone();
    }

    // Initialize TTS engine
    tracing::info!("Initializing TTS engine: {:?}", config.tts_engine);
    let tts_engine = tts::create_tts_engine(&config)?;

    // Get text to speak
    let text = if let Some(ref text) = args.text {
        text.clone()
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
            // Give more time for the speech to complete based on text length
            let wait_time = (line.len() as u64 * 100).max(2000);
            tracing::debug!("Waiting for {}ms for speech to complete", wait_time);
            tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
        }
        buffer
    };

    // Speak the text if provided via command line
    if !text.is_empty() && args.text.is_some() {
        tracing::info!("Speaking: {}", text);
        tts_engine.speak(&text)?;

        // Give time for speech to complete based on text length
        let wait_time = (text.len() as u64 * 100).max(5000);
        tracing::info!("Waiting for {}ms for speech to complete", wait_time);
        tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
    }

    Ok(())
}
