# TalkToMe - AI Coding Assistant Instructions

## Project Overview

**TalkToMe** is a cross-platform Tauri + Rust + Svelte desktop application for real-time speech-to-text transcription and translation. It uses OpenAI Whisper for STT and GPT models for translation, with voice activity detection (VAD) for smart audio processing.

- **Frontend**: Svelte 5.0 + TypeScript + TailwindCSS (SvelteKit)
- **Backend**: Rust (Tauri 2.0) with CPAL for audio capture
- **Storage**: Tauri store plugin + OS keyring for API keys
- **Platform**: Windows (primary), with Linux/macOS support

## Architecture & Data Flow

### Audio Pipeline
1. **Audio Capture** (`src-tauri/src/audio.rs`): CPAL captures microphone input → applies VAD signal conditioning (high-pass filter, AGC, noise gate) → produces AudioChunk via mpsc channel
2. **Hotkey FSM** (`src-tauri/src/hotkey_fsm.rs`): Debounced state machine (Idle ↔ Recording) prevents hotkey bounce on Ctrl+Shift+Space
3. **STT Pipeline**: Audio chunks → OpenAI Whisper API (via `stt.rs`) → raw transcription
4. **Translation**: Transcription → GPT API (via `translation.rs`) → translated text
5. **Text Insertion**: Result → clipboard/system paste (via `text_insertion.rs`, uses `enigo` for typing)

### State Management
- **Backend Settings** (`src-tauri/src/storage.rs`): Persists to Tauri store (`talktome-settings.dat`) using `tauri-plugin-store`
- **API Key** (`src-tauri/src/settings.rs`): **NEVER in localStorage** — stored in OS keyring only (Windows Credential Manager on Windows)
- **Frontend Store** (`src/lib/stores/settingsStore.ts`): Svelte writable store, syncs with backend via `invoke()` commands. API key always empty in frontend
- **Theme**: localStorage (client-side) + TailwindCSS dark class toggle

### Command Pattern (Frontend ↔ Backend)
Frontend calls Rust via `@tauri-apps/api/core#invoke()`:
```typescript
await invoke('command_name', { param: value })
```
Common commands: `start_recording`, `stop_recording`, `save_settings`, `get_settings`, `set_api_key`

## Critical Workflows

### Development Build & Run
```powershell
npm install              # Install Node dependencies
cd src-tauri && cargo build --debug  # Ensure Rust deps cached
cd ..
npm run tauri dev       # Start dev server on localhost:1420 + Tauri window
```
The `beforeDevCommand` in `tauri.conf.json` runs Vite dev server automatically.

### Production Build
```powershell
npm run build          # Build Svelte frontend to ./build
npm run tauri build    # Bundles frontend + Rust → MSI/EXE on Windows
```

### Code Quality Checks (Required Before Commit)
```powershell
cd src-tauri
cargo fmt --all        # Format all Rust code
cargo clippy --all -- -D warnings  # Lint (fail on warnings)
cd ..
npm run check          # Svelte type checking
```

### Hot Reload
- Svelte changes: auto-refresh via Vite (localhost:1420)
- Rust changes: restart `npm run tauri dev` (Tauri watches and rebuilds)

## Project-Specific Patterns

### Settings Persistence Pattern
**CRITICAL**: Settings must persist to Tauri store via backend, not localStorage alone.

1. Frontend changes setting in UI (e.g., API endpoint)
2. Frontend calls backend command: `await invoke('set_setting', { key, value })`
3. Backend calls `SettingsStore::save(app, &updated_settings)` → writes to disk
4. Backend emits event for frontend confirmation (optional)
5. Frontend reads updated value from store on reload

**Anti-pattern**: Relying on localStorage only — will be lost on app restart on some platforms.

### API Key Security Pattern
- **Storage**: OS keyring (keyring crate) only — never in settings struct
- **Retrieval**: `AppSettings::get_api_key(&app_handle)` → reads from Credential Manager on Windows
- **Frontend**: Always `apiKey: ""` in store; prompt user to enter via UI on startup
- **Commands**: API key passed separately: `await invoke('set_api_key', { apiKey })`

### Hotkey Toggle State Machine Pattern
Prevents double-triggers from repeated key presses:
```rust
// In HotkeySM::try_toggle()
if last_toggle < debounce_ms { return Ok(None) }  // Debounce (150ms default)
state = !state  // Deterministic toggle
return Ok(Some(new_state))
```
**Frontend sync**: Maintain `isRecording` state in both backend (HotkeySM) and frontend (Svelte store). Always validate frontend button click against backend state.

