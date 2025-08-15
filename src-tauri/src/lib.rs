use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    Manager, Emitter, AppHandle, State,
};
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut, ShortcutState, GlobalShortcutExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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

// Global state to track registered hotkeys and active recording
type HotkeyRegistry = Mutex<HashMap<String, String>>;
type RecordingState = Arc<Mutex<bool>>;

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
            .on_shortcut(shortcut, move |_app, _sc, ev| {
                let state = match ev.state {
                    ShortcutState::Pressed => "pressed",
                    ShortcutState::Released => "released",
                };
                if action_clone == "push_to_talk" || action_clone == "hands_free" {
                    if state == "pressed" {
                        // Start recording - need to get recording_state somehow
                        let _ = app_for_emit.emit("start-recording-from-hotkey", ());
                    } else {
                        // Stop recording
                        let _ = app_for_emit.emit("stop-recording-from-hotkey", ());
                    }
                } else {
                                    let _ = app_for_emit.emit(
                                        "hotkey-triggered",
                                        serde_json::json!({
                                            "action": action_clone,
                                            "state": state,
                                        }),
                                    );
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
async fn start_recording(app: AppHandle, recording_state: State<'_, RecordingState>) -> Result<(), String> {
    // Check if already recording
    {
        let state = recording_state.inner().lock().map_err(|e| e.to_string())?;
        if *state {
            return Err("Already recording".to_string());
        }
    }

    // Load settings
    let settings = AppSettings::load(&app).map_err(|e| e.to_string())?;
    
    // Create audio capture instance
    let mut audio_capture = AudioCapture::new();
    
    // Start capturing audio
    let mut audio_rx = audio_capture.start_capture().map_err(|e| e.to_string())?;
    
    // Set recording state to true
    {
        let mut state = recording_state.inner().lock().map_err(|e| e.to_string())?;
        *state = true;
    }
    
    // Create services
    let stt_service = STTService::new(settings.api_endpoint.clone(), settings.api_key.clone());
    let translation_service = if settings.translation_language != "none" {
        Some(TranslationService::new(settings.api_endpoint.clone(), settings.api_key.clone()))
    } else {
        None
    };
    let text_insertion_service = TextInsertionService::new();
    
    // Clone app handle for the processing task
    let app_clone = app.clone();
    let recording_state_clone = recording_state.inner().clone();
    
    // Spawn task to process audio chunks
    tokio::spawn(async move {
        let stt_service = stt_service;
        let translation_service = translation_service;
        let text_insertion_service = text_insertion_service;
        let app = app_clone;
        let settings = settings;
        
        // Process audio chunks
        while let Some(audio_chunk) = audio_rx.recv().await {
            // Check if recording has been stopped
            {
                let state = recording_state_clone.lock().unwrap();
                if !*state {
                    break;
                }
            }
            
            // Skip empty chunks
            if audio_chunk.is_empty() {
                continue;
            }
            
            // Transcribe audio chunk
            match stt_service.transcribe_chunk(audio_chunk, 48000).await {
                Ok(transcribed_text) => {
                    if !transcribed_text.trim().is_empty() {
                        // Translate text if needed
                        let final_text = if let Some(ref translation_service) = translation_service {
                            if settings.translation_language != "none" {
                                match translation_service.translate_text(
                                    &transcribed_text,
                                    &settings.spoken_language,
                                    &settings.translation_language
                                ).await {
                                    Ok(translated_text) => translated_text,
                                    Err(e) => {
                                        eprintln!("Translation error: {}", e);
                                        transcribed_text.clone() // Fallback to original text
                                    }
                                }
                            } else {
                                transcribed_text.clone()
                            }
                        } else {
                            transcribed_text.clone()
                        };
                        
                        // Insert text into focused application
                        if let Err(e) = text_insertion_service.insert_text(&final_text) {
                            eprintln!("Text insertion error: {}", e);
                        }
                        
                        // Emit event to frontend with transcribed text
                        let _ = app.emit("transcribed-text", &final_text);
                    }
                }
                Err(e) => {
                    eprintln!("Transcription error: {}", e);
                }
            }
        }
        
        // Clean up recording state
        {
            let mut state = recording_state_clone.lock().unwrap();
            *state = false;
        }
    });
    
    Ok(())
}

// Command to stop recording
#[tauri::command]
fn stop_recording(recording_state: State<'_, RecordingState>) -> Result<(), String> {
    let mut state = recording_state.inner().lock().map_err(|e| e.to_string())?;
    *state = false;
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

#[tauri::command]
async fn update_spoken_language(app: AppHandle, language: String) -> Result<(), String> {
    // Update settings and tray menu
    let mut settings = AppSettings::load(&app).map_err(|e| e.to_string())?;
    settings.spoken_language = language;
    settings.save(&app)?;
    
    // Update tray menu
    update_tray_menu(&app, &settings).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn update_translation_language(app: AppHandle, language: String) -> Result<(), String> {
    // Update settings and tray menu
    let mut settings = AppSettings::load(&app).map_err(|e| e.to_string())?;
    settings.translation_language = language;
    settings.save(&app)?;
    
    // Update tray menu
    update_tray_menu(&app, &settings).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
async fn update_audio_device(app: AppHandle, device: String) -> Result<(), String> {
    // Update settings and tray menu
    let mut settings = AppSettings::load(&app).map_err(|e| e.to_string())?;
    settings.audio_device = device;
    settings.save(&app)?;
    
    // Update tray menu
    update_tray_menu(&app, &settings).map_err(|e| e.to_string())?;
    
    Ok(())
}

fn update_tray_menu(app: &AppHandle, settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    // Get existing tray icon
    let tray = app.tray_by_id("main-tray").ok_or("Tray icon not found")?;
    
    // Spoken Language submenu items with checkmarks
    let spoken_auto_text = if settings.spoken_language == "auto" { 
        "Auto-detect 🌐 ✓" 
    } else { 
        "Auto-detect 🌐" 
    };
    let spoken_en_text = if settings.spoken_language == "en" { 
        "English 🇺🇸 ✓" 
    } else { 
        "English 🇺🇸" 
    };
    let spoken_pt_text = if settings.spoken_language == "pt" { 
        "Portuguese 🇵🇹 ✓" 
    } else { 
        "Portuguese 🇵🇹" 
    };
    let spoken_es_text = if settings.spoken_language == "es" { 
        "Spanish 🇪🇸 ✓" 
    } else { 
        "Spanish 🇪🇸" 
    };
    let spoken_fr_text = if settings.spoken_language == "fr" { 
        "French 🇫🇷 ✓" 
    } else { 
        "French 🇫🇷" 
    };
    let spoken_de_text = if settings.spoken_language == "de" { 
        "German 🇩🇪 ✓" 
    } else { 
        "German 🇩🇪" 
    };
    
    let spoken_auto = MenuItemBuilder::with_id("spoken_auto", spoken_auto_text).build(app)?;
    let spoken_en = MenuItemBuilder::with_id("spoken_en", spoken_en_text).build(app)?;
    let spoken_pt = MenuItemBuilder::with_id("spoken_pt", spoken_pt_text).build(app)?;
    let spoken_es = MenuItemBuilder::with_id("spoken_es", spoken_es_text).build(app)?;
    let spoken_fr = MenuItemBuilder::with_id("spoken_fr", spoken_fr_text).build(app)?;
    let spoken_de = MenuItemBuilder::with_id("spoken_de", spoken_de_text).build(app)?;
    let spoken_language_submenu = SubmenuBuilder::new(app, "Spoken Language")
        .items(&[&spoken_auto, &spoken_en, &spoken_pt, &spoken_es, &spoken_fr, &spoken_de])
        .build()?;
    
    // Translation Language submenu items with checkmarks
    let translation_none_text = if settings.translation_language == "none" { 
        "None (Disable Translation) ✓" 
    } else { 
        "None (Disable Translation)" 
    };
    let translation_en_text = if settings.translation_language == "en" { 
        "English 🇺🇸 ✓" 
    } else { 
        "English 🇺🇸" 
    };
    let translation_pt_text = if settings.translation_language == "pt" { 
        "Portuguese 🇵🇹 ✓" 
    } else { 
        "Portuguese 🇵🇹" 
    };
    let translation_es_text = if settings.translation_language == "es" { 
        "Spanish 🇪🇸 ✓" 
    } else { 
        "Spanish 🇪🇸" 
    };
    let translation_fr_text = if settings.translation_language == "fr" { 
        "French 🇫🇷 ✓" 
    } else { 
        "French 🇫🇷" 
    };
    let translation_de_text = if settings.translation_language == "de" { 
        "German 🇩🇪 ✓" 
    } else { 
        "German 🇩🇪" 
    };
    
    let translation_none = MenuItemBuilder::with_id("translation_none", translation_none_text).build(app)?;
    let translation_en = MenuItemBuilder::with_id("translation_en", translation_en_text).build(app)?;
    let translation_pt = MenuItemBuilder::with_id("translation_pt", translation_pt_text).build(app)?;
    let translation_es = MenuItemBuilder::with_id("translation_es", translation_es_text).build(app)?;
    let translation_fr = MenuItemBuilder::with_id("translation_fr", translation_fr_text).build(app)?;
    let translation_de = MenuItemBuilder::with_id("translation_de", translation_de_text).build(app)?;
    let translation_language_submenu = SubmenuBuilder::new(app, "Translation Language")
        .items(&[&translation_none, &translation_en, &translation_pt, &translation_es, &translation_fr, &translation_de])
        .build()?;
    
    // Audio Settings submenu items with checkmarks
    let audio_default_text = if settings.audio_device == "default" { 
        "Default Microphone ✓" 
    } else { 
        "Default Microphone" 
    };
    let audio_headset_text = if settings.audio_device == "mic1" { 
        "Headset Microphone (USB) ✓" 
    } else { 
        "Headset Microphone (USB)" 
    };
    let audio_builtin_text = if settings.audio_device == "mic2" { 
        "Built-in Microphone ✓" 
    } else { 
        "Built-in Microphone" 
    };
    let audio_external_text = if settings.audio_device == "mic3" { 
        "External Microphone (3.5mm) ✓" 
    } else { 
        "External Microphone (3.5mm)" 
    };
    
    let audio_default = MenuItemBuilder::with_id("audio_default", audio_default_text).build(app)?;
    let audio_headset = MenuItemBuilder::with_id("audio_headset", audio_headset_text).build(app)?;
    let audio_builtin = MenuItemBuilder::with_id("audio_builtin", audio_builtin_text).build(app)?;
    let audio_external = MenuItemBuilder::with_id("audio_external", audio_external_text).build(app)?;
    let audio_settings_submenu = SubmenuBuilder::new(app, "Audio Input")
        .items(&[&audio_default, &audio_headset, &audio_builtin, &audio_external])
        .build()?;
    
    // Other menu items
    let show_hide = MenuItemBuilder::with_id("show_hide", "Show/Hide TalkToMe").build(app)?;
    let start_recording = MenuItemBuilder::with_id("start_recording", "Start Recording").build(app)?;
    let stop_recording = MenuItemBuilder::with_id("stop_recording", "Stop Recording").enabled(false).build(app)?;
    let recording_submenu = SubmenuBuilder::new(app, "Recording")
        .items(&[&start_recording, &stop_recording])
        .build()?;
    
    let preferences = MenuItemBuilder::with_id("preferences", "Preferences...").build(app)?;
    let language_settings = MenuItemBuilder::with_id("language_settings", "Language Settings").build(app)?;
    let audio_settings = MenuItemBuilder::with_id("audio_settings", "Audio Settings").build(app)?;
    let settings_submenu = SubmenuBuilder::new(app, "Settings")
        .items(&[&preferences, &language_settings, &audio_settings, &spoken_language_submenu, &translation_language_submenu, &audio_settings_submenu])
        .build()?;
    
    let about = MenuItemBuilder::with_id("about", "About TalkToMe").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

    let tray_menu = MenuBuilder::new(app)
        .items(&[
            &show_hide,
            &recording_submenu,
            &settings_submenu,
            &about,
            &quit,
        ])
        .build()?;
    
    tray.set_menu(Some(tray_menu))?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(Mutex::<HashMap<String, String>>::new(HashMap::new()))
        .manage(Arc::new(Mutex::new(false)) as RecordingState)
        .invoke_handler(tauri::generate_handler![greet, start_recording, stop_recording, toggle_window, quit_app, update_spoken_language, update_translation_language, update_audio_device, register_hotkeys, test_stt_api, validate_settings])
        .setup(|app| {
            // Load current settings
            let settings = AppSettings::load(&app.handle()).unwrap_or_default();
            
            // Create the system tray menu with checkmarks
            let tray_menu = {
                // Spoken Language submenu items with checkmarks
                let spoken_auto_text = if settings.spoken_language == "auto" { 
                    "Auto-detect 🌐 ✓" 
                } else { 
                    "Auto-detect 🌐" 
                };
                let spoken_en_text = if settings.spoken_language == "en" { 
                    "English 🇺🇸 ✓" 
                } else { 
                    "English 🇺🇸" 
                };
                let spoken_pt_text = if settings.spoken_language == "pt" { 
                    "Portuguese 🇵🇹 ✓" 
                } else { 
                    "Portuguese 🇵🇹" 
                };
                let spoken_es_text = if settings.spoken_language == "es" { 
                    "Spanish 🇪🇸 ✓" 
                } else { 
                    "Spanish 🇪🇸" 
                };
                let spoken_fr_text = if settings.spoken_language == "fr" { 
                    "French 🇫🇷 ✓" 
                } else { 
                    "French 🇫🇷" 
                };
                let spoken_de_text = if settings.spoken_language == "de" { 
                    "German 🇩🇪 ✓" 
                } else { 
                    "German 🇩🇪" 
                };
                
                let spoken_auto = MenuItemBuilder::with_id("spoken_auto", spoken_auto_text).build(app)?;
                let spoken_en = MenuItemBuilder::with_id("spoken_en", spoken_en_text).build(app)?;
                let spoken_pt = MenuItemBuilder::with_id("spoken_pt", spoken_pt_text).build(app)?;
                let spoken_es = MenuItemBuilder::with_id("spoken_es", spoken_es_text).build(app)?;
                let spoken_fr = MenuItemBuilder::with_id("spoken_fr", spoken_fr_text).build(app)?;
                let spoken_de = MenuItemBuilder::with_id("spoken_de", spoken_de_text).build(app)?;
                let spoken_language_submenu = SubmenuBuilder::new(app, "Spoken Language")
                    .items(&[&spoken_auto, &spoken_en, &spoken_pt, &spoken_es, &spoken_fr, &spoken_de])
                    .build()?;
                
                // Translation Language submenu items with checkmarks
                let translation_none_text = if settings.translation_language == "none" { 
                    "None (Disable Translation) ✓" 
                } else { 
                    "None (Disable Translation)" 
                };
                let translation_en_text = if settings.translation_language == "en" { 
                    "English 🇺🇸 ✓" 
                } else { 
                    "English 🇺🇸" 
                };
                let translation_pt_text = if settings.translation_language == "pt" { 
                    "Portuguese 🇵🇹 ✓" 
                } else { 
                    "Portuguese 🇵🇹" 
                };
                let translation_es_text = if settings.translation_language == "es" { 
                    "Spanish 🇪🇸 ✓" 
                } else { 
                    "Spanish 🇪🇸" 
                };
                let translation_fr_text = if settings.translation_language == "fr" { 
                    "French 🇫🇷 ✓" 
                } else { 
                    "French 🇫🇷" 
                };
                let translation_de_text = if settings.translation_language == "de" { 
                    "German 🇩🇪 ✓" 
                } else { 
                    "German 🇩🇪" 
                };
                
                let translation_none = MenuItemBuilder::with_id("translation_none", translation_none_text).build(app)?;
                let translation_en = MenuItemBuilder::with_id("translation_en", translation_en_text).build(app)?;
                let translation_pt = MenuItemBuilder::with_id("translation_pt", translation_pt_text).build(app)?;
                let translation_es = MenuItemBuilder::with_id("translation_es", translation_es_text).build(app)?;
                let translation_fr = MenuItemBuilder::with_id("translation_fr", translation_fr_text).build(app)?;
                let translation_de = MenuItemBuilder::with_id("translation_de", translation_de_text).build(app)?;
                let translation_language_submenu = SubmenuBuilder::new(app, "Translation Language")
                    .items(&[&translation_none, &translation_en, &translation_pt, &translation_es, &translation_fr, &translation_de])
                    .build()?;
                
                // Audio Settings submenu items with checkmarks
                let audio_default_text = if settings.audio_device == "default" { 
                    "Default Microphone ✓" 
                } else { 
                    "Default Microphone" 
                };
                let audio_headset_text = if settings.audio_device == "mic1" { 
                    "Headset Microphone (USB) ✓" 
                } else { 
                    "Headset Microphone (USB)" 
                };
                let audio_builtin_text = if settings.audio_device == "mic2" { 
                    "Built-in Microphone ✓" 
                } else { 
                    "Built-in Microphone" 
                };
                let audio_external_text = if settings.audio_device == "mic3" { 
                    "External Microphone (3.5mm) ✓" 
                } else { 
                    "External Microphone (3.5mm)" 
                };
                
                let audio_default = MenuItemBuilder::with_id("audio_default", audio_default_text).build(app)?;
                let audio_headset = MenuItemBuilder::with_id("audio_headset", audio_headset_text).build(app)?;
                let audio_builtin = MenuItemBuilder::with_id("audio_builtin", audio_builtin_text).build(app)?;
                let audio_external = MenuItemBuilder::with_id("audio_external", audio_external_text).build(app)?;
                let audio_settings_submenu = SubmenuBuilder::new(app, "Audio Input")
                    .items(&[&audio_default, &audio_headset, &audio_builtin, &audio_external])
                    .build()?;
                
                // Main Settings submenu items
                let preferences = MenuItemBuilder::with_id("preferences", "Preferences...").build(app)?;
                let language_settings = MenuItemBuilder::with_id("language_settings", "Language Settings").build(app)?;
                let audio_settings = MenuItemBuilder::with_id("audio_settings", "Audio Settings").build(app)?;
                let settings_submenu = SubmenuBuilder::new(app, "Settings")
                    .items(&[&preferences, &language_settings, &audio_settings, &spoken_language_submenu, &translation_language_submenu, &audio_settings_submenu])
                    .build()?;
                
                let show_hide = MenuItemBuilder::with_id("show_hide", "Show/Hide TalkToMe").build(app)?;
                let start_recording = MenuItemBuilder::with_id("start_recording", "Start Recording").build(app)?;
                let stop_recording = MenuItemBuilder::with_id("stop_recording", "Stop Recording").enabled(false).build(app)?;
                let recording_submenu = SubmenuBuilder::new(app, "Recording")
                    .items(&[&start_recording, &stop_recording])
                    .build()?;
                
                let about = MenuItemBuilder::with_id("about", "About TalkToMe").build(app)?;
                let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;

                MenuBuilder::new(app)
                    .items(&[
                        &show_hide,
                        &recording_submenu,
                        &settings_submenu,
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
                        // Spoken Language events
                        "spoken_auto" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-spoken-language-change", "auto");
                            }
                        }
                        "spoken_en" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-spoken-language-change", "en");
                            }
                        }
                        "spoken_pt" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-spoken-language-change", "pt");
                            }
                        }
                        "spoken_es" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-spoken-language-change", "es");
                            }
                        }
                        "spoken_fr" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-spoken-language-change", "fr");
                            }
                        }
                        "spoken_de" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-spoken-language-change", "de");
                            }
                        }
                        // Translation Language events
                        "translation_none" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-translation-language-change", "none");
                            }
                        }
                        "translation_en" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-translation-language-change", "en");
                            }
                        }
                        "translation_pt" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-translation-language-change", "pt");
                            }
                        }
                        "translation_es" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-translation-language-change", "es");
                            }
                        }
                        "translation_fr" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-translation-language-change", "fr");
                            }
                        }
                        "translation_de" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-translation-language-change", "de");
                            }
                        }
                        // Audio Input events
                        "audio_default" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-audio-input-change", "default");
                            }
                        }
                        "audio_headset" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-audio-input-change", "mic1");
                            }
                        }
                        "audio_builtin" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-audio-input-change", "mic2");
                            }
                        }
                        "audio_external" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.emit("tray-audio-input-change", "mic3");
                            }
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
