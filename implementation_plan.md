# Implementation Plan

## Overview
This implementation plan addresses the legacy code refactoring and modernization of the TalkToMe Tauri application. The project currently has critical bugs in settings persistence and hotkey reliability, contains unnecessary complexity, and lacks modern Rust patterns. The plan implements a phased approach to fix bugs, simplify code, and modernize the architecture while maintaining cross-platform compatibility.

The implementation will modernize the codebase with state-of-the-art Rust patterns including proper error handling with thiserror/anyhow, improved state management, and cleaner async code. Settings persistence will be fixed through robust localStorage handling, and hotkey reliability will be improved with better state synchronization.

## Types
No major type system changes are required. The existing Settings interface and AppSettings struct are adequate but will be refined for better type safety.

- **Settings Interface** (frontend): Add strict typing for VAD parameters and validation
- **AppSettings Struct** (backend): Keep existing structure but add builder pattern for construction
- **AudioChunk**: Add duration and metadata fields for better chunk tracking
- **Error Types**: Introduce custom error types for different failure modes (API, Audio, Settings)

## Files
### New Files
- `src-tauri/src/error.rs` - Centralized error handling with thiserror
- `src-tauri/src/state.rs` - Application state management with proper locking
- `src-tauri/src/config.rs` - Configuration management with validation
- `src/lib/stores/persistence.ts` - Enhanced localStorage wrapper with error handling
- `src-tauri/tests/` - Unit and integration tests directory

### Modified Files
- `src-tauri/src/lib.rs` - Refactor main application logic, improve state management
- `src-tauri/src/settings.rs` - Fix API key storage, add validation
- `src-tauri/src/audio.rs` - Simplify audio capture, improve error handling
- `src/lib/stores/settingsStore.ts` - Fix localStorage persistence issues
- `src/routes/+page.svelte` - Improve hotkey and recording state synchronization
- `src-tauri/Cargo.toml` - Add modern dependencies (thiserror, anyhow, tokio-util)

### Deleted Files
- None - all files serve current purposes

### Configuration Updates
- `tauri.conf.json` - Update capabilities for better security
- `package.json` - Update dependencies for better TypeScript support

## Functions
### New Functions
- `settingsStore.ts`: `validateSettings()` - Client-side validation
- `settingsStore.ts`: `migrateLegacySettings()` - Handle old settings format
- `lib.rs`: `handle_hotkey_event()` - Centralized hotkey processing
- `audio.rs`: `validate_audio_config()` - Audio device validation
- `error.rs`: `AppError` variants for different error types

### Modified Functions
- `start_recording()` - Add proper state validation and error recovery
- `stop_recording()` - Improve state synchronization
- `register_hotkeys()` - Add debouncing and state checking
- `settings persistence` - Add error handling and fallbacks

### Removed Functions
- Unused debug functions in lib.rs
- Redundant validation functions
- Legacy migration code that's no longer needed

## Classes
### New Classes
- `AudioManager` - Dedicated class for audio state management
- `SettingsManager` - Handle all settings operations with validation
- `HotkeyManager` - Centralized hotkey registration and handling

### Modified Classes
- `AudioCapture` - Add proper cleanup and error handling
- `STTService` - Improve retry logic and error reporting
- `TranslationService` - Add timeout handling

### Removed Classes
- None - existing classes are functional

## Dependencies
### New Dependencies
- `thiserror = "1.0"` - Modern error handling for Rust
- `anyhow = "1.0"` - Application-level error handling
- `tokio-util = "0.7"` - Additional async utilities
- `serde_with = "3.0"` - Enhanced serialization
- `@types/node` - Better TypeScript support

### Updated Dependencies
- `tauri = "2.0"` - Ensure latest stable
- `tokio = "1.0"` - Latest async runtime
- `serde = "1.0"` - Latest serialization

### Removed Dependencies
- Unused audio processing crates
- Legacy logging dependencies

## Testing
### Test Files
- `src-tauri/src/error_tests.rs` - Error handling tests
- `src-tauri/src/settings_tests.rs` - Settings persistence tests
- `src-tauri/src/audio_tests.rs` - Audio functionality tests
- `src/lib/stores/__tests__/settingsStore.test.ts` - Frontend settings tests

### Existing Test Modifications
- Add integration tests for hotkey functionality
- Add tests for settings persistence across app restarts
- Add audio pipeline tests

### Validation Strategies
- Unit tests for all new error types
- Integration tests for settings persistence
- E2E tests for recording workflow
- Cross-platform testing on Windows/macOS/Linux

## Implementation Order
1. **Phase 2: Critical Bug Fixes**
   - Fix settings persistence in localStorage
   - Implement robust hotkey state management
   - Add proper error handling for API operations

2. **Phase 3: Code Cleanup & Simplification**
   - Remove unused code and functions
   - Simplify complex async logic
   - Clean up redundant validation

3. **Phase 4: Modernization & SOTA Improvements**
   - Implement thiserror/anyhow error handling
   - Add comprehensive test coverage
   - Improve async code with modern patterns
   - Enhance state management

4. **Testing & Validation**
   - Run all tests and fix failures
   - Manual testing of all features
   - Cross-platform validation
