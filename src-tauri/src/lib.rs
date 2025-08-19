use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder},
    Manager, Emitter, AppHandle, State,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState, GlobalShortcutExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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

// Global state to track registered hotkeys and active recording
type HotkeyRegistry = Mutex<HashMap<String, String>>;
type RecordingState = Arc<Mutex<bool>>;
type AudioStopSender = Arc<Mutex<Option<std::sync::mpsc::Sender<()>>>>;
// Flag used to temporarily suppress stop events while we perform text insertion
type InsertionSuppression = Arc<Mutex<bool>>;
// Track last stop timestamp to avoid rapid duplicate stops (cooldown)
type LastStopTime = Arc<Mutex<Option<std::time::Instant>>>;
// Track the last hotkey action and when it happened to help debug stop origins
type LastHotkey = Arc<Mutex<Option<(String, std::time::Instant)>>>;

// Commands sent to the single-threaded audio manager which owns the non-Send AudioCapture
enum AudioManagerCommand {
    Start {
        // reply channel to send back the audio chunk receiver or error
        reply: std_mpsc::Sender<Result<std_mpsc::Receiver<crate::audio::AudioChunk>, String>>,
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
            "win" | "super" | "cmd" => modifiers |= Modifiers::SUPER,
            key => {
                // Try to parse the key
                let code = match key.to_lowercase().as_str() {
                    "space" => Code::Space,
                    "escape" | "esc" => Code::Escape,
                    "enter" | "return" => Code::Enter,
                    "backspace" => Code::Backspace,
                    "tab" => Code::Tab,
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
                    // Single character keys
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
                    _ => return Err(format!("Unsupported key: {}", key)),
                };
                key_code = Some(code);
                break;
            }
        }
    }
    
    let code = key_code.ok_or_else(|| "No key specified in hotkey".to_string())?;
    Ok(Shortcut::new(Some(modifiers), code))
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
            format!("Failed to parse hotkey '{}' for action '{}': {}", hotkey_str, action, e)
        })?;
        // Register handler to emit an event when the shortcut is triggered
        let action_clone = action.clone();
        let app_for_emit = app.clone();
        global_shortcut
            .on_shortcut(shortcut, move |app_handle, _sc, ev| {
                    // Read insertion suppression early
                    let suppress = *app_handle.state::<InsertionSuppression>().inner().lock().unwrap();

                    // Debounce repeated hotkey firings from programmatic input (ms)
                    let debounce_ms = 150u128;
                    if let Ok(mut last_hotkey) = app_handle.state::<LastHotkey>().inner().lock() {
                        if let Some((ref last_action, ref when)) = *last_hotkey {
                            if last_action == &action_clone && when.elapsed().as_millis() < debounce_ms {
                                let ts_ms = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .map(|d| d.as_millis())
                                    .unwrap_or(0);
                                DebugLogger::log_info(&format!("HOTKEY_DEBOUNCE: action={}, state={:?}, suppress={}, ts_ms={}, last_elapsed={}ms", action_clone, ev.state, suppress, ts_ms, when.elapsed().as_millis()));
                                return;
                            }
                        }
                        // update last hotkey timestamp for correlation
                        *last_hotkey = Some((action_clone.clone(), std::time::Instant::now()));
                    }

                    let ts_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0);
                    DebugLogger::log_info(&format!("HOTKEY_TRIGGER: action={}, state={:?}, suppress={}, ts_ms={}", action_clone, ev.state, suppress, ts_ms));

                    // Normalize action names to support both camelCase and snake_case
                    let normalized = match action_clone.as_str() {
                        "pushToTalk" | "push_to_talk" => "push_to_talk",
                        "handsFree" | "hands_free" => "hands_free",
                        other => other,
                    };

                    // If an insertion is in progress, ignore start/toggle events entirely
                    if suppress {
                        DebugLogger::log_info("Hotkey event suppressed due to text insertion in progress");
                        return;
                    }

                    match (normalized, ev.state) {
                        // Push-to-talk: Pressed = start, Released = stop
                        ("push_to_talk", ShortcutState::Pressed) => {
                            let _ = app_for_emit.emit("start-recording-from-hotkey", ());
                        }
                        ("push_to_talk", ShortcutState::Released) => {
                            let _ = app_for_emit.emit("stop-recording-from-hotkey", ());
                        }
                        // Hands-free: Pressed = toggle (ignore release)
                        ("hands_free", ShortcutState::Pressed) => {
                            let _ = app_for_emit.emit("toggle-recording-from-hotkey", ());
                        }
                        _ => {
                            let state = match ev.state { ShortcutState::Pressed => "pressed", ShortcutState::Released => "released" };
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

// Command to start recording
#[tauri::command]
async fn start_recording(
    app: AppHandle, 
    recording_state: State<'_, RecordingState>,
    audio_stop_sender: State<'_, AudioStopSender>,
    audio_manager: State<'_, AudioManagerHandle>,
    
    spoken_language: String,
    translation_language: String,
    api_endpoint: String,
    stt_model: String,
    auto_mute: bool,
    translation_enabled: bool,
    translation_model: String,
    text_insertion_enabled: bool
) -> Result<(), String> {
    // Check if already recording
    {
        let state = recording_state.inner().lock().map_err(|e| e.to_string())?;
        if *state {
            return Err("Already recording".to_string());
        }
    }

    // Get API key (use default AppSettings instance for the method)
    DebugLogger::log_info("=== PIPELINE START: start_recording() called ===");
    DebugLogger::log_info(&format!("Recording params: spoken_lang={}, translation_lang={}, endpoint={}, stt_model={}, auto_mute={}, translation_enabled={}, text_insertion_enabled={}", 
        spoken_language, translation_language, api_endpoint, stt_model, auto_mute, translation_enabled, text_insertion_enabled));
    
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
            push_to_talk: "".to_string(), // Not used in recording
            hands_free: "".to_string(), // Not used in recording
        },
        auto_mute,
        translation_enabled,
        debug_logging: false, // Will be set properly by frontend debug logging state
        text_insertion_enabled,
    };
    
    // Request the audio manager (single-thread owner) to start capture and return the receiver
    DebugLogger::log_info("Requesting audio manager to start capture");
    let (reply_tx, reply_rx) = std_mpsc::channel();
    {
        let sender = audio_manager.lock().map_err(|e| e.to_string())?;
        sender.send(AudioManagerCommand::Start { reply: reply_tx }).map_err(|e| {
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
    
    // Set recording state to true
    {
        let mut state = recording_state.inner().lock().map_err(|e| e.to_string())?;
        *state = true;
        DebugLogger::log_info("Recording state set to true");
    }

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
    // Use the managed insertion suppression so register_hotkeys can observe it
    let insertion_suppression = app.state::<InsertionSuppression>().inner().clone();

    // Spawn a dedicated background task that performs the blocking insertions
    // using spawn_blocking so it doesn't block the Tokio runtime.
    let text_insertion_service_for_worker = text_insertion_service.clone();
    let insertion_ctrl_tx_for_worker = insertion_ctrl_tx.clone();
    let insertion_suppression_for_worker = insertion_suppression.clone();
    tokio::spawn(async move {
        DebugLogger::log_info("TEXT_INSERTION_WORKER: started");
        while let Some(text) = text_insertion_rx.recv().await {
            DebugLogger::log_info(&format!("TEXT_INSERTION_WORKER: received text (len={}) to insert", text.len()));
            // Signal insertion start
            let _ = insertion_ctrl_tx_for_worker.send(true);
            // Also set the suppression flag for consumers that check it directly
            {
                let mut s = insertion_suppression_for_worker.lock().unwrap();
                *s = true;
            }
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
            {
                let mut s = insertion_suppression_for_worker.lock().unwrap();
                *s = false;
            }
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
        
        DebugLogger::log_info("About to enter audio chunk processing loop");
        DebugLogger::log_info("Waiting for first audio chunk...");
        
        // Aggregation state: accumulate text and flush on utterance boundary
        use std::time::{Duration, Instant};
        let mut agg_text = String::new();
        let mut last_speech = Instant::now();
        let idle_threshold = Duration::from_millis(700);
        let mut utterance_inserted = false; // Track if current utterance was already inserted

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
                    // Periodically check for stop and idle flush
                    let stop = {
                        let state = recording_state_clone.lock().unwrap();
                        !*state
                    };
                    if stop {
                        DebugLogger::log_info("STOP_REASON: Recording state set to false (timeout check), breaking processing loop");
                        break;
                    }
                    if !agg_text.trim().is_empty() && last_speech.elapsed() >= idle_threshold && !utterance_inserted {
                        DebugLogger::log_info("Idle timeout reached, flushing aggregated text (no explicit silence chunk)");
                        let raw_text = agg_text.clone();
                        let final_text = if let Some(ref translation_service) = translation_service {
                            match translation_service.process_text(
                                &agg_text,
                                &settings.spoken_language,
                                &settings.translation_language,
                                settings.translation_enabled
                            ).await {
                                Ok(processed_text) => processed_text,
                                Err(e) => { DebugLogger::log_pipeline_error("translation", &e); agg_text.clone() }
                            }
                        } else { agg_text.clone() };
                        DebugLogger::log_info("TEXT_INSERTION: queueing text for insertion (idle flush)");
                        if settings.text_insertion_enabled {
                            if let Err(e) = text_insertion_tx.send(final_text.clone()) {
                                DebugLogger::log_pipeline_error("text_insertion", &format!("failed to queue text (idle flush): {}", e));
                            } else {
                                DebugLogger::log_info("TEXT_INSERTION: queued (idle flush)");
                                utterance_inserted = true; // Mark as inserted
                            }
                        } else {
                            DebugLogger::log_info("TEXT_INSERTION: skipped (text insertion disabled)");
                        }
                        let _ = app.emit("transcribed-text", serde_json::json!({
                            "raw": raw_text,
                            "final": final_text
                        }));
                        let _ = app.emit("processing-audio", false);
                        agg_text.clear();
                        // Note: Don't reset utterance_inserted here, wait for next speech
                    }
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
            
            // Handle silence marker to flush utterance if idle long enough
            if matches!(audio_chunk.chunk_type, crate::audio::ChunkType::SilenceChunk) {
                if !agg_text.trim().is_empty() && last_speech.elapsed() >= idle_threshold && !utterance_inserted {
                    DebugLogger::log_info(&format!("Idle >= {}ms, flushing utterance", idle_threshold.as_millis()));
                    let raw_text = agg_text.clone();
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

                    // Insert text into focused application once per utterance
                    DebugLogger::log_info("TEXT_INSERTION: queueing text for insertion (silence flush)");
                    if settings.text_insertion_enabled {
                        if let Err(e) = text_insertion_tx.send(final_text.clone()) {
                            DebugLogger::log_pipeline_error("text_insertion", &format!("failed to queue text (silence flush): {}", e));
                            let _ = app.emit("processing-error", format!("Text insertion queue error: {}", e));
                        } else {
                            DebugLogger::log_text_insertion(&final_text, true, None);
                            DebugLogger::log_info("TEXT_INSERTION: queued (silence flush)");
                            utterance_inserted = true; // Mark as inserted
                        }
                    } else {
                        DebugLogger::log_info("TEXT_INSERTION: skipped (text insertion disabled)");
                    }
                    let _ = app.emit("transcribed-text", serde_json::json!({
                        "raw": raw_text,
                        "final": final_text
                    }));
                    let _ = app.emit("processing-audio", false);
                    agg_text.clear();
                    // Note: Don't reset utterance_inserted here, wait for next speech
                }
                continue;
            }

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
            match stt_service.transcribe_chunk(audio_chunk.data, audio_chunk.sample_rate).await {
                Ok(transcribed_text) => {
                    DebugLogger::log_transcription_response(true, Some(&transcribed_text), None);
                    if !transcribed_text.trim().is_empty() {
                        // If this is the start of a new utterance, reset the insertion flag
                        if agg_text.is_empty() {
                            utterance_inserted = false;
                        }
                        append_dedup(&mut agg_text, &transcribed_text);
                        last_speech = Instant::now();
                        DebugLogger::log_info(&format!("Aggregated text length now: {}", agg_text.len()));
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
        
        // Final flush if any text remains
        if !agg_text.trim().is_empty() && !utterance_inserted {
            let raw_text = agg_text.clone();
            let final_text = if let Some(ref translation_service) = translation_service {
                match translation_service.process_text(
                    &agg_text,
                    &settings.spoken_language,
                    &settings.translation_language,
                    settings.translation_enabled
                ).await {
                    Ok(processed_text) => processed_text,
                    Err(_) => agg_text.clone(),
                }
            } else {
                agg_text.clone()
            };
            DebugLogger::log_info("TEXT_INSERTION: queueing text for insertion (final flush)");
            if settings.text_insertion_enabled {
                if let Err(e) = text_insertion_tx.send(final_text.clone()) {
                    DebugLogger::log_pipeline_error("text_insertion", &format!("failed to queue text (final flush): {}", e));
                } else {
                    DebugLogger::log_info("TEXT_INSERTION: queued (final flush)");
                }
            } else {
                DebugLogger::log_info("TEXT_INSERTION: skipped (text insertion disabled)");
            }
            let _ = app.emit("transcribed-text", serde_json::json!({
                "raw": raw_text,
                "final": final_text
            }));
        }

        // Clean up recording state
        {
            let mut state = recording_state_clone.lock().unwrap();
            *state = false;
            DebugLogger::log_info("RECORDING_STATE_CHANGE: Set to false in pipeline cleanup (natural termination)");
            DebugLogger::log_info("Recording state set to false");
        }
        
        DebugLogger::log_info("Emitting recording-stopped event to frontend");
        let _ = app.emit("recording-stopped", ());
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
    audio_manager: State<'_, AudioManagerHandle>
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
    let user_initiated = if let Ok(suppress) = app.state::<InsertionSuppression>().inner().lock() {
        !*suppress  // If not suppressed, it's likely user initiated
    } else {
        true
    };
    
    DebugLogger::log_info(&format!("STOP_RECORDING_CALLED: user_initiated={}", user_initiated));
    // If a text insertion is in progress, ignore stop requests to avoid
    // aborting the recording mid-insert (text insertion runs in background).
    if let Ok(suppress) = app.state::<InsertionSuppression>().inner().lock() {
        if *suppress {
            DebugLogger::log_info("stop_recording ignored because text insertion is in progress (InsertionSuppression=true)");
            return Ok(());
        }
    }
    // If we're not currently recording, ignore duplicate stop requests.
    // Also implement a short cooldown so rapid repeated Stop commands are dropped.
    let cooldown_ms = 300u128;
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
        if let Some(push_to_talk) = hotkeys.get("pushToTalk").and_then(|v| v.as_str()) {
            if push_to_talk.is_empty() {
                errors.push("Push to talk hotkey cannot be empty".to_string());
            }
        }
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
            }
            Ok(false) => {
                let _ = window.show();
                let _ = window.set_focus();
            }
            Err(_) => {
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
    let settings = AppSettings::default(); // Only need the methods, not loaded settings
    settings.store_api_key(&app, api_key)?;
    DebugLogger::log_info("API key stored successfully in backend");
    Ok(())
}

#[tauri::command]
async fn get_api_key(app: AppHandle) -> Result<String, String> {
    let settings = AppSettings::default(); // Only need the methods, not loaded settings
    settings.get_api_key(&app)
}

#[tauri::command]
async fn debug_api_key_info(app: AppHandle) -> Result<serde_json::Value, String> {
    let settings = AppSettings::default();
    settings.debug_api_key_info(&app)
}

#[tauri::command]
async fn has_api_key(app: AppHandle) -> Result<bool, String> {
    let settings = AppSettings::default(); // Only need the methods, not loaded settings
    Ok(settings.has_api_key(&app))
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
    spoken_language: String,
    translation_language: String,
    audio_device: String,
    theme: String,
    api_endpoint: String,
    api_key: String,
    stt_model: String,
    translation_model: String,
    auto_mute: bool,
    translation_enabled: bool,
    debug_logging: bool,
    push_to_talk_hotkey: String,
    hands_free_hotkey: String,
    text_insertion_enabled: bool
) -> Result<(), String> {
    // Log the settings being saved (without logging the API key for security)
    DebugLogger::log_info(&format!("SETTINGS_SAVE: spoken_language={}, translation_language={}, audio_device={}, theme={}, api_endpoint={}, stt_model={}, translation_model={}, api_key_provided={}, auto_mute={}, translation_enabled={}, debug_logging={}, push_to_talk={}, hands_free={}, text_insertion_enabled={}", 
        spoken_language, translation_language, audio_device, theme, api_endpoint, stt_model, translation_model, !api_key.is_empty(), auto_mute, translation_enabled, debug_logging, push_to_talk_hotkey, hands_free_hotkey, text_insertion_enabled));

    // Store API key securely if provided and not empty
    // (Note: we now send empty string for security, so API key is stored separately via store_api_key command)
    if !api_key.is_empty() {
        let settings = AppSettings::default();
        settings.store_api_key(&app, api_key)?;
        DebugLogger::log_info("API key stored securely in backend");
    }

    // Re-initialize debug logging with the new state
    DebugLogger::init_with_state(&app, debug_logging)?;
    
    Ok(())
}

#[tauri::command]
async fn init_debug_logging(app: AppHandle, enabled: bool) -> Result<(), String> {
    DebugLogger::log_info(&format!("Debug logging manually set to: {}", enabled));
    DebugLogger::init_with_state(&app, enabled)?;
    Ok(())
}

#[tauri::command]
fn export_legacy_api_key(app: AppHandle) -> Result<Option<String>, String> {
    let settings = AppSettings::default();
    settings.export_legacy_api_key(&app)
}

#[tauri::command]
fn delete_legacy_api_key(app: AppHandle) -> Result<(), String> {
    let settings = AppSettings::default();
    settings.delete_legacy_api_key(&app)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        // Register Stronghold plugin for encrypted at-rest storage (JS guest APIs available)
        .setup(|app| {
            // derive salt path for argon2 KDF used by the plugin
            let salt_path = app
                .path()
                .app_local_data_dir()
                .expect("could not resolve app local data path")
                .join("salt.txt");
            // register the plugin using the Builder::with_argon2 helper
            let _ = app.handle().plugin(tauri_plugin_stronghold::Builder::with_argon2(&salt_path).build());
            Ok(())
        })
        .manage(Mutex::<HashMap<String, String>>::new(HashMap::new()))
        .manage(Arc::new(Mutex::new(false)) as RecordingState)
        .manage(Arc::new(Mutex::new(None)) as AudioStopSender)
    .manage(Arc::new(Mutex::new(None)) as LastStopTime)
        .manage(Arc::new(Mutex::new(None)) as LastHotkey)
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
                        AudioManagerCommand::Start { reply } => {
                            DebugLogger::log_info("Audio manager received Start command");
                            // If already started, return error
                            if audio_capture_opt.is_some() {
                                let _ = reply.send(Err("Audio capture already started".to_string()));
                                continue;
                            }
                            // Create and start capture (only once)
                            let mut capture = AudioCapture::new();
                            match capture.start_capture() {
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
            export_legacy_api_key,
            delete_legacy_api_key,
            get_available_audio_devices,
            test_audio_capture,
            get_recording_status,
            get_debug_logs,
            clear_debug_logs,
            get_log_file_path,
            get_data_directory_info,
            frontend_log,
            translate_text,
            load_settings_from_frontend,
            save_settings_from_frontend,
            init_debug_logging
        ])
        .setup(|app| {
            // Initialize debug logging first (disabled by default, will be enabled by frontend)
            if let Err(e) = DebugLogger::init(&app.handle()) {
                eprintln!("Failed to initialize debug logging: {}", e);
            }
            
            DebugLogger::log_info("TalkToMe application starting up");
            DebugLogger::log_info("Using localStorage-based settings (no settings.json file)");
            DebugLogger::log_info("Initialized with default settings for tray menu");
            
            // Create a simple system tray menu without problematic dynamic submenus
            let tray_menu = {
                let show_hide = MenuItemBuilder::with_id("show_hide", "Show/Hide TalkToMe").build(app)?;
                let start_recording = MenuItemBuilder::with_id("start_recording", "Start Recording").build(app)?;
                let stop_recording = MenuItemBuilder::with_id("stop_recording", "Stop Recording").enabled(false).build(app)?;
                
                let preferences = MenuItemBuilder::with_id("preferences", "Preferences").build(app)?;
                let api_settings = MenuItemBuilder::with_id("api_settings", "API Settings").build(app)?;
                let language_settings = MenuItemBuilder::with_id("language_settings", "Language Settings").build(app)?;
                let audio_settings = MenuItemBuilder::with_id("audio_settings", "Audio Settings").build(app)?;
                let about = MenuItemBuilder::with_id("about", "About TalkToMe").build(app)?;
                let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

                MenuBuilder::new(app)
                    .items(&[
                        &show_hide,
                        &start_recording,
                        &stop_recording,
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
                        "start_recording" => {
                            // Emit event to frontend to start recording
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-start-recording", ());
                            }
                        }
                        "stop_recording" => {
                            // Emit event to frontend to stop recording
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-stop-recording", ());
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
                            // Prevent the default close behavior
                            api.prevent_close();
                            // Hide the window instead
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
