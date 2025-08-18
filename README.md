# TalkToMe - Voice to Text with Translation

A cross-platform desktop application built with Tauri and Svelte that provides real-time speech-to-text transcription with live translation capabilities.

## 🚀 Features

- **Real-time Speech Recognition**: Convert spoken words to text using OpenAI Whisper API
- **Live Language Translation**: Translate transcribed text to multiple languages using GPT models
- **Voice Activity Detection (VAD)**: Smart audio processing with noise filtering and speech detection
- **System Tray Integration**: Always-on functionality with system tray controls
- **Global Hotkeys**: Configurable keyboard shortcuts for hands-free operation
- **Multi-language Support**: Support for 20+ languages for both input and output
- **Cross-platform**: Works on Windows, macOS, and Linux
- **Secure API Key Storage**: Encrypted storage of API credentials
- **Audio Device Selection**: Choose from available microphone inputs
- **Customizable Themes**: Light and dark mode support
- **Debug Logging**: Comprehensive logging for troubleshooting

## 🏗️ Architecture Overview

TalkToMe is built using a modern hybrid architecture:

- **Frontend**: Svelte + TypeScript + TailwindCSS
- **Backend**: Rust with Tauri framework
- **Audio Processing**: CPAL (Cross-Platform Audio Library) with custom VAD
- **APIs**: OpenAI Whisper for STT, GPT models for translation

### Core Components

1. **Audio Pipeline** (`src-tauri/src/audio.rs`)
   - Real-time audio capture using CPAL
   - Voice Activity Detection with configurable thresholds
   - Audio chunking with overlap handling
   - Signal conditioning (high-pass filter, AGC, noise gate)

2. **Speech-to-Text Service** (`src-tauri/src/stt.rs`)
   - OpenAI Whisper API integration
   - Audio encoding to WAV format
   - Retry logic and error handling
   - Quality filtering for audio chunks

3. **Translation Service** (`src-tauri/src/translation.rs`)
   - OpenAI GPT model integration
   - Grammar correction and translation
   - Support for 20+ languages
   - Configurable translation models

4. **Settings Management** (`src-tauri/src/settings.rs`)
   - Secure API key storage
   - Portable data directory support
   - Cross-platform configuration

5. **Debug System** (`src-tauri/src/debug_logger.rs`)
   - Comprehensive logging system
   - WAV dump capability for audio debugging
   - Runtime log level control

## 🛠️ Installation & Setup

### Prerequisites

- **Node.js** (v18 or later)
- **Rust** (latest stable)
- **System Dependencies**:
  - Windows: Windows 10/11, WebView2
  - macOS: macOS 10.13+
  - Linux: WebKit2GTK, various system libraries

### Development Setup

1. **Clone the repository**:
   ```bash
   git clone https://github.com/bgeneto/TalkToMe.git
   cd TalkToMe
   ```

2. **Install dependencies**:
   ```bash
   npm install
   ```

3. **Install Rust dependencies**:
   ```bash
   cd src-tauri
   cargo build
   cd ..
   ```

4. **Run in development mode**:
   ```bash
   npm run tauri dev
   ```

### Production Build

```bash
# Build for current platform
npm run tauri build

# The built application will be in src-tauri/target/release/
```

## ⚙️ Configuration

### API Setup

