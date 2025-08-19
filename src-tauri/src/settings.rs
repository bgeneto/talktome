use serde::{Deserialize, Serialize};
// std::fs was used by legacy file-based API key handling which has been removed
use keyring::Entry;
use serde_json::json;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub spoken_language: String,
    pub translation_language: String,
    pub audio_device: String,
    pub theme: String,
    pub auto_save: bool,
    pub api_endpoint: String,
    pub stt_model: String,
    pub translation_model: String,
    pub hotkeys: Hotkeys,
    pub auto_mute: bool,
    pub translation_enabled: bool,
    pub debug_logging: bool,
    pub text_insertion_enabled: bool,
    pub audio_chunking_enabled: bool,
    pub max_recording_time_minutes: u32,
    // SECURITY: API key is NEVER stored in this struct or localStorage
    // It's handled separately via secure file storage (backend only)
    // Frontend stores it only in memory during runtime
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Hotkeys {
    pub hands_free: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            spoken_language: "auto".to_string(),
            translation_language: "none".to_string(),
            audio_device: "default".to_string(),
            theme: "auto".to_string(),
            auto_save: true,
            api_endpoint: "https://api.openai.com/v1".to_string(),
            stt_model: "whisper-large-v3".to_string(),
            translation_model: "gpt-3.5-turbo".to_string(),
            hotkeys: Hotkeys {
                hands_free: "Ctrl+Shift+Space".to_string(),
            },
            auto_mute: true,
            translation_enabled: false,
            debug_logging: false,
            text_insertion_enabled: true,
            audio_chunking_enabled: false, // Default to false - single recording mode only
            max_recording_time_minutes: 5, // Default to 5 minutes maximum recording time
        }
    }
}

impl AppSettings {
    // Note: load() and save() methods removed - now using localStorage-only approach
    // AppSettings struct is kept for internal backend operations like tray menu updates

    /// Get API key from secure storage
    pub fn get_api_key(&self, _app_handle: &AppHandle) -> Result<String, String> {
        // Try OS keyring first
        let service = "talktome_api_key";
        let username = whoami::username();
        let entry = Entry::new(service, &username);

        match entry.get_password() {
            Ok(pw) => {
                println!("API_KEY: Successfully retrieved from keyring");
                return Ok(pw);
            }
            Err(e) => {
                println!("API_KEY: Failed to get from keyring: {}", e);
                return Err("API key not found in secure storage".to_string());
            }
        }
    }

    /// Store API key securely (keyring only, no file fallback)
    pub fn store_api_key(&self, _app_handle: &AppHandle, api_key: String) -> Result<(), String> {
        // Validate API key
        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return Err("API key cannot be empty".to_string());
        }

        // Store in OS keyring
        let service = "talktome_api_key";
        let username = whoami::username();
        let entry = Entry::new(service, &username);

        match entry.set_password(trimmed_key) {
            Ok(_) => {
                println!("API_KEY: Successfully stored in keyring");
                Ok(())
            }
            Err(e) => {
                println!("API_KEY: Failed to store in keyring: {}", e);
                // Do NOT fallback to file-based storage for security reasons
                Err(format!("Failed to store API key in secure storage: {}", e))
            }
        }
    }

    /// Check if API key exists
    pub fn has_api_key(&self, app_handle: &AppHandle) -> bool {
        self.get_api_key(app_handle).is_ok()
    }

    /// Get portable data directory - tries local first, falls back to app_data_dir
    #[allow(dead_code)]
    fn get_portable_data_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
        // Try to get the executable directory first for portable mode
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let portable_dir = exe_dir.join("data");

                // Check if we can write to the exe directory (portable mode)
                if let Ok(_) = std::fs::create_dir_all(&portable_dir) {
                    if portable_dir.exists() {
                        return Ok(portable_dir);
                    }
                }
            }
        }

        // Fallback to app data directory
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| e.to_string())?;
        Ok(app_dir)
    }

    /// Diagnostic helper for debugging API key storage issues
    /// Returns JSON with path, exists, size (bytes) and a masked preview of the key
    pub fn debug_api_key_info(&self, _app_handle: &AppHandle) -> Result<serde_json::Value, String> {
        // Report whether a password exists in the OS keyring and basic masked info
        let service = "talktome_api_key";
        let username = whoami::username();
        let entry = Entry::new(service, &username);

        match entry.get_password() {
            Ok(pw) => {
                let len = pw.len();
                let preview = if len <= 10 {
                    "*".repeat(len)
                } else {
                    format!("{}{}{}", &pw[..4], "*".repeat(8), &pw[len - 4..])
                };
                Ok(json!({
                    "service": service,
                    "username": username,
                    "exists": true,
                    "length": len,
                    "preview": preview
                }))
            }
            Err(_) => Ok(json!({
                "service": service,
                "username": username,
                "exists": false
            })),
        }
    }
}
