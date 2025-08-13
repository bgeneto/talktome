# TalkToMe Desktop Application

## Project Overview

You are tasked with building **TalkToMe**, a modern cross-platform voice-to-text desktop application with live translation and smart audio features. This application uses:

- **Backend**: Rust with Tauri framework
- **Frontend**: Svelte with TypeScript and Tailwind CSS
- **Target Platforms**: Windows 10+ and Linux (Ubuntu 20.04+, Fedora 35+, Arch)

## Core Functionality

TalkToMe is a privacy-first desktop application that:

1. **Captures voice input** via configurable microphones
2. **Transcribes speech** using OpenAI-compatible Whisper API endpoints
3. **Translates text** using OpenAI-compatible chat/completions endpoints
4. **Inserts text** directly into any active application
5. **Manages system audio** by auto-muting music during dictation
6. **Provides rich system tray integration** with quick-access menus

## Project Structure

```
talktome/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs
│   │   ├── audio/
│   │   │   ├── mod.rs
│   │   │   ├── capture.rs
│   │   │   └── control.rs
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── stt.rs
│   │   │   └── translation.rs
│   │   ├── system/
│   │   │   ├── mod.rs
│   │   │   ├── tray.rs
│   │   │   └── text_insertion.rs
│   │   ├── hotkeys.rs
│   │   ├── settings.rs
│   │   └── cmd_processor.rs
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── src/
│   ├── App.svelte
│   ├── main.ts
│   ├── app.html
│   ├── stores/
│   │   ├── settings.ts
│   │   ├── dictation.ts
│   │   └── editor.ts
│   ├── components/
│   │   ├── SettingsModal.svelte
│   │   ├── TabGroup.svelte
│   │   ├── MicrophoneSelector.svelte
│   │   ├── LanguageSelector.svelte
│   │   ├── HotkeyInput.svelte
│   │   └── ApiKeyInput.svelte
│   └── styles/
│       └── app.css
├── package.json
├── vite.config.ts
├── tailwind.config.js
├── tsconfig.json
└── README.md
```

## Key Dependencies

### Rust (Cargo.toml)
```toml
[dependencies]
tauri = { version = "2.0", features = ["protocol-asset", "shell-open"] }
tauri-plugin-shell = "2.0"
tauri-plugin-dialog = "2.0"
tauri-plugin-fs = "2.0"
tauri-plugin-global-shortcut = "2.0"
tauri-plugin-notification = "2.0"
tauri-plugin-stronghold = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json", "multipart"] }
cpal = "0.15"
anyhow = "1.0"
thiserror = "1.0"

[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
    "Win32_Foundation",
    "Win32_System_Com",
    "Win32_Media_Audio",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_UI_WindowsAndMessaging"
]}

[target.'cfg(unix)'.dependencies]
x11 = { version = "2.21", features = ["xlib", "xtest"] }
libpulse-binding = "2.27"
```

### Frontend (package.json)
```json
{
  "devDependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "@tauri-apps/plugin-fs": "^2.0.0",
    "@tauri-apps/plugin-global-shortcut": "^2.0.0",
    "@tauri-apps/plugin-notification": "^2.0.0",
    "@tauri-apps/plugin-stronghold": "^2.0.0",
    "@tauri-apps/plugin-tray": "^2.0.0",
    "@sveltejs/vite-plugin-svelte": "^3.0.0",
    "svelte": "^4.2.0",
    "typescript": "^5.2.0",
    "vite": "^5.0.0",
    "tailwindcss": "^3.3.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.4.0"
  }
}
```

## Critical Features Implementation

### 1. System Tray with Rich Context Menu

The system tray must include a hierarchical right-click menu:

```
TalkToMe
├── About
├── Open Settings
├── ─────────────
├── Change Microphone ►
│   ├── Default Microphone
│   ├── USB Microphone (Yeti) ✓
│   └── Bluetooth Headset
├── Select Language ►
│   ├── English ✓
│   ├── Spanish
│   └── Portuguese
├── Translation Language ►
│   ├── English
│   ├── Spanish ✓
│   ├── Portuguese
│   └── Disable Translation
├── ─────────────
└── Quit
```

### 2. Smart Audio Management

Implement platform-specific audio control:
- **Windows**: Use WASAPI to detect and mute audio sessions
- **Linux**: Use PulseAudio/PipeWire APIs to control audio streams
- **Configurable**: Toggle in settings, restore previous state after dictation

### 3. Universal Text Insertion

Cross-platform text insertion into any active application:
- **Windows**: Use `SendInput` API and clipboard simulation
- **Linux**: Use `xdotool` for X11 or `wl-clipboard` for Wayland
- **Smart detection**: Identify password fields and handle appropriately

### 4. OpenAI-Compatible API Integration

Two distinct API endpoints:
- **STT Endpoint**: `/v1/audio/transcriptions` (Whisper-compatible)
- **Translation Endpoint**: `/v1/chat/completions` (GPT-compatible)

