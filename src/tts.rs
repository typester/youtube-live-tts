use crate::config::TtsEngine as TtsEngineType;
use crate::error::AppError;
use anyhow::Result;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub trait TextToSpeech: Send + Sync {
    fn speak(&self, text: &str) -> Result<()>;
}

// Factory function to create the appropriate TTS engine
pub fn create_tts_engine(config: &crate::config::Config) -> Result<Box<dyn TextToSpeech>> {
    match config.tts_engine {
        TtsEngineType::Windows => {
            let mut engine = WindowsTtsEngine::new()?;
            let voice_name = if !config.windows_voice.is_empty() {
                &config.windows_voice
            } else {
                &config.voice_name // For backward compatibility
            };

            if let Err(e) = engine.set_voice(voice_name) {
                tracing::warn!("Failed to set Windows voice '{}': {}", voice_name, e);
                tracing::info!("Using default Windows voice instead");
            }
            Ok(Box::new(engine))
        }
        TtsEngineType::OpenAI => {
            if let Some(api_key) = &config.openai_api_key {
                Ok(Box::new(OpenAITtsEngine::new(
                    api_key.clone(),
                    config.openai_model.clone(),
                    config.openai_voice.clone(),
                )?))
            } else {
                Err(AppError::Config(
                    "OpenAI API key is required for OpenAI TTS engine".to_string(),
                )
                .into())
            }
        }
    }
}

// Windows TTS implementation
pub struct WindowsTtsEngine {
    synthesizer: windows::Media::SpeechSynthesis::SpeechSynthesizer,
    is_speaking: Arc<AtomicBool>,
}

