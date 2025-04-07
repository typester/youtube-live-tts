# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Note:** This application is designed to run only on Windows systems, as it relies on the Windows TTS API.

## Build Commands

- Build (debug): `cargo build`
- Build (release): `cargo build --release`
- Cross-compile for Windows: `cargo build --release --target x86_64-pc-windows-gnu`
- Run: `cargo run -- --video-id YOUR_VIDEO_ID [--config path/to/config.toml]`
- Test: `cargo test`
- Test single: `cargo test test_name`
- Lint: `cargo clippy -- -D warnings`
- Format: `cargo fmt --all`

## Code Style Guidelines

- Follow standard Rust coding conventions and idioms
- Minimize comments - add them only for complex or non-obvious code sections
  - Avoid comments that merely repeat what the code is doing (e.g., "// Initialize logging" above a logging initialization)
  - Avoid comments for simple operations like "// Parse command line arguments"
  - Keep comments only when they explain WHY something is done, not WHAT is being done
- Always run `cargo fmt --all` before committing any code
- Always test cross-compilation with `cargo build --release --target x86_64-pc-windows-gnu` before committing
- Use fully qualified paths for macros (e.g., `tracing::info!()` instead of `use tracing::info; info!()`)
- Respond to user prompts in the same language they use (Japanese or English)
- All generated code and documentation should be in English regardless of prompt language
- Organize imports in the following order, with a blank line between groups:
  1. Standard library imports (`std::*`)
  2. External crate imports
  3. Internal crate imports (`crate::*`)
- Group related imports from the same namespace into a single consolidated `use` statement