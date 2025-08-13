use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
    Manager, Emitter,
};

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, toggle_recording, toggle_window, quit_app])
        .setup(|app| {
            // Create the system tray menu
            let show_hide = MenuItemBuilder::with_id("show_hide", "Show/Hide TalkToMe").build(app)?;
            
            // Recording submenu items
            let start_recording = MenuItemBuilder::with_id("start_recording", "Start Recording").build(app)?;
            let stop_recording = MenuItemBuilder::with_id("stop_recording", "Stop Recording").enabled(false).build(app)?;
            let recording_submenu = SubmenuBuilder::new(app, "Recording")
                .items(&[&start_recording, &stop_recording])
                .build()?;
            
            // Settings submenu items
            let preferences = MenuItemBuilder::with_id("preferences", "Preferences...").build(app)?;
            let language_settings = MenuItemBuilder::with_id("language_settings", "Language Settings").build(app)?;
            let audio_settings = MenuItemBuilder::with_id("audio_settings", "Audio Settings").build(app)?;
            let settings_submenu = SubmenuBuilder::new(app, "Settings")
                .items(&[&preferences, &language_settings, &audio_settings])
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
