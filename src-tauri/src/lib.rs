use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder},
    Manager, Emitter, AppHandle, State,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState, GlobalShortcutExt};
use tauri_plugin_notification::NotificationExt;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
// Global last-audio-manager error for diagnostics (frontend can query this)
static AUDIO_MANAGER_LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);
use std::sync::mpsc as std_mpsc;
// no additional thread/state for AudioCapture; it's not Send
mod settings;
use settings::AppSettings;
mod audio;
use audio::AudioCapture;
mod stt;
use stt::STTService;
mod translation;
use translation::TranslationService;
mod text_insertion;
use text_insertion::TextInsertionService;
mod system_audio;
use system_audio::SystemAudioControl;
mod debug_logger;
use debug_logger::DebugLogger;
mod storage;
use storage::SettingsStore;
mod hotkey_fsm;
use hotkey_fsm::HotkeySM;

// Global state to track registered hotkeys and active recording
type HotkeyRegistry = Mutex<HashMap<String, String>>;
type RecordingState = Arc<Mutex<bool>>;
type AudioStopSender = Arc<Mutex<Option<std::sync::mpsc::Sender<()>>>>;
// Track last stop timestamp to avoid rapid duplicate stops (cooldown)
type LastStopTime = Arc<Mutex<Option<std::time::Instant>>>;
// Track the last hotkey action and when it happened to help debug stop origins
type LastHotkey = Arc<Mutex<Option<(String, std::time::Instant)>>>;
// FSM for recording state with debouncing
type HotkeySMState = Arc<HotkeySM>;

// Commands sent to the single-threaded audio manager which owns the non-Send AudioCapture
enum AudioManagerCommand {
    Start {
        // reply channel to send back the audio chunk receiver or error
    reply: std_mpsc::Sender<Result<std_mpsc::Receiver<crate::audio::AudioChunk>, String>>,
    // Whether frontend requested real-time chunking (VAD). If false, capture should operate in passthrough
    audio_chunking_enabled: bool,
    },
    Stop {
        // optional reply to acknowledge stop
        reply: Option<std_mpsc::Sender<Result<(), String>>>,
    },
}

// Arc+Mutex wrapper so we can store the command sender in Tauri managed state
type AudioManagerHandle = Arc<Mutex<std_mpsc::Sender<AudioManagerCommand>>>;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Parse hotkey string to Shortcut struct
fn parse_hotkey(hotkey: &str) -> Result<Shortcut, String> {
    let parts: Vec<&str> = hotkey.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return Err("Empty hotkey".to_string());
    }
    
    let mut modifiers = Modifiers::empty();
    let mut key_code = None;
    
    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            "win" | "super" | "cmd" | "meta" => modifiers |= Modifiers::SUPER,
            key => {
                // Try to parse the key
                let code = match key.to_lowercase().as_str() {
                    // Special keys
                    "space" => Code::Space,
                    "escape" | "esc" => Code::Escape,
                    "enter" | "return" => Code::Enter,
                    "backspace" => Code::Backspace,
                    "tab" => Code::Tab,
                    "delete" | "del" => Code::Delete,
                    "insert" | "ins" => Code::Insert,
                    "home" => Code::Home,
                    "end" => Code::End,
                    "pageup" | "pgup" => Code::PageUp,
                    "pagedown" | "pgdn" => Code::PageDown,
                    "arrowup" | "up" => Code::ArrowUp,
                    "arrowdown" | "down" => Code::ArrowDown,
                    "arrowleft" | "left" => Code::ArrowLeft,
                    "arrowright" | "right" => Code::ArrowRight,
                    "capslock" => Code::CapsLock,
                    "numlock" => Code::NumLock,
                    "scrolllock" => Code::ScrollLock,
                    "printscreen" | "prtsc" => Code::PrintScreen,
                    "pause" => Code::Pause,
                    "contextmenu" | "menu" => Code::ContextMenu,
                    // Function keys
                    "f1" => Code::F1,
                    "f2" => Code::F2,
                    "f3" => Code::F3,
                    "f4" => Code::F4,
                    "f5" => Code::F5,
                    "f6" => Code::F6,
                    "f7" => Code::F7,
                    "f8" => Code::F8,
                    "f9" => Code::F9,
                    "f10" => Code::F10,
                    "f11" => Code::F11,
                    "f12" => Code::F12,
                    "f13" => Code::F13,
                    "f14" => Code::F14,
                    "f15" => Code::F15,
                    "f16" => Code::F16,
                    "f17" => Code::F17,
                    "f18" => Code::F18,
                    "f19" => Code::F19,
                    "f20" => Code::F20,
                    "f21" => Code::F21,
                    "f22" => Code::F22,
                    "f23" => Code::F23,
                    "f24" => Code::F24,
                    // Letter keys
                    "a" => Code::KeyA,
                    "b" => Code::KeyB,
                    "c" => Code::KeyC,
                    "d" => Code::KeyD,
                    "e" => Code::KeyE,
                    "f" => Code::KeyF,
                    "g" => Code::KeyG,
                    "h" => Code::KeyH,
                    "i" => Code::KeyI,
                    "j" => Code::KeyJ,
                    "k" => Code::KeyK,
                    "l" => Code::KeyL,
                    "m" => Code::KeyM,
                    "n" => Code::KeyN,
                    "o" => Code::KeyO,
                    "p" => Code::KeyP,
                    "q" => Code::KeyQ,
                    "r" => Code::KeyR,
                    "s" => Code::KeyS,
                    "t" => Code::KeyT,
                    "u" => Code::KeyU,
                    "v" => Code::KeyV,
                    "w" => Code::KeyW,
                    "x" => Code::KeyX,
                    "y" => Code::KeyY,
                    "z" => Code::KeyZ,
                    // Number keys
                    "0" => Code::Digit0,
                    "1" => Code::Digit1,
                    "2" => Code::Digit2,
                    "3" => Code::Digit3,
                    "4" => Code::Digit4,
                    "5" => Code::Digit5,
                    "6" => Code::Digit6,
                    "7" => Code::Digit7,
                    "8" => Code::Digit8,
                    "9" => Code::Digit9,
                    // Numpad keys
                    "numpad0" => Code::Numpad0,
                    "numpad1" => Code::Numpad1,
                    "numpad2" => Code::Numpad2,
                    "numpad3" => Code::Numpad3,
                    "numpad4" => Code::Numpad4,
                    "numpad5" => Code::Numpad5,
                    "numpad6" => Code::Numpad6,
                    "numpad7" => Code::Numpad7,
                    "numpad8" => Code::Numpad8,
                    "numpad9" => Code::Numpad9,
                    "numpadmultiply" | "numpadadd" | "numpadsubtract" | "numpaddecimal" | "numpaddivide" => {
                        match key.to_lowercase().as_str() {
                            "numpadmultiply" => Code::NumpadMultiply,
                            "numpadadd" => Code::NumpadAdd,
                            "numpadsubtract" => Code::NumpadSubtract,
                            "numpaddecimal" => Code::NumpadDecimal,
                            "numpaddivide" => Code::NumpadDivide,
                            _ => return Err(format!("Unsupported numpad key: {}", key)),
                        }
                    }
                    // Symbol keys
                    "minus" | "-" => Code::Minus,
                    "equal" | "=" => Code::Equal,
                    "bracketleft" | "[" => Code::BracketLeft,
                    "bracketright" | "]" => Code::BracketRight,
                    "semicolon" | ";" => Code::Semicolon,
                    "quote" | "'" => Code::Quote,
                    "backquote" | "`" => Code::Backquote,
                    "comma" | "," => Code::Comma,
                    "period" | "." => Code::Period,
                    "slash" | "/" => Code::Slash,
                    "backslash" | "\\" => Code::Backslash,
                    _ => return Err(format!("Unsupported key: {}", key)),
                };
                key_code = Some(code);
                break;
            }
        }
    }
    
    // Handle modifier-only combinations
    // For combinations like Ctrl+Shift+Space or Shift+Ctrl+Alt, we need to use a placeholder key
    // We'll use a key that's unlikely to conflict with normal usage
    if key_code.is_none() {
        // Check if we have valid modifier combinations
        if !modifiers.is_empty() {
            // Use F24 as a placeholder key for modifier-only combinations
            // F24 is rarely used and should work well as a placeholder
            key_code = Some(Code::F24);
            DebugLogger::log_info(&format!("Using F24 as placeholder for modifier-only combination: {:?}", modifiers));
        } else {
            return Err("No modifiers or keys specified in hotkey".to_string());
        }
    }
    
    let code = key_code.ok_or_else(|| "No key specified in hotkey".to_string())?;
    Ok(Shortcut::new(Some(modifiers), code))
}

/// Get last audio manager error (for diagnostics)
#[tauri::command]
fn get_audio_manager_last_error() -> Option<String> {
    if let Ok(err) = AUDIO_MANAGER_LAST_ERROR.lock() {
        err.clone()
    } else {
        None
    }
}

