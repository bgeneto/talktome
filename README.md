# TalkToMe - Voice to Text with Translation

A modern cross-platform desktop application built with Tauri, Svelte, and Tailwind CSS that provides real-time voice-to-text transcription with live translation capabilities.

## Features

- **Real-time Speech Recognition**: Convert speech to text using browser's Web Speech API
- **Live Translation**: Translate transcribed text to multiple languages
- **Cross-platform**: Runs on Windows, macOS, and Linux
- **Modern UI**: Beautiful, responsive interface with dark/light theme support
- **Smart Audio Processing**: Advanced voice recognition with noise filtering
- **Multi-language Support**: Support for 10+ languages for both input and output

## Technology Stack

- **Backend**: Tauri (Rust) for native desktop app framework
- **Frontend**: Svelte with SvelteKit for reactive UI
- **Styling**: Tailwind CSS for modern, responsive design
- **Audio Processing**: Web Audio API integration
- **Speech Recognition**: Browser's native Speech Recognition API

## Prerequisites

Before running this application, make sure you have:

- [Node.js](https://nodejs.org/) (v18 or later)
- [Rust](https://rustup.rs/) (latest stable)
- [Tauri CLI](https://tauri.app/start/prerequisites/)

## Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd TalkToMe
```

2. Install dependencies:
```bash
npm install
```

3. Install Tauri CLI (if not already installed):
```bash
npm install -g @tauri-apps/cli
```

## Development

To run the application in development mode:

```bash
npm run tauri dev
```

This will:
- Start the Vite development server
- Launch the Tauri application
- Enable hot reloading for both frontend and backend

## Building

To build the application for production:

```bash
npm run tauri build
```

This creates platform-specific installers in the `src-tauri/target/release/bundle/` directory.

## Usage

1. **Start Recording**: Click the microphone button to begin voice recording
2. **Select Languages**: Choose source and target languages from the dropdowns
3. **View Results**: See real-time transcription and translation in the text boxes
4. **Copy Text**: Use the copy buttons to copy text to clipboard
5. **Export**: Save transcriptions and translations to text files
6. **Theme Toggle**: Switch between light and dark themes using the header button

## Supported Languages

- Auto Detect
- English
- Spanish
- French
- German
- Italian
- Portuguese
- Russian
- Japanese
- Korean
- Chinese

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Svelte UI     │◄──►│   Tauri Core     │◄──►│  Native APIs    │
│   Components    │    │   (Rust)         │    │  (Audio/File)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│  Tailwind CSS   │    │  Web Audio API   │    │  File System    │
│  Styling        │    │  Speech API      │    │  Integration    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Project Structure

```
project/
├── src/                    # Frontend source code
│   ├── routes/            # Svelte routes
│   ├── app.css           # Global styles
│   └── app.html          # HTML template
├── src-tauri/             # Rust backend
│   ├── src/              # Rust source code
│   ├── Cargo.toml        # Rust dependencies
│   └── tauri.conf.json   # Tauri configuration
├── static/               # Static assets
├── package.json          # Node.js dependencies
└── tailwind.config.js    # Tailwind configuration
```

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/new-feature`
3. Make your changes and commit: `git commit -m 'Add new feature'`
4. Push to the branch: `git push origin feature/new-feature`
5. Submit a pull request

## License

This project is licensed under the MIT License.

## Troubleshooting

### Common Issues

1. **Microphone Access Denied**: Ensure the application has microphone permissions
2. **Speech Recognition Not Working**: Check browser compatibility and microphone setup
3. **Build Errors**: Verify Rust and Node.js versions meet requirements

### Platform-Specific Notes

- **Windows**: May require Visual Studio Build Tools
- **macOS**: Requires Xcode Command Line Tools
- **Linux**: May need additional dependencies for audio processing