1. **OpenAI API Key**: Required for speech recognition and translation
   - Go to API Settings page in the application
   - Enter your OpenAI API key
   - Configure API endpoint (default: https://api.openai.com/v1)
   - Select STT model (e.g., whisper-large-v3)
   - Select translation model (e.g., gpt-3.5-turbo)

2. **Language Configuration**:
   - Set spoken language (auto-detect recommended)
   - Set target translation language
   - Configure quick-access languages

3. **Audio Settings**:
   - Select microphone device
   - Test audio levels
   - Configure VAD parameters

4. **Hotkeys**:
   - Push-to-talk: Default `Ctrl+Win`
   - Hands-free recording: Default `Ctrl+Win+Space`

### VAD (Voice Activity Detection) Tuning

Fine-tune voice detection in the settings store:

```typescript
vad: {
  speechThreshold: 0.02,        // Energy threshold for speech
  silenceThreshold: 0.01,       // Energy threshold for silence
  maxChunkDurationMs: 5000,     // Maximum chunk duration
  silenceTimeoutMs: 500,        // Silence timeout
  overlapMs: 220,               // Overlap to prevent word cutting
  sampleRate: 16000             // Audio sample rate
}
```

## 🎯 Usage

### Basic Operation

1. **Start the Application**: Launch TalkToMe from desktop or start menu
2. **Configure API**: Set up your OpenAI API key in API Settings
3. **Select Languages**: Choose input and output languages
4. **Start Recording**: Click the record button or use hotkey
5. **Speak Naturally**: Talk into your microphone
6. **View Results**: See transcription and translation in real-time
7. **Copy/Export**: Use built-in tools to copy or export text

### System Tray

The application runs in the system tray with these options:
- Start/Stop Recording
- Show Main Window
- Preferences
- API Settings
- Language Settings
- Audio Settings
- About
- Quit

### Hotkeys

- **Push-to-Talk**: Hold to record, release to stop
- **Hands-Free**: Toggle recording on/off
- **Emergency Stop**: Immediately stop all recording

## 🔧 Advanced Configuration

### Audio Pipeline Customization

The audio processing pipeline can be customized in `audio.rs`:

```rust
// VAD Configuration
VoiceActivityDetector {
    speech_threshold: 0.02,
    silence_threshold: 0.01,
    min_speech_duration_ms: 350,
    max_speech_duration_ms: 5000,
    silence_timeout_ms: 500,
    overlap_ms: 220,
    // Signal processing
    target_rms: 0.1,
    max_gain: 8.0,
    noise_gate: 0.005,
}
```

### Custom API Endpoints

TalkToMe supports OpenAI-compatible APIs:
- OpenAI (official)
- Azure OpenAI
- Local Whisper servers
- Custom implementations

## 🐛 Troubleshooting

### Common Issues

1. **No Audio Input**:
   - Check microphone permissions
   - Select correct audio device in settings
   - Test microphone in audio settings

2. **API Errors**:
   - Verify API key is correct
   - Check API endpoint URL
   - Ensure sufficient API credits

3. **Poor Recognition**:
   - Adjust VAD thresholds
   - Check microphone quality
   - Reduce background noise

4. **Performance Issues**:
   - Enable debug logging
   - Check system resources
   - Adjust chunk duration

### Debug Logging

Enable debug logging in Preferences to troubleshoot issues:
- Log location: `%APPDATA%/TalkToMe/logs/`
- Contains detailed pipeline information
- Includes WAV dumps for audio analysis

## 🏆 Project Structure

```
TalkToMe/
├── src/                          # Svelte frontend
│   ├── lib/
│   │   └── stores/
│   │       └── settingsStore.ts  # Settings management
│   ├── routes/                   # Page components
│   │   ├── +layout.svelte        # Main layout
│   │   ├── +page.svelte          # Home page
│   │   ├── preferences/          # Settings pages
│   │   ├── api-settings/
│   │   ├── language-settings/
│   │   ├── audio-settings/
│   │   └── about/
│   └── app.html                  # HTML template
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Main application logic
│   │   ├── audio.rs              # Audio capture & VAD
│   │   ├── stt.rs                # Speech-to-text service
│   │   ├── translation.rs        # Translation service
│   │   ├── text_insertion.rs     # Text insertion utilities
│   │   ├── system_audio.rs       # System audio controls
│   │   ├── settings.rs           # Settings management
│   │   └── debug_logger.rs       # Debug logging system
│   ├── Cargo.toml               # Rust dependencies
│   └── tauri.conf.json          # Tauri configuration
├── static/                      # Static assets
├── package.json                 # Node.js dependencies
└── README.md                    # This file
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Add tests if applicable
5. Commit: `git commit -am 'Add feature'`
6. Push: `git push origin feature-name`
7. Create a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app) - Cross-platform desktop framework
- [Svelte](https://svelte.dev) - Frontend framework
- [OpenAI Whisper](https://openai.com/whisper) - Speech recognition
- [CPAL](https://github.com/RustAudio/cpal) - Cross-platform audio
- [TailwindCSS](https://tailwindcss.com) - Styling framework

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/bgeneto/TalkToMe/issues)
- **Discussions**: [GitHub Discussions](https://github.com/bgeneto/TalkToMe/discussions)
- **Documentation**: [Project Wiki](https://github.com/bgeneto/TalkToMe/wiki)

---

**TalkToMe** - Bridging languages through voice technology 🗣️ → 📝 → 🌍