/// Clear the last audio manager error
#[tauri::command]
fn clear_audio_manager_last_error() {
    if let Ok(mut err) = AUDIO_MANAGER_LAST_ERROR.lock() {
        *err = None;
    }
}

/// Test hotkey parsing (for debugging)
#[tauri::command]
fn test_hotkey_parsing(hotkey: String) -> Result<String, String> {
    match parse_hotkey(&hotkey) {
        Ok(shortcut) => {
            Ok(format!("Successfully parsed hotkey '{}': {:?}", hotkey, shortcut))
        }
        Err(e) => {
            Err(format!("Failed to parse hotkey '{}': {}", hotkey, e))
        }
    }
}

// Command to register hotkeys
#[tauri::command]
async fn register_hotkeys(
    app: AppHandle,
    hotkeys: std::collections::HashMap<String, String>,
    registry: State<'_, HotkeyRegistry>,
) -> Result<(), String> {
    let global_shortcut = app.global_shortcut();
    DebugLogger::log_info(&format!("register_hotkeys called, hotkeys_count={}", hotkeys.len()));
    
    // Log each hotkey being registered
    for (action, hotkey_str) in &hotkeys {
        DebugLogger::log_info(&format!("Attempting to register hotkey: action='{}', hotkey='{}'", action, hotkey_str));
    }
    
    // Unregister existing hotkeys
    {
        let mut reg = registry.lock().unwrap();
        for (_, hotkey_str) in reg.iter() {
            if let Ok(shortcut) = parse_hotkey(hotkey_str) {
                let _ = global_shortcut.unregister(shortcut);
            }
        }
        reg.clear();
    }
    
    // Register new hotkeys
    for (action, hotkey_str) in &hotkeys {
        if hotkey_str.is_empty() {
            continue;
        }
        
        let shortcut = parse_hotkey(hotkey_str).map_err(|e| {
            let error_msg = format!("Failed to parse hotkey '{}' for action '{}': {}", hotkey_str, action, e);
            DebugLogger::log_info(&error_msg);
            error_msg
        })?;
        
        DebugLogger::log_info(&format!("Successfully parsed hotkey '{}' for action '{}': {:?}", hotkey_str, action, shortcut));
        
        // Register handler to emit an event when the shortcut is triggered
        let action_clone = action.clone();
        let app_for_emit = app.clone();
        global_shortcut
            .on_shortcut(shortcut, move |app_handle, _sc, ev| {
                let ts_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0);

                // Normalize action names to support both camelCase and snake_case
                let normalized = match action_clone.as_str() {
                    "handsFree" | "hands_free" => "hands_free",
                    other => other,
                };

                match (normalized, ev.state) {
                    // Hands-free: Only process key press (ignore release)
                    ("hands_free", ShortcutState::Pressed) => {
                        // Use FSM to toggle state with debouncing
                        if let Some(fsm) = app_handle.try_state::<HotkeySMState>() {
                            match fsm.try_toggle() {
                                Ok(Some(new_state)) => {
                                    DebugLogger::log_info(&format!(
                                        "HOTKEY_FSM_TOGGLE: action=hands_free, new_state={:?}, ts_ms={}",
                                        new_state, ts_ms
                                    ));
                                    let _ = app_for_emit.emit("toggle-recording-from-hotkey", ());
                                }
                                Ok(None) => {
                                    DebugLogger::log_info(&format!(
                                        "HOTKEY_FSM_DEBOUNCED: action=hands_free, ts_ms={}",
                                        ts_ms
                                    ));
                                }
                                Err(e) => {
                                    DebugLogger::log_pipeline_error(
                                        "hotkey_fsm",
                                        &format!("FSM error: {}", e),
                                    );
                                }
                            }
                        } else {
                            DebugLogger::log_info("FSM not available, fallback to event emit");
                            let _ = app_for_emit.emit("toggle-recording-from-hotkey", ());
                        }
                    }
                    _ => {
                        let state = match ev.state {
                            ShortcutState::Pressed => "pressed",
                            ShortcutState::Released => "released",
                        };
                        let _ = app_for_emit.emit(
                            "hotkey-triggered",
                            serde_json::json!({ "action": action_clone, "state": state }),
                        );
                    }
                }
            })
            .map_err(|e| {
                format!(
                    "Failed to attach handler for hotkey '{}' (action '{}'): {}",
                    hotkey_str, action, e
                )
            })?;
    }
    
    // Update registry
    {
        let mut reg = registry.lock().unwrap();
        *reg = hotkeys;
    }
    
    Ok(())
}

// Command to show recording started notification
#[tauri::command]
async fn show_recording_started_notification(
    app: AppHandle,
    recording_state: State<'_, RecordingState>
) -> Result<(), String> {
    // Check if we should actually show notification (prevent showing when already recording)
    {
        let state = recording_state.inner().lock().map_err(|e| e.to_string())?;
        if *state {
            DebugLogger::log_info("show_recording_started_notification: Already recording, skipping notification display");
            return Ok(()); // Don't show notification if already recording
        }
    }
    
    DebugLogger::log_info("Showing recording started notification");
    
    app.notification()
        .builder()
        .title("Recording Started")
        .body("🎤 Listening for speech...")
        .show()
        .map_err(|e| e.to_string())?;
        
    Ok(())
}

// Command to show recording stopped notification
#[tauri::command]
async fn show_recording_stopped_notification(
    app: AppHandle,
    _recording_state: State<'_, RecordingState>
) -> Result<(), String> {
    DebugLogger::log_info("Showing recording stopped notification");

    app.notification()
        .builder()
        .title("Recording Stopped")
        .body("⏳ Processing audio...")
        .show()
        .map_err(|e| e.to_string())?;

    Ok(())
}

