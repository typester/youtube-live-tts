use crate::error::AppError;
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::HSTRING;
use windows::Media::SpeechSynthesis::SpeechSynthesizer;

pub struct TtsEngine {
    synthesizer: SpeechSynthesizer,
    is_speaking: Arc<AtomicBool>,
}

impl TtsEngine {
    pub fn new() -> Result<Self> {
        let synthesizer = SpeechSynthesizer::new()
            .map_err(|e| AppError::Windows(format!("Failed to create TTS engine: {}", e)))?;

        Ok(Self {
            synthesizer,
            is_speaking: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn set_voice(&mut self, voice_name: &str) -> Result<()> {
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

    pub fn speak(&self, text: &str) -> Result<()> {
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
                    use std::time::Duration;
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
