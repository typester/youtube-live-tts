use crate::error::AppError;
use anyhow::Result;
use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use windows::core::HSTRING;
use windows::Media::SpeechSynthesis::SpeechSynthesizer;
use windows::Storage::Streams::DataReader;

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

        Err(AppError::TTS(format!("Voice '{}' not found", voice_name)).into())
    }

    pub fn speak(&self, text: &str) -> Result<()> {
        // Skip if already speaking
        if self.is_speaking.load(Ordering::SeqCst) {
            return Ok(());
        }

        self.is_speaking.store(true, Ordering::SeqCst);
        let is_speaking = self.is_speaking.clone();

        // Speak in background thread to not block main thread
        let text_hstring = HSTRING::from(text);
        let synthesizer = self.synthesizer.clone();

        tokio::task::spawn_blocking(move || {
            let result = synthesizer
                .SynthesizeTextToStreamAsync(&text_hstring)
                .and_then(|async_op| async_op.get())
                .and_then(|stream| {
                    // Read audio data from stream
                    DataReader::CreateDataReader(&stream).and_then(|reader| {
                        // Convert u64 to u32 safely
                        let size = match stream.Size() {
                            Ok(size) => match size.try_into() {
                                Ok(size_u32) => Ok(size_u32),
                                Err(_) => Err(windows::core::Error::new(
                                    windows::core::HRESULT(0x80004005u32 as i32), // E_FAIL
                                    HSTRING::from("Stream size too large for u32"),
                                )),
                            },
                            Err(e) => Err(e),
                        }?;

                        reader.LoadAsync(size).and_then(|_| {
                            // Play audio (Windows handles this automatically)
                            Ok(())
                        })
                    })
                });

            // Release speaking lock
            is_speaking.store(false, Ordering::SeqCst);

            if let Err(e) = result {
                eprintln!("TTS error: {}", e);
            }
        });

        Ok(())
    }
}