// Command to start recording
#[tauri::command]
async fn start_recording(
    app: AppHandle,
    recording_state: State<'_, RecordingState>,
    audio_stop_sender: State<'_, AudioStopSender>,
    audio_manager: State<'_, AudioManagerHandle>,
    fsm: State<'_, HotkeySMState>,

    spoken_language: String,
    translation_language: String,
    api_endpoint: String,
    stt_model: String,
    auto_mute: bool,
    translation_enabled: bool,
    translation_model: String,
    text_insertion_enabled: bool,
    audio_chunking_enabled: bool,
    max_recording_time_minutes: u32,
    debug_logging: bool
) -> Result<(), String> {
    // Check if already recording
    {
        let state = recording_state.inner().lock().map_err(|e| e.to_string())?;
        if *state {
            DebugLogger::log_info("start_recording called but already recording - rejecting duplicate start");
            return Err("Already recording".to_string());
        }
    }

    // Get API key (use default AppSettings instance for the method)
    DebugLogger::log_info("=== PIPELINE START: start_recording() called ===");
    DebugLogger::log_info(&format!("Recording params: spoken_lang={}, translation_lang={}, endpoint={}, stt_model={}, auto_mute={}, translation_enabled={}, text_insertion_enabled={}, audio_chunking_enabled={}, debug_logging={}", 
        spoken_language, translation_language, api_endpoint, stt_model, auto_mute, translation_enabled, text_insertion_enabled, audio_chunking_enabled, debug_logging));
    
    // Update debug logging state to match the frontend preference
    DebugLogger::init_with_state(&app, debug_logging)?;
    DebugLogger::log_info(&format!("Debug logging state updated to: {}", debug_logging));
    
    let settings_for_api = AppSettings::default();
    let api_key = settings_for_api.get_api_key(&app).map_err(|e| {
        let error_msg = format!("Failed to get API key: {}", e);
        DebugLogger::log_pipeline_error("settings", &error_msg);
        error_msg
    })?;
    DebugLogger::log_info(&format!("API key obtained, length: {} chars", api_key.len()));
    
    // Create a settings struct for the processing pipeline
    let settings = AppSettings {
        spoken_language,
        translation_language,
        audio_device: "default".to_string(), // Not used in recording
        theme: "auto".to_string(), // Not used in recording
        auto_save: true, // Not used in recording
        api_endpoint,
        stt_model,
        translation_model: translation_model.clone(),
        hotkeys: crate::settings::Hotkeys {
            hands_free: "".to_string(), // Not used in recording
        },
        auto_mute,
        translation_enabled,
        debug_logging, // Use the value passed from frontend
        text_insertion_enabled,
        audio_chunking_enabled,
        max_recording_time_minutes,
    };
    
    // Request the audio manager (single-thread owner) to start capture and return the receiver
    DebugLogger::log_info("Requesting audio manager to start capture");
    let (reply_tx, reply_rx) = std_mpsc::channel();
    {
        let sender = audio_manager.lock().map_err(|e| e.to_string())?;
        sender.send(AudioManagerCommand::Start { reply: reply_tx, audio_chunking_enabled }).map_err(|e| {
            let msg = format!("Failed to send start command to audio manager: {}", e);
            DebugLogger::log_pipeline_error("audio_manager", &msg);
            msg
        })?;
    }
    // Wait for manager to reply with the audio receiver
    let audio_rx = match reply_rx.recv_timeout(std::time::Duration::from_secs(5)) {
    Ok(Ok(rx)) => rx,
    Ok(Err(e)) => {
            DebugLogger::log_pipeline_error("audio_manager", &e);
            return Err(e);
        }
        Err(e) => {
            let msg = format!("Timed out waiting for audio manager start reply: {}", e);
            DebugLogger::log_pipeline_error("audio_manager", &msg);
            return Err(msg);
        }
    };
    DebugLogger::log_info("Audio capture started successfully (owned by audio manager thread)");
    
    // Track recording start time for timeout monitoring
    let recording_start_time = std::time::Instant::now();
    let max_recording_duration = std::time::Duration::from_secs((max_recording_time_minutes as u64) * 60);
    DebugLogger::log_info(&format!("Recording timeout set to {} minutes", max_recording_time_minutes));
    
    // Set recording state to true
    {
        let mut state = recording_state.inner().lock().map_err(|e| e.to_string())?;
        *state = true;
        DebugLogger::log_info("RECORDING_STATE_CHANGE: Set to true in start_recording (recording started)");
    }

    // Update FSM to Recording state
    fsm.force_set_state(hotkey_fsm::RecordingState::Recording)
        .unwrap_or_else(|e| DebugLogger::log_info(&format!("Failed to set FSM to Recording: {}", e)));

    // Show "Recording Started" notification
    DebugLogger::log_info("Showing recording started notification");
    let _ = app.notification()
        .builder()
        .title("Recording Started")
        .body("🎤 Listening for speech...")
        .show();

    // Emit recording-started event to frontend to ensure state synchronization
    DebugLogger::log_info("Emitting recording-started event to frontend");
    let _ = app.emit("recording-started", ());

    // Create stop channel for proper audio cleanup
    let (stop_tx, stop_rx) = std::sync::mpsc::channel();
    
    // Store the stop sender in global state so stop_recording can use it
    {
        let mut audio_stop = audio_stop_sender.inner().lock().map_err(|e| e.to_string())?;
        *audio_stop = Some(stop_tx);
        DebugLogger::log_info("Audio stop sender stored in global state");
    }

    // Keep the audio_capture alive (non-Send) until pipeline stops
    
    // Create services with API key
    DebugLogger::log_info("Creating STT service");
    let stt_service = STTService::new(
        settings.api_endpoint.clone(),
        api_key.clone(),
        settings.stt_model.clone(),
        settings.spoken_language.clone(),
    );
    DebugLogger::log_info(&format!("STT service created with endpoint: {} and model: {}", settings.api_endpoint, settings.stt_model));
    
    let translation_service = if settings.translation_enabled && settings.translation_language != "none" {
        DebugLogger::log_info("Creating translation service (translation enabled)");
        Some(TranslationService::new(settings.api_endpoint.clone(), api_key, settings.translation_model.clone()))
    } else {
        // Always create translation service for text correction
        DebugLogger::log_info("Creating translation service (text correction only)");
        Some(TranslationService::new(settings.api_endpoint.clone(), api_key, settings.translation_model.clone()))
    };
    DebugLogger::log_info("Translation service created");
    
    DebugLogger::log_info("Creating text insertion service");
    let text_insertion_service = std::sync::Arc::new(TextInsertionService::new());
    DebugLogger::log_info("Text insertion service created");
    // Create a non-blocking background worker for text insertion so the audio
    // pipeline never blocks on platform typing utilities (PowerShell/xdotool/etc.).
    let (text_insertion_tx, mut text_insertion_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    // Control channel for the worker to notify when insertion starts/ends
    let (insertion_ctrl_tx, mut _insertion_ctrl_rx) = tokio::sync::mpsc::unbounded_channel::<bool>();

    // Spawn a dedicated background task that performs the blocking insertions
    // using spawn_blocking so it doesn't block the Tokio runtime.
    let text_insertion_service_for_worker = text_insertion_service.clone();
    let insertion_ctrl_tx_for_worker = insertion_ctrl_tx.clone();
    tokio::spawn(async move {
        DebugLogger::log_info("TEXT_INSERTION_WORKER: started");
        while let Some(text) = text_insertion_rx.recv().await {
            DebugLogger::log_info(&format!("TEXT_INSERTION_WORKER: received text (len={}) to insert", text.len()));
            // Signal insertion start
            let _ = insertion_ctrl_tx_for_worker.send(true);
            
            let svc = text_insertion_service_for_worker.clone();
            let t = text.clone();
            // Run the platform Command in a blocking thread pool
            let res = tokio::task::spawn_blocking(move || svc.insert_text(&t)).await;
            match res {
                Ok(Ok(())) => DebugLogger::log_info("TEXT_INSERTION_WORKER: insertion succeeded"),
                Ok(Err(e)) => DebugLogger::log_pipeline_error("text_insertion_worker", &format!("insertion error: {}", e)),
                Err(e) => DebugLogger::log_pipeline_error("text_insertion_worker", &format!("spawn_blocking failed: {}", e)),
            }
            // Signal insertion complete
            let _ = insertion_ctrl_tx_for_worker.send(false);
        }
        DebugLogger::log_info("TEXT_INSERTION_WORKER: exiting (sender closed)");
    });
    
    // Clone values for the async task
    let app_clone = app.clone();
    let recording_state_clone = recording_state.inner().clone();
    let auto_mute = settings.auto_mute;
    
    // Spawn task to process audio chunks and monitor stop signal
    tokio::spawn(async move {
        // Create system audio control inside the task for auto-mute if enabled
        DebugLogger::log_info(&format!("Auto-mute setting: {}", auto_mute));
        let audio_control = if auto_mute {
            DebugLogger::log_info("Attempting to create system audio control for auto-mute");
            match SystemAudioControl::new() {
                Ok(control) => {
                    DebugLogger::log_info("System audio control created successfully");
                    // Mute system audio
                    if let Err(e) = control.mute_system_audio() {
                        let error_msg = format!("Failed to mute system audio: {}", e);
                        eprintln!("{}", error_msg);
                        DebugLogger::log_pipeline_error("system_audio", &error_msg);
                    } else {
                        DebugLogger::log_info("System audio muted successfully");
                    }
                    Some(control)
                },
                Err(e) => {
                    let error_msg = format!("Failed to initialize system audio control: {}", e);
                    eprintln!("{}", error_msg);
                    DebugLogger::log_pipeline_error("system_audio", &error_msg);
                    None
                }
            }
        } else {
            DebugLogger::log_info("Auto-mute disabled, not creating system audio control");
            None
        };
        
    let stt_service = stt_service;
    let translation_service = translation_service;
    let app = app_clone;
    let settings = settings;
        
        DebugLogger::log_info("Starting audio processing pipeline");
        DebugLogger::log_info(&format!("Pipeline settings: translation_enabled={}, spoken_lang={}, target_lang={}", 
            settings.translation_enabled, settings.spoken_language, settings.translation_language));
        
        DebugLogger::log_info("About to enter audio processing pipeline");
        DebugLogger::log_info(&format!("Audio chunking mode: {}", if settings.audio_chunking_enabled { "ENABLED (real-time chunks)" } else { "DISABLED (single recording)" }));
        
        if settings.audio_chunking_enabled {
            // === CHUNKED MODE: Real-time processing ===
            DebugLogger::log_info("Waiting for first audio chunk...");
            
            // Move audio_rx into chunked mode
            let audio_rx = audio_rx;
            
            // Aggregation state: accumulate text until recording stops
            use std::time::Duration;
            let mut agg_text = String::new();

            fn append_dedup(agg: &mut String, next: &str) {
                // Token-aware suffix/prefix dedup: use last up to 12 chars as heuristic
                let take = agg.chars().rev().take(12).collect::<String>();
                let tail: String = take.chars().rev().collect();
                if !tail.is_empty() && next.starts_with(&tail) {
                    agg.push_str(&next[tail.len()..]);
                } else {
                    if !agg.is_empty() { agg.push(' '); }
                    agg.push_str(next);
                }
            }

            // Process audio chunks with timeout to detect stop/idle
            loop {
                use std::sync::mpsc::RecvTimeoutError;
            
            // Check stop signal first
            match stop_rx.try_recv() {
                Ok(_) => {
                    DebugLogger::log_info("STOP_REASON: Stop signal received manually, breaking processing loop");
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    DebugLogger::log_info("STOP_REASON: Stop signal channel disconnected (audio system failure), breaking processing loop");
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // No stop signal, continue processing
                }
            }
            
            let audio_chunk = match audio_rx.recv_timeout(Duration::from_millis(200)) {
                Ok(chunk) => chunk,
                Err(RecvTimeoutError::Timeout) => {
                    // Periodically check for stop
                    let stop = {
                        let state = recording_state_clone.lock().unwrap();
                        !*state
                    };
                    if stop {
                        DebugLogger::log_info("STOP_REASON: Recording state set to false (timeout check), breaking processing loop");
                        break;
                    }
                    
                    // Check if recording has exceeded max time limit
                    if recording_start_time.elapsed() >= max_recording_duration {
                        DebugLogger::log_info(&format!("STOP_REASON: Recording exceeded maximum time limit of {} minutes", max_recording_time_minutes));
                        
                        // Set recording state to false
                        {
                            let mut state = recording_state_clone.lock().unwrap();
                            *state = false;
                        }
                        
                        // Emit timeout notification to frontend
                        let _ = app.emit("recording-timeout", ());
                        
                        break;
                    }
                    
                    // Continue waiting for more audio
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => {
                    DebugLogger::log_info("STOP_REASON: Audio channel disconnected (audio device/system failure), breaking processing loop");
                    break;
                }
            };
            DebugLogger::log_info("=== NEW AUDIO CHUNK RECEIVED ===");
            
            // Check if recording has been stopped
            {
                let state = recording_state_clone.lock().unwrap();
                if !*state {
                    DebugLogger::log_info("STOP_REASON: Recording state set to false (chunk processing check), breaking audio processing loop");
                    break;
                }
            }
            
            // Handle all audio chunks the same way in simplified mode
            // No special handling for silence chunks needed
            
            // Log audio chunk details
            let max_amplitude = audio_chunk.data.iter().map(|&x| x.abs()).fold(0.0, f32::max);
            let has_activity = audio_chunk.has_audio_activity();
            DebugLogger::log_audio_chunk(audio_chunk.data.len(), audio_chunk.sample_rate, has_activity, max_amplitude);

            // Skip empty or silent chunks
            if audio_chunk.is_empty() || !has_activity {
                DebugLogger::log_info("Skipping empty or silent audio chunk");
                continue;
            }

            // Emit status to frontend
            let _ = app.emit("processing-audio", true);

            // Transcribe audio chunk
            DebugLogger::log_info("=== STARTING STT TRANSCRIPTION ===");
            match stt_service.transcribe_chunk(audio_chunk.data, audio_chunk.sample_rate, None).await {
                Ok(transcribed_text) => {
                    DebugLogger::log_transcription_response(true, Some(&transcribed_text), None);
                    if !transcribed_text.trim().is_empty() {
                        append_dedup(&mut agg_text, &transcribed_text);
                        DebugLogger::log_info(&format!("Aggregated text length now: {}", agg_text.len()));
                        
                        // Store transcribed text but don't insert yet - wait for user to stop recording
                        DebugLogger::log_info("TEXT_INSERTION: deferring until user stops recording");
                        
                        // Emit transcribed text to frontend for display (without final processing)
                        let _ = app.emit("transcribed-text", serde_json::json!({
                            "raw": agg_text,
                            "final": agg_text  // Show raw text for now
                        }));
                    }
                    let _ = app.emit("processing-audio", false);
                }
                Err(e) => {
                    DebugLogger::log_transcription_response(false, None, Some(&e));
                    DebugLogger::log_pipeline_error("transcription", &e);
                    let _ = app.emit("processing-error", format!("Transcription error: {}", e));
                    let _ = app.emit("processing-audio", false);
                }
            }
    }
        
        DebugLogger::log_info("Audio receiver channel closed - no more audio chunks");
        DebugLogger::log_info("This could indicate:");
        DebugLogger::log_info("1. Audio stream ended unexpectedly");
        DebugLogger::log_info("2. Audio capture was stopped externally");  
        DebugLogger::log_info("3. Audio channel sender was dropped");
        DebugLogger::log_info("=== PIPELINE CLEANUP STARTING ===");
        // Unmute system audio if it was muted
        if let Some(ref audio_control) = audio_control {
            if audio_control.is_muted() {
                DebugLogger::log_info("Attempting to unmute system audio during cleanup");
                if let Err(e) = audio_control.unmute_system_audio() {
                    let error_msg = format!("Failed to unmute system audio during cleanup: {}", e);
                    eprintln!("{}", error_msg);
                    DebugLogger::log_pipeline_error("system_audio_cleanup", &error_msg);
                } else {
                    DebugLogger::log_info("System audio unmuted successfully during cleanup");
                }
            } else {
                DebugLogger::log_info("System audio was not muted, no cleanup needed");
            }
        } else {
            DebugLogger::log_info("No system audio control to clean up");
        }
        
        // Final flush - process and insert text when recording stops
        if !agg_text.trim().is_empty() {
            let raw_text = agg_text.clone();
            DebugLogger::log_info("TEXT_INSERTION: processing final text after recording stopped");
            let final_text = if let Some(ref translation_service) = translation_service {
                match translation_service.process_text(
                    &agg_text,
                    &settings.spoken_language,
                    &settings.translation_language,
                    settings.translation_enabled
                ).await {
                    Ok(processed_text) => {
                        DebugLogger::log_translation_response(true, Some(&processed_text), None, None);
                        processed_text
                    },
                    Err(e) => {
                        DebugLogger::log_translation_response(false, None, Some(&e), None);
                        DebugLogger::log_pipeline_error("translation", &e);
                        let _ = app.emit("processing-error", format!("Translation Error - Using fallback: {}", e));
                        agg_text.clone()
                    }
                }
            } else {
                agg_text.clone()
            };
            
            // Now insert the text since recording has stopped
            DebugLogger::log_info("TEXT_INSERTION: queueing text for insertion (recording stopped)");
            if settings.text_insertion_enabled {
                if let Err(e) = text_insertion_tx.send(final_text.clone()) {
                    DebugLogger::log_pipeline_error("text_insertion", &format!("failed to queue text (final flush): {}", e));
                } else {
                    DebugLogger::log_text_insertion(&final_text, true, None);
                    DebugLogger::log_info("TEXT_INSERTION: queued (recording stopped)");
                }
            } else {
                DebugLogger::log_info("TEXT_INSERTION: skipped (text insertion disabled)");
            }
            
            // Emit final processed text to frontend
            let _ = app.emit("transcribed-text", serde_json::json!({
                "raw": raw_text,
                "final": final_text
            }));
        }

        } else {
            // === SINGLE RECORDING MODE: Capture entire session ===
            DebugLogger::log_info("Starting single recording session - collecting all audio data...");
            
            // Move audio_rx into single recording mode
            let audio_rx = audio_rx;
            
            // Spawn a separate task for single recording mode to avoid blocking UI
            let app_single = app.clone();
            let stop_rx_single = stop_rx;
            let recording_state_single = recording_state_clone.clone();
            let stt_service_single = stt_service;
            let translation_service_single = translation_service;
            let settings_single = settings.clone();
            let text_insertion_tx_single = text_insertion_tx.clone();
            
            // Run single recording session inline and await completion so the outer pipeline
            // does not proceed to cleanup while the single-recording task is still active.
            (async move {
                let mut all_audio_data: Vec<f32> = Vec::new();
                let mut sample_rate = 48000; // Default sample rate, will be updated from first chunk
                
                // Collect all audio data until recording stops
                loop {
                    use std::sync::mpsc::RecvTimeoutError;
                    
                    // Check stop signal first
                    match stop_rx_single.try_recv() {
                        Ok(_) => {
                            DebugLogger::log_info("STOP_REASON: Stop signal received manually, draining remaining chunks before ending single recording session");
                            // Drain remaining chunks from the channel to prevent backup
                            let drain_start = std::time::Instant::now();
                            let mut drained_count = 0;
                            while drain_start.elapsed() < std::time::Duration::from_millis(500) {
                                match audio_rx.try_recv() {
                                    Ok(_) => drained_count += 1,
                                    Err(_) => break,
                                }
                            }
                            if drained_count > 0 {
                                DebugLogger::log_info(&format!("Single recording: drained {} remaining chunks from audio channel", drained_count));
                            }
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            DebugLogger::log_info("STOP_REASON: Stop signal channel disconnected, draining remaining chunks before ending single recording session");
                            // Drain remaining chunks from the channel to prevent backup
                            let drain_start = std::time::Instant::now();
                            let mut drained_count = 0;
                            while drain_start.elapsed() < std::time::Duration::from_millis(500) {
                                match audio_rx.try_recv() {
                                    Ok(_) => drained_count += 1,
                                    Err(_) => break,
                                }
                            }
                            if drained_count > 0 {
                                DebugLogger::log_info(&format!("Single recording: drained {} remaining chunks from audio channel", drained_count));
                            }
                            break;
                        }
                        Err(std::sync::mpsc::TryRecvError::Empty) => {
                            // No stop signal, continue collecting
                        }
                    }
                    
                    let audio_chunk = match audio_rx.recv_timeout(std::time::Duration::from_millis(200)) {
                        Ok(chunk) => chunk,
                        Err(RecvTimeoutError::Timeout) => {
                            // Check if recording state changed
                            let stop = {
                                let state = recording_state_single.lock().unwrap();
                                !*state
                            };
                            if stop {
                                DebugLogger::log_info("STOP_REASON: Recording state set to false (single recording mode), draining remaining chunks before ending session");
                                // Wait longer for the audio processing thread to send the final chunk
                                DebugLogger::log_info("DRAIN_PHASE: Waiting for final audio processing to complete...");
                                let drain_start = std::time::Instant::now();
                                let mut drained_count = 0;
                                let mut final_chunk_received = false;
                                
                                // Wait up to 2 seconds for final chunk (audio processing takes time)
                                while drain_start.elapsed() < std::time::Duration::from_millis(2000) {
                                    match audio_rx.recv_timeout(std::time::Duration::from_millis(100)) {
                                        Ok(chunk) => {
                                            drained_count += 1;
                                            final_chunk_received = true;
                                            DebugLogger::log_info(&format!("DRAIN_PHASE: Received final audio chunk {} samples at {}Hz", chunk.data.len(), chunk.sample_rate));
                                            
                                            // Process this final chunk
                                            if !chunk.data.is_empty() {
                                                sample_rate = chunk.sample_rate;
                                                all_audio_data.extend_from_slice(&chunk.data);
                                            }
                                        }
                                        Err(_) => {
                                            // Continue waiting if no final chunk yet
                                            if final_chunk_received {
                                                break; // We got the final chunk, no more expected
                                            }
                                            std::thread::sleep(std::time::Duration::from_millis(50));
                                        }
                                    }
                                }
                                
                                DebugLogger::log_info(&format!("DRAIN_PHASE: Completed - received {} chunks, final_chunk_received: {}", drained_count, final_chunk_received));
                                break;
                            }
                            
                            // Check if recording has exceeded max time limit
                            if recording_start_time.elapsed() >= max_recording_duration {
                                DebugLogger::log_info(&format!("STOP_REASON: Single recording exceeded maximum time limit of {} minutes", max_recording_time_minutes));
                                
                                // Set recording state to false
                                {
                                    let mut state = recording_state_single.lock().unwrap();
                                    *state = false;
                                }
                                
                                // Emit timeout notification to frontend
                                let _ = app_single.emit("recording-timeout", ());
                                
                                // Drain remaining chunks and break
                                let drain_start = std::time::Instant::now();
                                let mut drained_count = 0;
                                while drain_start.elapsed() < std::time::Duration::from_millis(500) {
                                    match audio_rx.try_recv() {
                                        Ok(_) => drained_count += 1,
                                        Err(_) => break,
                                    }
                                }
                                if drained_count > 0 {
                                    DebugLogger::log_info(&format!("Single recording: drained {} remaining chunks from audio channel after timeout", drained_count));
                                }
                                break;
                            }
                            
                            continue; // Keep waiting for more audio
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            DebugLogger::log_info("STOP_REASON: Audio channel disconnected, ending single recording session");
                            break;
                        }
                    };
                    
                    // Collect audio data from this chunk
                    if !audio_chunk.data.is_empty() {
                        sample_rate = audio_chunk.sample_rate;
                        all_audio_data.extend_from_slice(&audio_chunk.data);
                    }
                }
                
                // Process the complete audio recording
                if !all_audio_data.is_empty() {
                    DebugLogger::log_info(&format!("Single recording complete: {} samples ({:.1}s) at {}Hz", 
                        all_audio_data.len(), all_audio_data.len() as f32 / sample_rate as f32, sample_rate));
                    
                    // Convert to WAV format and send to STT service
                    DebugLogger::log_info("Sending complete recording to STT service...");
                    
                    match stt_service_single.transcribe_chunk(all_audio_data, sample_rate, Some("stt_single")).await {
                            Ok(transcription) => {
                                DebugLogger::log_info(&format!("STT complete transcription: '{}'", transcription));
                        // IMMEDIATELY emit raw transcription to frontend (don't wait for translation)
                                let _ = app_single.emit("transcribed-text", serde_json::json!({
                                    "raw": transcription,
                                    "final": "" // Empty final initially - will be updated when translation completes
                                }));
                                DebugLogger::log_info("EMIT: Sent raw transcription immediately to frontend");

                                // Emit processing progress to show translation is happening
                                let _ = app_single.emit("processing-status", serde_json::json!({"status": "translating"}));

                                // Now do translation/correction in background and emit update when done
                                let final_text = if let Some(ref translation_service) = translation_service_single {
                                    match translation_service.process_text(
                                        &transcription,
                                        &settings_single.spoken_language,
                                        &settings_single.translation_language,
                                        settings_single.translation_enabled
                                    ).await {
                                        Ok(processed_text) => {
                                            DebugLogger::log_translation_response(true, Some(&processed_text), None, None);

                                            // EMIT FINAL PROCESSED TEXT
                                            let _ = app_single.emit("transcribed-text", serde_json::json!({
                                                "raw": transcription,
                                                "final": processed_text
                                            }));
                                            DebugLogger::log_info("EMIT: Sent final processed text to frontend");

                                            processed_text
                                        },
                                        Err(e) => {
                                            DebugLogger::log_translation_response(false, None, Some(&e), None);
                                            DebugLogger::log_pipeline_error("translation", &e);
                                            let _ = app_single.emit("processing-error", format!("Translation Error - Using fallback: {}", e));

                                            // FALLBACK: Use raw transcription as final (don't leave empty)
                                            let _ = app_single.emit("transcribed-text", serde_json::json!({
                                                "raw": transcription,
                                                "final": transcription // Use raw as fallback
                                            }));
                                            DebugLogger::log_info("EMIT: Sent raw transcription as fallback final text");

                                            transcription.clone()
                                        }
                                    }
                                } else {
                                    // No translation service - just send raw transcription as final
                                    let _ = app_single.emit("transcribed-text", serde_json::json!({
                                        "raw": transcription,
                                        "final": transcription
                                    }));
                                    DebugLogger::log_info("EMIT: Sent raw transcription as final (no translation service)");

                                    transcription.clone()
                                };

                                // CLEAR PROCESSING STATUS after completion
                                let _ = app_single.emit("processing-status", serde_json::json!({"status": ""}));
                                
                                // In single recording mode, the recording has already stopped, so insert text
                                if settings_single.text_insertion_enabled {
                                    DebugLogger::log_info("TEXT_INSERTION: queueing complete transcription for insertion (single mode - recording already stopped)");
                                    if let Err(e) = text_insertion_tx_single.send(final_text.clone()) {
                                        DebugLogger::log_pipeline_error("text_insertion", &format!("failed to queue complete transcription: {}", e));
                                    } else {
                                        DebugLogger::log_text_insertion(&final_text, true, None);
                                        DebugLogger::log_info("TEXT_INSERTION: queued complete transcription");
                                    }
                                } else {
                                    DebugLogger::log_info("TEXT_INSERTION: skipped (text insertion disabled)");
                                }
                                
                                // Note: transcribed-text events already emitted above at each stage
                            },
                            Err(e) => {
                                DebugLogger::log_pipeline_error("stt", &format!("STT processing failed: {}", e));
                                let _ = app_single.emit("processing-error", format!("STT Error: {}", e));
                            }
                        }
                } else {
                    DebugLogger::log_info("Single recording session ended with no audio data collected");
                }
            }).await;
        }

        // Common cleanup for both modes
        {
            let mut state = recording_state_clone.lock().unwrap();
            *state = false;
            DebugLogger::log_info("RECORDING_STATE_CHANGE: Set to false in pipeline cleanup (natural termination)");
            DebugLogger::log_info("Recording state set to false");
        }
        // Show completion notification when processing ends
        DebugLogger::log_info("Showing processing completed notification");
        let _ = app.notification()
            .builder()
            .title("Processing completed")
            .body("✍️ Text copied to clipboard")
            .show();

        // Emit recording-stopped event AFTER transcription has been shown to frontend
        DebugLogger::log_info("Emitting recording-stopped event to frontend");
        let _ = app.emit("recording-stopped", {});
            
        DebugLogger::log_info("=== PIPELINE CLEANUP COMPLETE ===");
    });
    
    // Store the audio_capture in a way that allows proper cleanup
    // We need to modify the audio capture to use the recording_state for stopping
    // For now, we'll implement the stop mechanism in the stop_recording command
    
    Ok(())
}

// Command to stop recording
#[tauri::command]
fn stop_recording(
    app: AppHandle,
    recording_state: State<'_, RecordingState>,
    audio_stop_sender: State<'_, AudioStopSender>,
    audio_manager: State<'_, AudioManagerHandle>,
    fsm: State<'_, HotkeySMState>
) -> Result<(), String> {
    // Dump last hotkey info for correlation
    if let Ok(last) = app.state::<LastHotkey>().inner().lock() {
        if let Some((action, when)) = &*last {
            let since = when.elapsed().as_millis();
            DebugLogger::log_info(&format!("stop_recording invoked - last_hotkey: action={}, {}ms ago", action, since));
        } else {
            DebugLogger::log_info("stop_recording invoked - last_hotkey: none");
        }
    }
    
    // Log call stack info to track unexpected stops
    DebugLogger::log_info("STOP_RECORDING_CALLED: Analyzing call source...");
    
    // Check if this is a legitimate user-initiated stop vs automatic/unexpected stop
    let user_initiated = true; // Always treat as user-initiated since we removed suppression mechanism
    
    DebugLogger::log_info(&format!("STOP_RECORDING_CALLED: user_initiated={}", user_initiated));
    
    // If we're not currently recording, ignore duplicate stop requests.
    // Also implement a short cooldown so rapid repeated Stop commands are dropped.
    let cooldown_ms = 100u128; // Reduced from 300ms for better responsiveness
    if let Ok(lst) = app.state::<LastStopTime>().inner().lock() {
        if let Some(prev) = *lst {
            let elapsed = prev.elapsed().as_millis();
            if elapsed < cooldown_ms {
                DebugLogger::log_info(&format!("stop_recording ignored due to cooldown ({}ms since last stop)", elapsed));
                return Ok(());
            }
        }
    }
    {
        let state = recording_state.inner().lock().map_err(|e| e.to_string())?;
        if !*state {
            DebugLogger::log_info("stop_recording called but recording_state already false - ignoring duplicate stop");
            return Ok(());
        }
    }

    // Send Stop command to audio manager-owned capture if available
    if let Ok(sender) = audio_manager.lock() {
        let (ack_tx, ack_rx) = std_mpsc::channel();
        let _ = sender.send(AudioManagerCommand::Stop { reply: Some(ack_tx) });
        match ack_rx.recv_timeout(std::time::Duration::from_secs(2)) {
            Ok(Ok(_)) => DebugLogger::log_info("Audio manager acknowledged stop"),
            Ok(Err(e)) => DebugLogger::log_pipeline_error("audio_manager", &format!("Stop error: {}", e)),
            Err(_) => DebugLogger::log_info("No ack from audio manager on stop (continuing)")
        }
    }
    DebugLogger::log_info("stop_recording command called");
    
    // Set recording state to false
    {
        let mut state = recording_state.inner().lock().map_err(|e| e.to_string())?;
        *state = false;
        DebugLogger::log_info("RECORDING_STATE_CHANGE: Set to false in stop_recording command (user/external stop)");
        DebugLogger::log_info("Recording state set to false in stop_recording");
    }

    // Update FSM to Idle state
    fsm.force_set_state(hotkey_fsm::RecordingState::Idle)
        .unwrap_or_else(|e| DebugLogger::log_info(&format!("Failed to set FSM to Idle: {}", e)));

    // Send stop signal to audio processing task
    {
        let mut audio_stop = audio_stop_sender.inner().lock().map_err(|e| e.to_string())?;
        if let Some(sender) = audio_stop.take() {
            match sender.send(()) {
                Ok(_) => DebugLogger::log_info("Stop signal sent to audio processing task"),
                Err(_) => DebugLogger::log_info("Failed to send stop signal (channel may be closed)"),
            }
        } else {
            DebugLogger::log_info("No audio stop sender available (recording may not be active)");
        }
    }
    // Update last stop time
    if let Ok(mut lst) = app.state::<LastStopTime>().inner().lock() {
        *lst = Some(std::time::Instant::now());
    }
    
    let _ = app.emit("recording-stopped", ());
    DebugLogger::log_info("Recording stopped successfully");
    Ok(())
}

// Command to test API connectivity
#[tauri::command]
async fn test_stt_api(endpoint: String, api_key: String) -> Result<bool, String> {
    if endpoint.is_empty() {
        return Err("API endpoint cannot be empty".to_string());
    }
    
    if api_key.is_empty() {
        return Err("API key cannot be empty".to_string());
    }

    let client = reqwest::Client::new();
    
    // Try to test the models endpoint first (common for OpenAI-compatible APIs)
    let models_url = format!("{}/models", endpoint);
    
    match client
        .get(&models_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                Ok(true)
            } else if response.status() == 401 {
                Err("Unauthorized: Invalid API key".to_string())
            } else if response.status() == 404 {
                // Models endpoint might not exist, try a simple health check or audio transcription endpoint
                let transcription_url = format!("{}/audio/transcriptions", endpoint);
                match client
                    .head(&transcription_url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .timeout(std::time::Duration::from_secs(10))
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.status().is_success() || resp.status() == 400 || resp.status() == 422 {
                            // 400/422 is expected for HEAD request without proper audio data
                            Ok(true)
                        } else if resp.status() == 401 {
                            Err("Unauthorized: Invalid API key".to_string())
                        } else {
                            Err(format!("API returned status code: {}", resp.status()))
                        }
                    }
                    Err(e) => Err(format!("Network error: {}", e))
                }
            } else {
                Err(format!("API returned status code: {}", response.status()))
            }
        }
        Err(e) => {
            if e.is_timeout() {
                Err("Request timed out. Check your internet connection and API endpoint.".to_string())
            } else if e.is_connect() {
                Err("Cannot connect to API endpoint. Check the URL and your internet connection.".to_string())
            } else {
                Err(format!("Network error: {}", e))
            }
        }
    }
}

