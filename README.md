# YouTube Live TTS Bot

A command-line tool for reading YouTube Live chat messages with Text-to-Speech.

> **Important Note:** This application requires Windows to run. It was designed primarily for Windows systems and uses Windows-specific APIs for the default TTS engine.

## Features

- Simple, portable Windows executable
- Monitors YouTube Live chat in real-time
- Reads new messages aloud using Windows TTS or OpenAI TTS
- Configurable voices, engines, and polling settings

## Usage

You can start the bot using either a video ID or a channel ID:

```
# Using video ID directly
youtube-live-tts.exe --video-id YOUR_VIDEO_ID [--config path/to/config.toml]

# Using channel ID or username (auto-detects active live stream)
youtube-live-tts.exe --channel-id CHANNEL_ID_OR_USERNAME [--config path/to/config.toml]

# Specify TTS engine (windows or openai)
youtube-live-tts.exe --video-id YOUR_VIDEO_ID --tts-engine openai

# Using OpenAI TTS with specific voice
youtube-live-tts.exe --video-id YOUR_VIDEO_ID --tts-engine openai --openai-voice nova
```

Where:
- `YOUR_VIDEO_ID` is the ID of the YouTube Live stream (the part after `v=` in the URL)
- `CHANNEL_ID_OR_USERNAME` is either a channel ID (starting with "UC") or a username
- `config.toml` is an optional path to your configuration file
- `--tts-engine` can be either `windows` (default) or `openai`
- `--openai-voice` selects an OpenAI voice (when using OpenAI TTS)

### Debug Utilities

The package includes two utility programs for testing and debugging:

#### Chat Monitor

Monitors YouTube Live chat without TTS:

```
# By video ID
chat_monitor.exe --video-id YOUR_VIDEO_ID

# By channel ID or username
chat_monitor.exe --channel-id CHANNEL_ID_OR_USERNAME
```

#### Text Speaker

Test the TTS engine directly:

```
# Speak text from command line
speak_text.exe --text "Hello, world!"

# Specify a Windows voice
speak_text.exe --voice "Microsoft Zira"

# Use OpenAI TTS
speak_text.exe --tts-engine openai --text "Hello, world!"

# Use OpenAI TTS with specific voice and model
speak_text.exe --tts-engine openai --openai-voice nova --openai-model tts-1-hd --text "Hello, world!"

# Interactive mode (reads from stdin)
speak_text.exe
```

## Configuration

Create a `config.toml` file with the following options:

```toml
# Required: Your YouTube API key
api_key = "YOUR_API_KEY_HERE"

# Optional: How often to poll for new messages (milliseconds)
poll_interval_ms = 3000

# TTS Configuration
# TTS engine to use: "windows" or "openai"
tts_engine = "windows"

# Windows TTS configuration (when tts_engine = "windows")
# Common voices: "Microsoft David", "Microsoft Zira", "Microsoft Mark", etc.
windows_voice = "Microsoft David"

# OpenAI TTS configuration (when tts_engine = "openai")
openai_api_key = "YOUR_OPENAI_API_KEY_HERE"
# Available models: tts-1, tts-1-hd
openai_model = "tts-1"
# Available voices: alloy, echo, fable, onyx, nova, shimmer
openai_voice = "alloy"
```

The configuration file can be placed in one of these locations:
1. Path specified with `--config` argument
2. User config directory: `%APPDATA%\youtube-live-tts\config.toml`
3. Current directory: `config.toml`

## Building from Source

Requirements:
- Rust toolchain (https://rustup.rs/)
- Windows target support (`rustup target add x86_64-pc-windows-gnu`)

Build commands:

```
# For local Windows build
cargo build --release

# For cross-compilation from Linux/Mac to Windows
cargo build --release --target x86_64-pc-windows-gnu
```

> **Note:** This application must run on Windows, even when using OpenAI TTS, due to dependencies on Windows-specific APIs. When using OpenAI TTS, the application requires an internet connection to access the OpenAI API.

## Getting a YouTube API Key

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project
3. Enable the YouTube Data API v3
4. Create API credentials
5. Copy the API key to your config.toml file

## License

MIT
