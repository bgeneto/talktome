# Crush Agent Configuration for TalkToMe

## Commands
- npm run dev               # Run development server
- npm run build             # Build for production
- npm run check             # Typecheck Svelte files
- npm run tauri dev         # Run Tauri app in development mode
- npm run tauri build       # Build Tauri app for production
- cargo test               # Run Rust unit tests
- cargo check              # Check Rust compilation
- cargo fmt                # Format Rust code
- cargo clippy             # Lint Rust code

## Code Style Guidelines
- Use TypeScript with strict typing
- Follow Svelte best practices
- Apply Tailwind CSS utility classes
- Implement responsive design patterns
- Use Rust idioms and error handling (Result<T, E>)
- Apply serde for serialization/deserialization
- Use async/await with Tokio for non-blocking operations
- Implement proper error boundaries and user feedback
- Store API keys securely using tauri-plugin-stronghold

## File Organization
```bash
# Rust code (backend)
src-tauri/src/

# Svelte components (frontend)
src/

# Configuration
src-tauri/Cargo.toml
src-tauri/tauri.conf.json
package.json
tailwind.config.js
tsconfig.json
```

## Svelte Component Conventions
- Use PascalCase for component names (e.g. SettingsModal.svelte)
- Organize components in src/components/
- Use Svelte stores for state management (src/stores/)
- Follow component composition patterns

## Rust Module Structure
```rust
// Core modules
main.rs          // Entry point
audio/            // Audio capture and processing
services/         // STT and translation services
system/           // System tray and text insertion
hotkeys.rs        // Global hotkey management
settings.rs       // Settings persistence
cmd_processor.rs   // Command processing (future)
```

## Security Practices
- Never log or expose API keys
- Process audio data in memory only
- Validate all API responses
- Use HTTPS for all network requests
- Handle errors gracefully without exposing sensitive data

## Platform-specific Implementation
- Windows: WASAPI for audio control, SendInput for text insertion
- Linux: PulseAudio/PipeWire for audio, xdotool/wl-clipboard for text insertion

## Testing Approach
- Unit tests for Rust modules with cargo test
- Integration tests with mock API endpoints
- Manual testing for audio features and cross-platform behavior
- Accessibility testing with screen readers

## Copilot Instructions (from .github/copilot-instructions.md)
- Follow Tauri + Svelte best practices
- Implement responsive design with Tailwind CSS
- Focus on real-time performance for audio processing
- Ensure cross-platform compatibility (Windows, macOS, Linux)
- Implement proper error handling and user feedback