// Command to validate settings
#[tauri::command]
async fn validate_settings(settings: serde_json::Value) -> Result<serde_json::Value, String> {
    let mut errors = Vec::new();

    // Validate API endpoint
    if let Some(endpoint) = settings["apiEndpoint"].as_str() {
        if endpoint.is_empty() {
            errors.push("API endpoint cannot be empty".to_string());
        } else if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            errors.push("API endpoint must start with http:// or https://".to_string());
        }
    } else {
        errors.push("API endpoint is required".to_string());
    }

    // Validate API key
    if let Some(api_key) = settings["apiKey"].as_str() {
        if api_key.is_empty() {
            errors.push("API key cannot be empty".to_string());
        } else if api_key.len() < 10 {
            errors.push("API key seems too short".to_string());
        }
    } else {
        errors.push("API key is required".to_string());
    }

    // Validate hotkeys
    if let Some(hotkeys) = settings["hotkeys"].as_object() {
        if let Some(hands_free) = hotkeys.get("handsFree").and_then(|v| v.as_str()) {
            if hands_free.is_empty() {
                errors.push("Hands-free hotkey cannot be empty".to_string());
            }
        }
    }

    Ok(serde_json::json!({
        "valid": errors.is_empty(),
        "errors": errors
    }))
}

// Command to show/hide main window
#[tauri::command]
fn toggle_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        match window.is_visible() {
            Ok(true) => {
                let _ = window.hide();
                let _ = window.set_skip_taskbar(true);
            }
            Ok(false) => {
                let _ = window.set_skip_taskbar(false);
                let _ = window.show();
                let _ = window.set_focus();
            }
            Err(_) => {
                let _ = window.set_skip_taskbar(false);
                let _ = window.show();
                let _ = window.set_focus();
            }
        }
    }
    Ok(())
}

