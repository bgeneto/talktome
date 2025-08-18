use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use serde_json::json;
use keyring::Entry;

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
    // SECURITY: API key is NEVER stored in this struct or localStorage
    // It's handled separately via secure file storage (backend only)
    // Frontend stores it only in memory during runtime
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Hotkeys {
    pub push_to_talk: String,
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
                push_to_talk: "Ctrl+Win".to_string(),
                hands_free: "Ctrl+Win+Space".to_string(),
            },
            auto_mute: true,
            translation_enabled: false,
            debug_logging: false,
        }
    }
}

impl AppSettings {
    // Note: load() and save() methods removed - now using localStorage-only approach
    // AppSettings struct is kept for internal backend operations like tray menu updates

    /// Get API key from secure file storage (temporary solution, can be upgraded to Stronghold later)
    pub fn get_api_key(&self, app_handle: &AppHandle) -> Result<String, String> {
        // First try OS keyring
        let service = "talktome_api_key";
        let username = whoami::username();
        let entry = Entry::new(service, &username);
        match entry.get_password() {
            Ok(pw) => return Ok(pw),
            Err(_) => {
                // Fallthrough to try legacy file-based key (migration path)
            }
        }

        // Legacy fallback: check plain api.key file for migration support
        let api_key_path = Self::get_api_key_path(app_handle)?;
        if api_key_path.exists() {
            let content = fs::read_to_string(&api_key_path)
                .map_err(|e| format!("Failed to read legacy API key at {}: {}", api_key_path.display(), e))?;
            let key = content.trim().to_string();
            // Attempt to store in keyring for future secure access
            if let Err(e) = entry.set_password(&key) {
                // Log but still return the key so caller can continue
                return Err(format!("Failed to migrate API key to secure storage: {}", e));
            }
            // On successful migration, attempt to securely delete the legacy file
            let _ = fs::remove_file(&api_key_path);
            return Ok(key);
        }

        Err(format!("API key not found in secure storage or legacy file"))
    }

    /// Store API key securely
    pub fn store_api_key(&self, app_handle: &AppHandle, api_key: String) -> Result<(), String> {
    // Store in OS keyring
    let service = "talktome_api_key";
    let username = whoami::username();
    let entry = Entry::new(service, &username);
    entry.set_password(api_key.trim()).map_err(|e| format!("Failed to store API key in secure storage: {}", e))?;
    Ok(())
    }

    /// Check if API key exists
    pub fn has_api_key(&self, app_handle: &AppHandle) -> bool {
        self.get_api_key(app_handle).is_ok()
    }

    fn get_api_key_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let app_dir = Self::get_portable_data_dir(app_handle)?;
        Ok(app_dir.join("api.key"))
    }

    /// Get portable data directory - tries local first, falls back to app_data_dir
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
        let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
        Ok(app_dir)
    }

    /// Diagnostic helper for debugging API key storage issues
    /// Returns JSON with path, exists, size (bytes) and a masked preview of the key
    pub fn debug_api_key_info(&self, app_handle: &AppHandle) -> Result<serde_json::Value, String> {
        let api_key_path = Self::get_api_key_path(app_handle)?;
        let exists = api_key_path.exists();
        if exists {
            match fs::metadata(&api_key_path) {
                Ok(meta) => {
                    let size = meta.len();
                    match fs::read_to_string(&api_key_path) {
                        Ok(content) => {
                            let trimmed = content.trim();
                            let preview = if trimmed.len() <= 10 {
                                // show as-is but masked
                                format!("{}", "*".repeat(trimmed.len()))
                            } else {
                                let start = &trimmed[..4];
                                let end = &trimmed[trimmed.len()-4..];
                                format!("{}{}{}", start, "*".repeat(8), end)
                            };
                            Ok(json!({
                                "path": api_key_path.display().to_string(),
                                "exists": true,
                                "size": size,
                                "preview": preview
                            }))
                        }
                        Err(e) => Err(format!("Failed to read API key at {}: {}", api_key_path.display(), e)),
                    }
                }
                Err(e) => Err(format!("Failed to stat API key at {}: {}", api_key_path.display(), e)),
            }
        } else {
            Ok(json!({
                "path": api_key_path.display().to_string(),
                "exists": false,
            }))
        }
    }

    /// Export the legacy plain-text API key (if present) so the frontend can migrate it into Stronghold.
    /// Returns Ok(Some(key)) if a legacy key was found, Ok(None) if not found.
    pub fn export_legacy_api_key(&self, app_handle: &AppHandle) -> Result<Option<String>, String> {
        let api_key_path = Self::get_api_key_path(app_handle)?;
        if api_key_path.exists() {
            let content = fs::read_to_string(&api_key_path)
                .map_err(|e| format!("Failed to read legacy API key at {}: {}", api_key_path.display(), e))?;
            Ok(Some(content.trim().to_string()))
        } else {
            Ok(None)
        }
    }

    /// Delete the legacy plain-text API key file after successful migration.
    pub fn delete_legacy_api_key(&self, app_handle: &AppHandle) -> Result<(), String> {
        let api_key_path = Self::get_api_key_path(app_handle)?;
        if api_key_path.exists() {
            fs::remove_file(&api_key_path).map_err(|e| format!("Failed to delete legacy API key at {}: {}", api_key_path.display(), e))?;
        }
        Ok(())
    }
}
