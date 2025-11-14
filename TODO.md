# TalkToMe Refactoring - TODO

## Completed ✅
- [x] Persistent settings storage implementation
- [x] Tauri store plugin integration
- [x] Frontend synchronization with persistent store
- [x] Hotkey FSM module with tests
- [x] Documentation in CHANGES.md
- [x] Hotkey FSM integration into hotkey handler
- [x] Frontend FSM state checking and synchronization
- [x] Sync FSM state with recording start/stop commands

## In Progress 🚀

### Phase 3: Code Quality (Priority 3)
- [ ] Run `cargo fmt --all -- --check` and fix formatting
  - **Command**: `cd src-tauri && cargo fmt --all`
  
- [ ] Run `cargo clippy --all -- -D warnings` and address warnings
  - **Command**: `cd src-tauri && cargo clippy --all -- -D warnings`
  - **Current Warnings**:
    - Unused items in `hotkey_fsm.rs` (will be used after integration)
    
- [ ] Remove unused imports and dead code
  - **Current**: None identified yet
  
- [ ] Add unit tests for persistent settings
  - **File**: `src-tauri/src/storage.rs`
  - **Tests Needed**:
    - `test_serialize_deserialize()`
    - `test_load_empty_store()`
    - `test_save_and_load_roundtrip()`

### Phase 4: Settings Validation (Priority 3)
- [ ] Add settings schema validation
  - **File**: `src-tauri/src/storage.rs`
  - **Task**: Validate settings on load, fallback to defaults on invalid data
  - **Details**: Handle corrupted store files gracefully

- [ ] Add settings migration for future schema changes
  - **Task**: Support loading old settings formats
  - **Details**: Version-based migration logic

## Future Enhancements (Lower Priority)

### Audio Chunking Optimization
- [ ] Review chunked recording mode
- **REFACTOR.md Note**: "Think a smart way to properly implement the chunk recording mechanism to avoid submitting a large data request by always using the single recording mode"
- [ ] Consider if chunking should be re-enabled with improvements
- **Note**: Currently always disabled for stability

### Settings Management Features
- [ ] Add settings export/backup functionality
- [ ] Add settings reset-to-defaults button
- [ ] Add settings import from file

### Cross-Platform Testing
- [ ] Test on Linux (currently tested on Windows)
- [ ] Test on macOS (currently tested on Windows)
- [ ] Verify keyring works on all platforms

## Testing Checklist

### Manual Tests
- [ ] Settings persist after app restart
- [ ] Each setting change is saved immediately
- [ ] Corrupted store file doesn't crash app
- [ ] API key remains secure in keyring
- [ ] Default settings used if store is empty
- [ ] Hotkey toggles recording exactly once per press
- [ ] Rapid hotkey presses don't trigger multiple toggles
- [ ] Recording button reflects true state

### Automated Tests
- [ ] `cargo test` passes with no failures
- [ ] `cargo clippy` shows no errors or warnings (except allowed)
- [ ] `cargo fmt --check` passes

### Windows Compatibility
- [ ] App launches successfully
- [ ] Tray icon works correctly
- [ ] Settings persist through restart
- [ ] Hotkeys register without conflicts
- [ ] Audio devices enumerated correctly
- [ ] API key stored in Windows Credential Manager

## Commit Strategy

### Commit 1 (DONE): Foundation
```
feat(settings): add Tauri store plugin for persistent settings storage
```

### Commit 2 (TODO): Hotkey FSM Integration
```
feat(hotkey): implement deterministic state machine for hotkey handling

- Integrate HotkeySM into register_hotkeys command
- Replace debounce-only logic with FSM state transitions
- Ensure single toggle per hotkey press
- Add safeguards against phantom triggers
```

### Commit 3 (TODO): Code Quality
```
chore: apply cargo fmt and fix clippy warnings

- Format all Rust code
- Remove dead code warnings
- Add allow attributes where necessary
```

### Commit 4 (TODO): Tests
```
test: add comprehensive tests for settings and hotkeys

- Add persistent settings roundtrip tests
- Add hotkey FSM integration tests
- Add edge case tests for rapid inputs
```

### Commit 5 (TODO): Documentation
```
docs: update README and API documentation

- Document new persistent settings API
- Document hotkey FSM behavior
- Add troubleshooting section for common issues
```

## Notes

### Important Considerations
1. **Hotkey Registration**: The current `register_hotkeys()` command should be enhanced, not replaced
2. **State Synchronization**: The `RecordingState` variable is shared between hotkey handler and audio manager - ensure consistency
3. **Platform Compatibility**: Test store path on Windows/Linux/macOS
4. **Error Handling**: All store operations should gracefully fallback

### Debug Info
To check persistent settings location:
```bash
# Windows
%APPDATA%/TalkToMe/talktome-settings.json

# Linux
~/.local/share/TalkToMe/talktome-settings.json

# macOS
~/Library/Application Support/TalkToMe/talktome-settings.json
```

### Useful Commands
```bash
# Check for warnings
cargo clippy --all

# Format code
cargo fmt --all

# Run all tests
cargo test --all

# Run specific test module
cargo test hotkey_fsm --lib

# Build for release
cargo build --release
```

## Progress Tracking

| Task | Status | Estimated Effort | Actual Effort | Notes |
|------|--------|------------------|---------------|-------|
| Settings persistence | ✅ Complete | 2h | 2h | Store plugin integrated, frontend synced |
| Hotkey FSM module | ✅ Complete | 1h | 1h | Module created with tests |
| Hotkey FSM integration | ✅ Complete | 1.5h | 1.5h | Integrated into handler, synced with states |
| Frontend FSM checking | ✅ Complete | 0.5h | 0.5h | Added checkFsmState() helper |
| Code quality fixes | ⏳ In Progress | 1h | - | cargo fmt/clippy next |
| Tests & validation | 📋 Planned | 2h | - | Unit + integration tests |
| Documentation | ✅ Complete | 1h | 1h | CHANGES.md and TODO.md created |

---

**Last Updated**: 2025-11-14  
**Next Review**: After hotkey FSM integration  
**Owner**: Refactoring Task (fix-refactor-storage-hotkey branch)