### Tauri Command Declaration Pattern
Commands return `Result<T, String>` and are tagged with `#[tauri::command]`:
```rust
#[tauri::command]
fn my_command(state: State<MyState>, param: String) -> Result<String, String> {
    // ... do work
    Ok("result".to_string())
}
```
Register in `run()` via `.invoke_handler(tauri::generate_handler![cmd1, cmd2, ...])`.

### Audio Chunking Disabled by Default
Audio chunking (VAD-based real-time streaming) is **force-disabled** for stability:
- Setting `audio_chunking_enabled: false` in defaults
- Frontend ignores cached chunking value and always disables it
- Single recording mode only: capture entire audio → submit to Whisper in one request
- See REFACTOR.md for future optimization plans

## Key Files & Responsibilities

| File | Purpose |
|------|---------|
| `src-tauri/src/lib.rs` | Main entry point, Tauri commands, hotkey registration, tray menu |
| `src-tauri/src/audio.rs` | CPAL audio capture, VAD signal conditioning, AudioChunk producer |
| `src-tauri/src/hotkey_fsm.rs` | Recording state machine (Idle ↔ Recording) with debounce |
| `src-tauri/src/storage.rs` | Persistent settings (Tauri store) CRUD operations |
| `src-tauri/src/settings.rs` | AppSettings struct, API key keyring operations |
| `src-tauri/src/stt.rs` | OpenAI Whisper API client |
| `src-tauri/src/translation.rs` | OpenAI GPT API client for translation |
| `src/routes/+layout.svelte` | Main layout, theme toggle, navigation sidebar |
| `src/routes/+page.svelte` | Home/recording UI, start/stop button, transcript display |
| `src/routes/api-settings/+page.svelte` | API key input (frontend-only, never stored) |
| `src/routes/preferences/+page.svelte` | Auto-mute, text insertion, hotkey bind UI |
| `src/lib/stores/settingsStore.ts` | Svelte writable store, sync logic with backend |

## Common Tasks

### Adding a New Setting
1. Add field to `PersistentSettings` in `src-tauri/src/storage.rs`
2. Add default value in `impl Default`
3. Add to `Settings` interface in `src/lib/stores/settingsStore.ts`
4. Add UI control in appropriate settings page (e.g., `preferences/+page.svelte`)
5. Create/update Tauri command in `lib.rs` to handle changes
6. Update `settingsStore.ts` to call the command on change

### Debugging Audio Issues
- Enable debug logging: UI → Preferences → Debug Logging (checkbox)
- Check `src-tauri/src/debug_logger.rs` for log output location
- Audio chunks can be dumped to WAV files if enabled (see debug_logger)
- CPAL device list can be queried via `get_audio_devices` command

### Adding a New Language
1. Update language enum/list in UI pages (language-settings, preferences)
2. Update OpenAI model language support in `stt.rs` and `translation.rs`
3. No backend code changes needed (Whisper/GPT handle language dynamically)

## Known Constraints

- **Audio Chunking**: Always disabled (single recording mode). Chunking optimization is a future task.
- **Windows-First**: Develop and test on Windows first; Linux/macOS support may have platform-specific audio or keyring issues.
- **Hotkey Global Scope**: Uses `tauri-plugin-global-shortcut` which requires system permissions on macOS/Linux.
- **API Key Storage**: OS keyring must be available; fails gracefully if keyring service is unavailable.
- **Max Recording**: Limited to configurable minutes (default 2) to prevent accidental long recordings.

## Testing & Validation

### Manual Testing Checklist
- [ ] Start app → confirm default settings loaded
- [ ] Change setting → confirm saved on restart
- [ ] API key prompt → enter key → confirm not shown in settings file
- [ ] Press hotkey (Ctrl+Shift+Space) → single start event (no double-trigger)
- [ ] Press button → confirm button state syncs with hotkey FSM state
- [ ] Record audio → confirm transcription + (optional) translation appears
- [ ] Test on Windows, Linux, macOS if possible

### Before Commit
```powershell
cd src-tauri && cargo test && cargo fmt --all && cargo clippy --all -- -D warnings
npm run check
```

## Recent Refactoring Context

Recent work focused on:
- **Persistent Settings**: Implemented via Tauri store (not localStorage)
- **Hotkey FSM**: Deterministic state machine to fix double-trigger bug
- **API Key Security**: Moved from localStorage → OS keyring
- **Settings Validation**: Graceful fallback to defaults if store is corrupted

See `REFACTOR.md` and `TODO.md` for detailed task breakdown and remaining work.