// Command to quit the application
#[tauri::command]
fn quit_app(app: tauri::AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

// Removed update_spoken_language - now using localStorage-only approach

// Removed update_translation_language - now using localStorage-only approach

// Removed update_audio_device - now using localStorage-only approach

#[tauri::command]
async fn store_api_key(app: AppHandle, api_key: String) -> Result<(), String> {
    DebugLogger::log_info(&format!("store_api_key called with key length: {}", api_key.len()));
    AppSettings::default().store_api_key(&app, api_key)?;
    DebugLogger::log_info("API key stored successfully in backend");
    Ok(())
}

#[tauri::command]
async fn get_api_key(app: AppHandle) -> Result<String, String> {
    AppSettings::default().get_api_key(&app)
}

#[tauri::command]
async fn debug_api_key_info(app: AppHandle) -> Result<serde_json::Value, String> {
    AppSettings::default().debug_api_key_info(&app)
}

#[tauri::command]
async fn has_api_key(app: AppHandle) -> Result<bool, String> {
    Ok(AppSettings::default().has_api_key(&app))
}

// Removed update_api_endpoint - now using localStorage-only approach

// Removed toggle_translation - now using localStorage-only approach

// Removed update_auto_mute - now using localStorage-only approach  

// Removed update_debug_logging - now using localStorage-only approach

#[tauri::command]
async fn get_available_audio_devices() -> Result<Vec<String>, String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    
    let host = cpal::default_host();
    let mut devices = Vec::new();
    
    // Add default device
    devices.push("default".to_string());
    
    // Add available input devices
    match host.input_devices() {
        Ok(input_devices) => {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    devices.push(name);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to enumerate input devices: {}", e);
        }
    }
    
    Ok(devices)
}

