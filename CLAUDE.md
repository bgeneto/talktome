# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TalkToMe is a cross-platform desktop application for real-time speech-to-text transcription with live translation capabilities. It's built as a Tauri application combining SvelteKit frontend with Rust backend.

**Technology Stack:**
- Frontend: SvelteKit + TypeScript + TailwindCSS
- Backend: Rust + Tauri v2
- Audio: CPAL (Cross-Platform Audio Library)
- APIs: OpenAI Whisper (STT) + GPT (translation)
- Security: Tauri Stronghold for secure storage

## Development Commands

```bash
# Development
npm run tauri dev          # Start dev server with hot reload
npm run dev               # Frontend only development

# Building & Testing
npm run build             # Build frontend
npm run tauri build       # Production build for current platform
npm run preview           # Preview production build

# Type Checking
npm run check             # Run TypeScript checks
npm run check:watch       # Continuous type checking

# Rust Development
cd src-tauri
cargo build               # Build Rust backend
cargo test                # Run Rust tests
cargo clippy              # Rust linting
```

## Architecture Overview

### Core Components

**Audio Pipeline** (`src-tauri/src/audio.rs`):
- Real-time audio capture using CPAL
- Voice Activity Detection (VAD) with configurable thresholds
- Audio chunking with overlap handling
- Single-threaded audio manager (non-Send AudioCapture constraints)

**Speech-to-Text Service** (`src-tauri/src/stt.rs`):
- OpenAI Whisper API integration
- WAV audio format encoding
- Retry logic and error handling
- Audio quality filtering

**Translation Service** (`src-tauri/src/translation.rs`):
- OpenAI GPT model integration
- Support for 20+ languages
- Grammar correction capabilities

**Settings Management**:
- Backend: `src-tauri/src/settings.rs` (Rust storage)
- Frontend: `src/lib/stores/settingsStore.ts` (Svelte store)
- Secure API key storage using Tauri Stronghold

**System Integration** (`src-tauri/src/lib.rs`):
- Global hotkeys (default: Ctrl+Shift+Space)
- System tray integration with menu
- Tauri notifications
- Text insertion utilities

### Key Architectural Patterns

1. **Single-threaded Audio**: AudioCapture is non-Send, requiring single-threaded management via message passing
2. **Event-driven**: Frontend-backend communication via Tauri events and commands
3. **Immutable State**: Svelte stores for reactive state management
4. **Modular Services**: Separate services for audio, STT, translation, settings

### Frontend Structure

```
src/
├── lib/stores/settingsStore.ts    # Centralized settings management
├── routes/                        # SvelteKit pages
│   ├── +layout.svelte            # Main layout with sidebar
│   ├── +page.svelte              # Recording interface
│   ├── preferences/              # Settings pages
│   ├── api-settings/             # OpenAI API config
│   ├── language-settings/        # Language preferences
│   └── audio-settings/           # Audio device & VAD
```

### Backend Structure

```
src-tauri/src/
├── lib.rs                        # Main Tauri app logic & commands
├── audio.rs                      # Audio capture & VAD
├── stt.rs                        # Speech-to-text service
├── translation.rs                # Translation service
├── settings.rs                   # Settings management
├── text_insertion.rs             # Text insertion utilities
└── debug_logger.rs               # Debug logging system
```

## Important Implementation Details

### VAD Configuration
Voice Activity Detection settings in `settingsStore.ts` control speech recognition:
- `speechThreshold`: Energy threshold for speech (default: 0.02)
- `silenceThreshold`: Energy threshold for silence (default: 0.01)
- `maxChunkDurationMs`: Maximum chunk duration (default: 5000ms)
- `overlapMs`: Overlap to prevent word cutting (default: 220ms)

### Audio Processing Constraints
- AudioCapture is non-Send due to CPAL limitations
- Use single-threaded audio manager with message passing
- Downsample to 16kHz for optimal speech recognition

### API Integration
- OpenAI-compatible endpoints supported (Azure, custom servers)
- API keys stored securely with Tauri Stronghold
- Retry logic and comprehensive error handling

### System Integration
- Global hotkeys parsed from strings (e.g., "Ctrl+Shift+Space")
- System tray with menu integration
- Application starts minimized to tray
- Cross-platform text insertion support

## Configuration Files

- `tauri.conf.json`: Tauri app configuration, window settings, build targets
- `package.json`: Node.js dependencies and scripts
- `src-tauri/Cargo.toml`: Rust dependencies and features
- `vite.config.js`: Vite bundler configuration
- `tailwind.config.js`: TailwindCSS styling configuration

## Development Notes

- Frontend runs on `http://localhost:1420` during development
- Production build output goes to `src-tauri/target/release/`
- Debug logs available in `%APPDATA%/TalkToMe/logs/` when enabled
- Audio debugging supports WAV dump functionality
- Settings are portable and work across different machines