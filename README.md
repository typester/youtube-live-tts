# YouTube Live TTS Bot

A command-line tool for reading YouTube Live chat messages with Text-to-Speech.

## Features

- Simple, portable Windows executable
- Monitors YouTube Live chat in real-time
- Reads new messages aloud using Windows TTS voices
- Configurable voice and polling settings

## Usage

```
youtube-live-tts.exe --video-id YOUR_VIDEO_ID [--config path/to/config.toml]
```

Where:
- `YOUR_VIDEO_ID` is the ID of the YouTube Live stream (the part after `v=` in the URL)
- `config.toml` is an optional path to your configuration file

## Configuration

Create a `config.toml` file with the following options:

```toml
# Required: Your YouTube API key
api_key = "YOUR_API_KEY_HERE"

# Optional: How often to poll for new messages (milliseconds)
poll_interval_ms = 3000

# Optional: TTS voice to use (Windows voice name)
voice_name = "Microsoft David"
```

The configuration file can be placed in one of these locations:
1. Path specified with `--config` argument
2. User config directory: `%APPDATA%\youtube-live-tts\config.toml`
3. Current directory: `config.toml`

## Building from Source

Requirements:
- Rust toolchain (https://rustup.rs/)

Build commands:

```
cargo build --release
```

For Windows cross-compilation:

```
cargo build --release --target x86_64-pc-windows-gnu
```

## Getting a YouTube API Key

1. Go to the [Google Cloud Console](https://console.cloud.google.com/)
2. Create a new project
3. Enable the YouTube Data API v3
4. Create API credentials
5. Copy the API key to your config.toml file

## License

MIT