#[tauri::command]
async fn test_audio_capture() -> Result<String, String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    
    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or("No input device available")?;
    
    let config = device.default_input_config()
        .map_err(|e| format!("Failed to get input config: {}", e))?;
    
    Ok(format!(
        "Audio device: {}\nSample rate: {}\nChannels: {}\nFormat: {:?}",
        device.name().unwrap_or_else(|_| "Unknown".to_string()),
        config.sample_rate().0,
        config.channels(),
        config.sample_format()
    ))
}

#[tauri::command]
async fn get_recording_status(recording_state: State<'_, RecordingState>) -> Result<bool, String> {
    let state = recording_state.inner().lock().map_err(|e| e.to_string())?;
    Ok(*state)
}

#[tauri::command]
async fn get_debug_logs(app: AppHandle, lines: Option<usize>) -> Result<String, String> {
    DebugLogger::get_recent_logs(&app, lines.unwrap_or(100))
}

#[tauri::command]
async fn clear_debug_logs(app: AppHandle) -> Result<(), String> {
    DebugLogger::clear_log(&app)
}

#[tauri::command]
async fn get_log_file_path(app: AppHandle) -> Result<String, String> {
    DebugLogger::get_log_file_path(&app)
}

#[tauri::command]
async fn get_data_directory_info(app: AppHandle) -> Result<serde_json::Value, String> {
    use serde_json::json;
    
    // Get the actual data directory being used
    let data_dir = if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let portable_dir = exe_dir.join("data");
            if portable_dir.exists() {
                ("portable", portable_dir.to_string_lossy().to_string())
            } else {
                let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
                ("appdata", app_dir.to_string_lossy().to_string())
            }
        } else {
            let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
            ("appdata", app_dir.to_string_lossy().to_string())
        }
    } else {
        let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        ("appdata", app_dir.to_string_lossy().to_string())
    };
    
    let log_path = DebugLogger::get_log_file_path(&app)?;
    
    Ok(json!({
        "mode": data_dir.0,
        "dataDirectory": data_dir.1,
        "settingsFile": format!("{}/settings.json", data_dir.1),
        "apiKeyFile": format!("{}/api.key", data_dir.1),
        "logFile": log_path,
        "isPortable": data_dir.0 == "portable"
    }))
}