impl WindowsTtsEngine {
    pub fn new() -> Result<Self> {
        use windows::Media::SpeechSynthesis::SpeechSynthesizer;

        let synthesizer = SpeechSynthesizer::new()
            .map_err(|e| AppError::Windows(format!("Failed to create TTS engine: {}", e)))?;

        Ok(Self {
            synthesizer,
            is_speaking: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn set_voice(&mut self, voice_name: &str) -> Result<()> {
        use windows::Media::SpeechSynthesis::SpeechSynthesizer;

        // Get all available voices using the static method
        let voices = SpeechSynthesizer::AllVoices()
            .map_err(|e| AppError::Windows(format!("Failed to get voices: {}", e)))?;

        // Find requested voice
        let size = voices
            .Size()
            .map_err(|e| AppError::Windows(format!("Failed to get voices size: {}", e)))?;

        for i in 0..size {
            let voice = voices.GetAt(i).map_err(|e| {
                AppError::Windows(format!("Failed to get voice at index {}: {}", i, e))
            })?;

            let name = voice
                .DisplayName()
                .map_err(|e| AppError::Windows(format!("Failed to get voice name: {}", e)))?;

            if name.to_string().contains(voice_name) {
                // Set voice
                self.synthesizer
                    .SetVoice(&voice)
                    .map_err(|e| AppError::Windows(format!("Failed to set voice: {}", e)))?;

                return Ok(());
            }
        }

        Err(AppError::Tts(format!("Voice '{}' not found", voice_name)).into())
    }
}

impl TextToSpeech for WindowsTtsEngine {
    fn speak(&self, text: &str) -> Result<()> {
        use windows::core::HSTRING;

        if self.is_speaking.load(Ordering::SeqCst) {
            tracing::debug!("Already speaking, skipping text: {}", text);
            return Ok(());
        }

        self.is_speaking.store(true, Ordering::SeqCst);
        let is_speaking = self.is_speaking.clone();

        let text_hstring = HSTRING::from(text);
        let synthesizer = self.synthesizer.clone();

        tokio::task::spawn_blocking(move || {
            let result = synthesizer
                .SynthesizeTextToStreamAsync(&text_hstring)
                .and_then(|async_op| async_op.get())
                .and_then(|stream| {
                    use std::thread;
                    use windows::Media::Core::MediaSource;
                    use windows::Media::Playback::{MediaPlaybackItem, MediaPlayer};

                    // Create a MediaPlayer and play the stream
                    let player = MediaPlayer::new().map_err(|e| {
                        windows::core::Error::new(
                            windows::core::HRESULT(0x80004005u32 as i32),
                            HSTRING::from(format!("Failed to create MediaPlayer: {}", e)),
                        )
                    })?;

                    // Create a MediaSource from the stream
                    let content_type = HSTRING::from("");
                    let media_source = MediaSource::CreateFromStream(&stream, &content_type)
                        .map_err(|e| {
                            windows::core::Error::new(
                                windows::core::HRESULT(0x80004005u32 as i32),
                                HSTRING::from(format!("Failed to create MediaSource: {}", e)),
                            )
                        })?;

                    // Create a MediaPlaybackItem from the source
                    let playback_item = MediaPlaybackItem::Create(&media_source).map_err(|e| {
                        windows::core::Error::new(
                            windows::core::HRESULT(0x80004005u32 as i32),
                            HSTRING::from(format!("Failed to create MediaPlaybackItem: {}", e)),
                        )
                    })?;

                    // Set the source and play
                    player.SetSource(&playback_item).map_err(|e| {
                        windows::core::Error::new(
                            windows::core::HRESULT(0x80004005u32 as i32),
                            HSTRING::from(format!("Failed to set source: {}", e)),
                        )
                    })?;

                    player.Play().map_err(|e| {
                        windows::core::Error::new(
                            windows::core::HRESULT(0x80004005u32 as i32),
                            HSTRING::from(format!("Failed to play audio: {}", e)),
                        )
                    })?;

                    // Estimate duration based on text length (rough approximation) with a minimum
                    let estimated_duration_ms = (text_hstring.len() as u64 * 100).max(2000); // ~100ms per character with 2sec minimum
                    tracing::debug!(
                        "Playing audio, estimated duration: {}ms",
                        estimated_duration_ms
                    );

                    // Sleep to allow playback to complete
                    thread::sleep(Duration::from_millis(estimated_duration_ms));

                    tracing::debug!("Audio playback completed");
                    Ok(())
                });

            is_speaking.store(false, Ordering::SeqCst);

            if let Err(e) = result {
                tracing::error!("TTS error: {}", e);
            }
        });

        Ok(())
    }
}

// OpenAI TTS implementation
pub struct OpenAITtsEngine {
    api_key: String,
    model: String,
    voice: String,
    is_speaking: Arc<AtomicBool>,
    client: reqwest::Client,
    temp_dir: PathBuf,
}

impl OpenAITtsEngine {
    pub fn new(api_key: String, model: String, voice: String) -> Result<Self> {
        let temp_dir = tempfile::Builder::new()
            .prefix("youtube-live-tts")
            .tempdir()?
            .into_path();

        Ok(Self {
            api_key,
            model,
            voice,
            is_speaking: Arc::new(AtomicBool::new(false)),
            client: reqwest::Client::new(),
            temp_dir,
        })
    }
}

impl TextToSpeech for OpenAITtsEngine {
    fn speak(&self, text: &str) -> Result<()> {
        if self.is_speaking.load(Ordering::SeqCst) {
            tracing::debug!("Already speaking with OpenAI TTS, skipping text: {}", text);
            return Ok(());
        }

        // Mark as speaking
        self.is_speaking.store(true, Ordering::SeqCst);
        let is_speaking = self.is_speaking.clone();

        // Clone required values for async task
        let api_key = self.api_key.clone();
        let model = self.model.clone();
        let voice = self.voice.clone();
        let client = self.client.clone();
        let text = text.to_string();
        let temp_dir = self.temp_dir.clone();

        // Spawn async task for TTS
        tokio::spawn(async move {
            let result = async {
                // Create request JSON
                let json = serde_json::json!({
                    "model": model,
                    "input": text,
                    "voice": voice,
                    "response_format": "mp3"
                });

                tracing::debug!("Sending TTS request to OpenAI API for text: {}", text);

                // Send request to OpenAI API
                let response = client
                    .post("https://api.openai.com/v1/audio/speech")
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .json(&json)
                    .send()
                    .await?;

                // Check for error
                if !response.status().is_success() {
                    let error_text = response.text().await?;
                    return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
                }

                // Get the audio bytes
                let bytes = response.bytes().await?;
                tracing::debug!("Received {} bytes of audio from OpenAI", bytes.len());

                // Save to temporary file
                let temp_file_path =
                    temp_dir.join(format!("tts_{}.mp3", chrono::Utc::now().timestamp_millis()));
                let mut file = File::create(&temp_file_path).await?;
                file.write_all(&bytes).await?;
                file.flush().await?;
                drop(file);

                tracing::debug!("Saved audio to temporary file: {:?}", temp_file_path);

                // Play the audio using rodio
                let audio_bytes = bytes.to_vec();
                tokio::task::spawn_blocking(move || -> Result<()> {
                    let cursor = Cursor::new(audio_bytes);

                    // Initialize audio output
                    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
                    let sink = rodio::Sink::try_new(&stream_handle)?;

                    // Load and play the audio
                    sink.append(rodio::Decoder::new(cursor)?);

                    // Wait for playback to complete
                    sink.sleep_until_end();

                    tracing::debug!("OpenAI TTS audio playback completed");
                    Ok(())
                })
                .await??;

                // Try to clean up temp file
                if let Err(e) = tokio::fs::remove_file(&temp_file_path).await {
                    tracing::warn!("Failed to clean up temp file: {}", e);
                }

                Ok(())
            }
            .await;

            // Reset speaking flag regardless of result
            is_speaking.store(false, Ordering::SeqCst);

            // Log any errors
            if let Err(e) = result {
                tracing::error!("OpenAI TTS error: {}", e);
            }
        });

        Ok(())
    }
}