### 5. Global Hotkeys

Three configurable global shortcuts:
- **Push-to-Talk**: Hold while speaking (default: `Ctrl+Win`)
- **Hands-Free Toggle**: Toggle continuous listening (default: `Ctrl+Win+Space`)
- **Emergency Stop**: Immediately stop all dictation (default: `Escape`)

## Development Guidelines

### Code Organization

1. **Separation of Concerns**: Keep audio, networking, system integration, and UI logic separate
2. **Error Handling**: Use `Result<T, E>` extensively, provide meaningful error messages
3. **Async/Await**: Use Tokio for all async operations, avoid blocking the main thread
4. **State Management**: Use Tauri's state management for shared resources
5. **Event System**: Use Tauri's event system for communication between Rust and frontend

### Security Requirements

1. **API Keys**: Store using `tauri-plugin-stronghold`, never log or expose
2. **Audio Data**: Process in memory only, never write to disk
3. **Network**: HTTPS only, validate certificates, handle timeouts
4. **Permissions**: Request minimal required system permissions

### Error Handling Strategy

1. **Graceful Degradation**: If translation fails, fall back to transcription only
2. **User Feedback**: Show clear error messages and recovery suggestions
3. **Logging**: Log errors for debugging but never log sensitive data
4. **Recovery**: Implement retry logic with exponential backoff

### Platform-Specific Implementation

#### Windows-Specific Features
- Use Windows APIs for audio session management
- Implement proper Windows installer (MSI)
- Handle Windows-specific hotkey registration

#### Linux-Specific Features
- Support both X11 and Wayland
- Handle PulseAudio and PipeWire audio systems
- Create AppImage, .deb, and .rpm packages

## Settings Schema

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    // UI Settings
    pub theme: String, // "light" | "dark" | "system"
    
    // Audio Settings
    pub audio: AudioConfig,
    
    // API Configuration
    pub api: ApiConfig,
    
    // Translation Settings
    pub translation: TranslationConfig,
    
    // Hotkey Configuration
    pub hotkeys: HotkeyConfig,
    
    // System Tray Configuration
    pub tray: TrayConfig,
    
    // Behavior Settings
    pub dictation_language: String,
    pub privacy_mode: bool,
    pub text_insertion_enabled: bool,
    pub auto_save_enabled: bool,
}
```

## Event Flow

### Dictation Flow
1. User presses hotkey → Global hotkey manager catches event
2. Audio control manager mutes system audio (if enabled)
3. Audio capture starts recording from selected microphone
4. Audio data sent to STT service (Whisper API)
5. If translation enabled, text sent to translation service
6. Final text inserted into active application
7. Audio control manager restores system audio

### Settings Management
1. Settings loaded from JSON file on startup
2. API keys loaded from Stronghold secure storage
3. Settings changes trigger immediate application of new configuration
4. Settings automatically saved on change

## Testing Requirements

1. **Unit Tests**: Test all Rust modules with mock dependencies
2. **Integration Tests**: Test API interactions with mock servers
3. **Platform Tests**: Automated testing on Windows and Linux
4. **UI Tests**: Test Svelte components and user interactions
5. **Manual Tests**: Audio functionality, system integration, accessibility

## Build and Deployment

### Development Setup
```bash
# Install Rust and Node.js dependencies
npm install
cargo install tauri-cli

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Release Process
1. Create tagged release on GitHub
2. Automated CI/CD builds for Windows and Linux
3. Generate signed installers and packages
4. Upload release assets to GitHub Releases

## Performance Requirements

- **Audio Latency**: < 100ms for local processing
- **API Response**: < 2000ms for transcription + translation
- **Memory Usage**: < 150MB during normal operation
- **CPU Usage**: Minimal when idle, efficient during processing
- **Text Insertion**: < 50ms latency

## Accessibility Requirements

- Full keyboard navigation support
- Screen reader compatibility
- High contrast mode support
- Auto-adapt to OS system theme (dark/light)
- Configurable text sizes and UI scaling
- Clear visual indicators for all states

---

## Getting Started Instructions

When implementing this project:

1. **Start with project scaffolding**: Set up Tauri + Svelte (typescript) + Tailwind project structure
2. **Implement core settings system**: Settings persistence and secure API key storage
3. **Build audio foundation**: Microphone selection and audio capture
4. **Add system tray**: Basic tray with simple menu structure
5. **Implement global hotkeys**: Hotkey registration and event handling
6. **Create STT service**: OpenAI-compatible Whisper API integration
7. **Add translation service**: Chat/completions API for translation
8. **Implement text insertion**: Platform-specific text insertion services
9. **Build comprehensive UI**: Settings modal with all configuration options
10. **Add advanced features**: Audio control, rich tray menus, command mode

Focus on creating a robust, user-friendly application that handles edge cases gracefully and provides excellent cross-platform compatibility.