// Command used by the frontend to annotate backend logs with frontend-originated events
#[tauri::command]
async fn frontend_log(tag: String, payload: Option<serde_json::Value>) -> Result<(), String> {
    let payload_str = payload.map(|p| p.to_string()).unwrap_or_else(|| "null".to_string());
    DebugLogger::log_info(&format!("FRONTEND_LOG: tag={}, payload={}", tag, payload_str));
    Ok(())
}

// Test command for text insertion debugging
#[tauri::command]
async fn test_text_insertion(test_text: String) -> Result<(), String> {
    DebugLogger::log_info(&format!("TEST_TEXT_INSERTION: called with text='{}'", test_text));
    let text_insertion_service = TextInsertionService::new();
    text_insertion_service.test_insert(&test_text)
}

// Translation command for frontend
#[tauri::command]
async fn translate_text(
    text: String,
    source_lang: String,
    target_lang: String,
    app_state: State<'_, Mutex<AppSettings>>,
    app: AppHandle
) -> Result<String, String> {
    DebugLogger::log_info(&format!("translate_text called: '{}' from {} to {}", text, source_lang, target_lang));
    
    // Get current settings and clone necessary values to avoid holding the lock across await
    let (api_endpoint, translation_model) = {
        let settings = app_state.lock().map_err(|e| format!("Failed to lock settings: {}", e))?;
        (settings.api_endpoint.clone(), settings.translation_model.clone())
    };
    
    // Get API key using the same method as start_recording
    let settings_for_api = AppSettings::default();
    let api_key = settings_for_api.get_api_key(&app).map_err(|e| {
        let error_msg = format!("Failed to get API key: {}", e);
        DebugLogger::log_info(&format!("No API key available for translation: {}", error_msg));
        error_msg
    })?;
    
    // Create translation service
    let translation_service = TranslationService::new(
        api_endpoint,
        api_key,
        translation_model
    );
    
    // Perform translation
    match translation_service.process_text(&text, &source_lang, &target_lang, true).await {
        Ok(translated) => {
            DebugLogger::log_info(&format!("Translation successful: '{}'", translated));
            Ok(translated)
        }
        Err(e) => {
            DebugLogger::log_info(&format!("Translation failed: {}", e));
            Err(e)
        }
    }
}

// New commands for localStorage-based settings
#[tauri::command]
async fn load_settings_from_frontend() -> Result<String, String> {
    // This is a placeholder - the frontend will handle localStorage directly
    // We just return "ok" to indicate the command exists
    Ok("localStorage".to_string())
}

#[tauri::command]
async fn save_settings_from_frontend(
    app: AppHandle,
    spoken_language: Option<String>,
    translation_language: Option<String>,
    audio_device: Option<String>,
    theme: Option<String>,
    api_endpoint: Option<String>,
    api_key: Option<String>,
    stt_model: Option<String>,
    translation_model: Option<String>,
    auto_mute: Option<bool>,
    translation_enabled: Option<bool>,
    debug_logging: Option<bool>,
    hands_free_hotkey: Option<String>,
    text_insertion_enabled: Option<bool>,
    audio_chunking_enabled: Option<bool>,
    max_recording_time_minutes: Option<u32>
) -> Result<(), String> {
    // Log the settings being saved (without logging the API key for security)
    DebugLogger::log_info(&format!("SETTINGS_SAVE_FRONTEND: spoken_language={:?}, translation_language={:?}, audio_device={:?}, theme={:?}, api_endpoint={:?}, stt_model={:?}, translation_model={:?}, api_key_provided={}, auto_mute={:?}, translation_enabled={:?}, debug_logging={:?}, hands_free={:?}, text_insertion_enabled={:?}, audio_chunking_enabled={:?}, max_recording_time_minutes={:?}",
        spoken_language, translation_language, audio_device, theme, api_endpoint, stt_model, translation_model, api_key.as_ref().map_or(false, |k| !k.is_empty()), auto_mute, translation_enabled, debug_logging, hands_free_hotkey, text_insertion_enabled, audio_chunking_enabled, max_recording_time_minutes));

    // Validate that we have at least some parameters
    if spoken_language.is_none() && translation_language.is_none() && theme.is_none() && auto_mute.is_none() {
        return Err("No valid settings provided to save".to_string());
    }

    // Store API key securely if provided and not empty
    // (Note: we now send empty string for security, so API key is stored separately via store_api_key command)
    if let Some(api_key_val) = api_key {
        if !api_key_val.is_empty() {
            let settings = AppSettings::default();
            settings.store_api_key(&app, api_key_val)?;
            DebugLogger::log_info("API key stored securely in backend");
        }
    }

    // Re-initialize debug logging with the new state if provided
    if let Some(debug_enabled) = debug_logging {
        DebugLogger::init_with_state(&app, debug_enabled)?;
    }

    Ok(())
}

#[tauri::command]
async fn init_debug_logging(app: AppHandle, enabled: bool) -> Result<(), String> {
    DebugLogger::log_info(&format!("Debug logging manually set to: {}", enabled));
    DebugLogger::init_with_state(&app, enabled)?;
    Ok(())
}

