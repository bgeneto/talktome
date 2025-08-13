use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    Manager, Emitter, AppHandle,
};
mod settings;
use settings::AppSettings;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Command to handle recording state
#[tauri::command]
fn toggle_recording(_app: tauri::AppHandle) -> Result<bool, String> {
    // This will be connected to the frontend recording state
    Ok(true)
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
        .invoke_handler(tauri::generate_handler![greet, toggle_recording, toggle_window, quit_app, update_spoken_language, update_translation_language, update_audio_device])
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
