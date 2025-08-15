use serde::{Deserialize, Serialize};
use std::fs;
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
    pub hotkeys: Hotkeys,
    pub auto_mute: bool,
    pub translation_enabled: bool,
    pub debug_logging: bool,
    // Note: api_key is handled separately via Stronghold for security
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
        let api_key_path = Self::get_api_key_path(app_handle)?;
        
        if api_key_path.exists() {
            fs::read_to_string(api_key_path)
                .map_err(|e| format!("Failed to read API key: {}", e))
                .map(|key| key.trim().to_string())
        } else {
            Err("API key not found".to_string())
        }
    }

    /// Store API key securely
    pub fn store_api_key(&self, app_handle: &AppHandle, api_key: String) -> Result<(), String> {
        let api_key_path = Self::get_api_key_path(app_handle)?;
        
        // Create directory if it doesn't exist
        if let Some(parent) = api_key_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        
        fs::write(api_key_path, api_key.trim())
            .map_err(|e| format!("Failed to store API key: {}", e))?;
        
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
}