#[tauri::command]
async fn show_recording_timeout_notification(app: AppHandle, max_time_minutes: u32) -> Result<(), String> {
    DebugLogger::log_info(&format!("Recording stopped due to maximum time limit of {} minutes", max_time_minutes));
    
    // Emit event to frontend to show tray notification
    app.emit("show-recording-timeout-notification", max_time_minutes)
        .map_err(|e| format!("Failed to emit timeout notification event: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn load_persistent_settings(app: AppHandle) -> Result<serde_json::Value, String> {
    let settings = SettingsStore::load(&app)?;
    Ok(serde_json::to_value(settings).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn save_persistent_settings(app: AppHandle, settings: serde_json::Value) -> Result<(), String> {
    DebugLogger::log_info(&format!("SETTINGS_SAVE_PERSISTENT: Incoming settings JSON: {}", settings));
    match serde_json::from_value::<storage::PersistentSettings>(settings.clone()) {
        Ok(parsed_settings) => {
            DebugLogger::log_info(&format!("SETTINGS_SAVE_PERSISTENT: Successfully parsed settings object"));
            match SettingsStore::save(&app, &parsed_settings) {
                Ok(_) => {
                    DebugLogger::log_info("SETTINGS_SAVE_PERSISTENT: Successfully saved to store");
                    Ok(())
                }
                Err(e) => {
                    DebugLogger::log_pipeline_error("settings_store", &format!("Failed to save settings: {}", e));
                    Err(format!("Failed to save settings to store: {}", e))
                }
            }
        }
        Err(deserialize_err) => {
            let error_msg = format!("SETTINGS_SAVE_PERSISTENT: Failed to deserialize settings - missing fields error: {}", deserialize_err);
            DebugLogger::log_pipeline_error("settings_deserialize", &error_msg);
            DebugLogger::log_info(&format!("SETTINGS_SAVE_PERSISTENT: Incoming JSON was: {}", settings));
            Err(error_msg)
        }
    }
}

#[tauri::command]
async fn update_persistent_setting(app: AppHandle, field: String, value: serde_json::Value) -> Result<(), String> {
    SettingsStore::update_field(&app, &field, value)?;
    Ok(())
}

#[tauri::command]
fn get_hotkey_fsm_state(fsm: State<'_, HotkeySMState>) -> Result<String, String> {
    let state = fsm.get_state()?;
    let state_str = match state {
        hotkey_fsm::RecordingState::Idle => "Idle",
        hotkey_fsm::RecordingState::Recording => "Recording",
    };
    Ok(state_str.to_string())
}

#[tauri::command]
fn reset_hotkey_fsm(fsm: State<'_, HotkeySMState>) -> Result<(), String> {
    fsm.force_set_state(hotkey_fsm::RecordingState::Idle)?;
    fsm.reset_debounce()?;
    Ok(())
}

#[tauri::command]
fn set_hotkey_fsm_recording(fsm: State<'_, HotkeySMState>, recording: bool) -> Result<(), String> {
    let state = if recording {
        hotkey_fsm::RecordingState::Recording
    } else {
        hotkey_fsm::RecordingState::Idle
    };
    fsm.force_set_state(state)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
    .plugin(tauri_plugin_notification::init())
    .plugin(
        tauri_plugin_store::Builder::new()
            .build()
    )
        // Register Stronghold plugin for encrypted at-rest storage (JS guest APIs available)
        .setup(|app| {
            // Initialize Stronghold plugin for encrypted storage
            let salt_path = app
                .path()
                .app_local_data_dir()
                .expect("could not resolve app local data path")
                .join("salt.txt");
            let _ = app.handle().plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build());
            
            // Initialize debug logging first (disabled by default, will be enabled by frontend)
            if let Err(e) = DebugLogger::init(&app.handle()) {
                eprintln!("Failed to initialize debug logging: {}", e);
            }
            
            DebugLogger::log_info("TalkToMe application starting up");
            DebugLogger::log_info("Initialized with default settings for tray menu");
            
            // Create a simple system tray menu
            let tray_menu = {
                let show_hide = MenuItemBuilder::with_id("show_hide", "Show/Hide TalkToMe").build(app)?;
                
                let preferences = MenuItemBuilder::with_id("preferences", "Preferences").build(app)?;
                let api_settings = MenuItemBuilder::with_id("api_settings", "API Settings").build(app)?;
                let language_settings = MenuItemBuilder::with_id("language_settings", "Language Settings").build(app)?;
                let audio_settings = MenuItemBuilder::with_id("audio_settings", "Audio Settings").build(app)?;
                let about = MenuItemBuilder::with_id("about", "About TalkToMe").build(app)?;
                let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

                MenuBuilder::new(app)
                    .items(&[
                        &show_hide,
                        &preferences,
                        &api_settings,
                        &language_settings, 
                        &audio_settings,
                        &about,
                        &quit,
                    ])
                    .build()?
            };

            // Build the system tray
            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("TalkToMe - Voice to Text with Translation")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&tray_menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "show_hide" => {
                            if let Err(e) = toggle_window(app.clone()) {
                                eprintln!("Failed to toggle window: {}", e);
                            }
                        }
                        "preferences" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("show-preferences", ());
                            }
                        }
                        "api_settings" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("show-api-settings", ());
                            }
                        }
                        "language_settings" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("show-language-settings", ());
                            }
                        }
                        "audio_settings" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("show-audio-settings", ());
                            }
                        }
                        "about" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("show-about", ());
                            }
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            let app = tray.app_handle();
                            if let Err(e) = toggle_window(app.clone()) {
                                eprintln!("Failed to toggle window from tray click: {}", e);
                            }
                        }
                        TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        } => {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            // Handle window close request (minimize to tray instead of closing)
            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.app_handle().clone();
                window.on_window_event(move |event| {
                    match event {
                        tauri::WindowEvent::CloseRequested { api, .. } => {
                            api.prevent_close();
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        _ => {}
                    }
                });
            }

            Ok(())
        })
        .manage(Mutex::<HashMap<String, String>>::new(HashMap::new()))
        .manage(Arc::new(Mutex::new(false)) as RecordingState)
        .manage(Arc::new(Mutex::new(None)) as AudioStopSender)
    .manage(Arc::new(Mutex::new(None)) as LastStopTime)
        .manage(Arc::new(Mutex::new(None)) as LastHotkey)
        .manage(Arc::new(HotkeySM::new(150)) as HotkeySMState)
        // Spawn a dedicated single-thread audio manager to own non-Send AudioCapture
        .manage({
            // Create an mpsc channel for sending commands to the manager
            let (cmd_tx, cmd_rx) = std_mpsc::channel::<AudioManagerCommand>();
            // Spawn thread that owns AudioCapture and responds to commands
            std::thread::spawn(move || {
                DebugLogger::log_info("Audio manager thread starting");
                // The audio capture instance is owned here on this single thread
                let mut audio_capture_opt: Option<AudioCapture> = None;
                for cmd in cmd_rx.iter() {
                    match cmd {
                        AudioManagerCommand::Start { reply, audio_chunking_enabled } => {
                            DebugLogger::log_info("Audio manager received Start command");
                            // If already started, return error
                            if audio_capture_opt.is_some() {
                                DebugLogger::log_info("Audio manager received duplicate Start - capture already running");
                                let err_msg = "Audio capture already started; call stop_recording() before starting a new capture".to_string();
                                // store for diagnostics
                                if let Ok(mut last_err) = AUDIO_MANAGER_LAST_ERROR.lock() {
                                    *last_err = Some(err_msg.clone());
                                }
                                let _ = reply.send(Err(err_msg));
                                continue;
                            }
                            // Create and start capture (only once)
                            let mut capture = AudioCapture::new();
                            match capture.start_capture(audio_chunking_enabled) {
                                Ok(rx) => {
                                    audio_capture_opt = Some(capture);
                                    DebugLogger::log_info("Audio manager successfully started capture and returned receiver");
                                    let _ = reply.send(Ok(rx));
                                }
                                Err(e) => {
                                    let msg = format!("Failed to start capture in manager: {}", e);
                                    DebugLogger::log_pipeline_error("audio_manager", &msg);
                                    let _ = reply.send(Err(msg));
                                }
                            }
                        }
                        AudioManagerCommand::Stop { reply } => {
                            DebugLogger::log_info("Audio manager received Stop command");
                            if let Some(mut cap) = audio_capture_opt.take() {
                                DebugLogger::log_info("Audio manager is stopping active capture (cap was Some)");
                                if let Err(e) = cap.stop_recording() {
                                    DebugLogger::log_pipeline_error("audio_manager", &format!("Error stopping capture: {}", e));
                                } else {
                                    DebugLogger::log_info("Audio manager stop_recording() returned Ok");
                                }
                            } else {
                                DebugLogger::log_info("Audio manager Stop called but no active capture was present (cap was None)");
                                if let Ok(mut last_err) = AUDIO_MANAGER_LAST_ERROR.lock() {
                                    *last_err = Some("Stop called but no active capture present".to_string());
                                }
                            }
                            if let Some(r) = reply {
                                let _ = r.send(Ok(()));
                            }
                        }
                    }
                }
                DebugLogger::log_info("Audio manager thread exiting");
            });
            Arc::new(Mutex::new(cmd_tx)) as AudioManagerHandle
        })
        .invoke_handler(tauri::generate_handler![
            greet, 
            start_recording, 
            stop_recording, 
            toggle_window, 
            quit_app, 
            register_hotkeys, 
            test_stt_api, 
            validate_settings,
            store_api_key,
            get_api_key,
            has_api_key,
            debug_api_key_info,
            get_available_audio_devices,
            test_audio_capture,
            get_recording_status,
            get_debug_logs,
            clear_debug_logs,
            get_log_file_path,
            get_data_directory_info,
            frontend_log,
            test_text_insertion,
            translate_text,
            load_settings_from_frontend,
            save_settings_from_frontend,
            init_debug_logging,
            get_audio_manager_last_error,
            clear_audio_manager_last_error,
            show_recording_timeout_notification,
            test_hotkey_parsing,
            show_recording_started_notification,
            show_recording_stopped_notification,
            load_persistent_settings,
            save_persistent_settings,
            update_persistent_setting,
            get_hotkey_fsm_state,
            reset_hotkey_fsm,
            set_hotkey_fsm_recording
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
