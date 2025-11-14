# Changes and Migration Notes

## Version 0.3.1 - Settings Persistence and Hotkey Improvements

### 🎯 Major Changes

#### Settings Persistence
- **Fixed**: User settings now persist reliably across app restarts
- **Implementation**: Migrated from localStorage-only to Tauri Store plugin for cross-platform reliability
- **Security**: API keys continue to be stored securely via OS keyring (separate from other settings)
- **Migration**: Existing settings are automatically migrated from localStorage to the new persistent store

#### Hotkey System Improvements
- **Fixed**: Deterministic hotkey behavior with proper debouncing and state management
- **Implementation**: Replaced toggle-based hotkeys with explicit start/stop events
- **Reliability**: Eliminated phantom triggers and race conditions between frontend/backend
- **Debounce**: Increased debounce timeout to 200ms to prevent duplicate triggers

### 🔧 Technical Improvements

#### Backend Changes
- **Store Plugin**: Added `tauri-plugin-store` for reliable settings persistence
- **Error Handling**: Replaced `unwrap()` and `expect()` calls with proper error handling
- **State Machine**: Implemented deterministic hotkey state management
- **Permissions**: Added store permissions to `tauri.conf.json`

#### Frontend Changes
- **Settings Store**: Refactored to use backend persistence instead of localStorage
- **Event Handling**: Updated to handle new deterministic hotkey events
- **State Synchronization**: Improved frontend-backend state consistency

### 🧪 Testing
- **Unit Tests**: Added comprehensive tests for settings persistence and hotkey parsing
- **Test Coverage**: Tests cover default settings, JSON serialization, and hotkey validation
- **Hotkey Tests**: Added tests for various hotkey formats and edge cases

### 📋 Manual Verification Steps

#### Settings Persistence
1. Open TalkToMe application
2. Change any setting (e.g., spoken language, model selection)
3. Close and restart the application
4. Verify that your settings are preserved

#### Hotkey Functionality
1. Configure a hotkey (default: Ctrl+Shift+Space)
2. Press the hotkey once - should start recording
3. Press the hotkey again - should stop recording
4. Verify no duplicate start/stop events occur

### 🔄 Migration Notes

#### For Developers
- **Settings API**: The settings persistence API has changed. Frontend code should now use `save_persistent_settings` and `load_persistent_settings` commands.
- **Hotkey Events**: Hotkey events changed from `toggle-recording-from-hotkey` to `start-recording-from-hotkey` and `stop-recording-from-hotkey`.
- **Error Handling**: Backend functions now return `Result` types instead of panicking on errors.

#### For Users
- **No Action Required**: Existing settings will be automatically migrated
- **Hotkeys**: Your existing hotkey configuration will continue to work
- **API Keys**: API keys remain securely stored and are not affected by these changes

### 🛠️ Development Notes

#### Build Requirements
- **Tauri Store Plugin**: Added to `Cargo.toml` dependencies
- **Permissions**: Store permissions added to `tauri.conf.json`
- **Testing**: Test suite can be run with `cargo test` (requires mock setup for full integration tests)

#### Code Quality Improvements
- **Anti-patterns Removed**: Eliminated unsafe `unwrap()` calls
- **Error Propagation**: Proper error handling throughout the codebase
- **Debug Logging**: Removed excessive console.log statements
- **Type Safety**: Improved type checking and validation

### 🔍 Debugging Information

#### Settings Persistence Issues
- Check browser console for "Settings loaded from backend persistent store" message
- Verify `.settings.dat` file exists in app data directory
- Use `debug_api_key_info` command to verify API key storage

#### Hotkey Issues
- Check debug logs for "HOTKEY_PRESSED" and "HOTKEY_STATE_MACHINE" messages
- Verify hotkey registration succeeds on startup
- Check for "HOTKEY_DEBOUNCE" messages if hotkeys seem unresponsive

### 📝 Breaking Changes

- **None**: All changes are backward compatible
- **API**: Internal API changes do not affect user-facing functionality
- **Configuration**: Existing configuration files remain valid

### 🔮 Future Improvements

- **Enhanced Testing**: Integration tests for complete settings persistence flow
- **Performance**: Optimized hotkey debounce timing based on user feedback
- **Cross-platform**: Additional platform-specific hotkey support
- **Settings UI**: Improved settings validation and error reporting

---

## Technical Debt Addressed

1. **Settings Persistence**: Resolved settings not persisting across restarts
2. **Hotkey Reliability**: Fixed deterministic hotkey behavior
3. **Error Handling**: Replaced unsafe unwrap() calls with proper error handling
4. **Code Quality**: Removed anti-patterns and improved type safety
5. **Test Coverage**: Added comprehensive unit tests for critical functionality

## Security Notes

- **API Keys**: Continue to be stored securely via OS keyring
- **Settings Encryption**: Sensitive settings are protected by Tauri's store plugin
- **No Data Leakage**: No sensitive information is logged or persisted insecurely