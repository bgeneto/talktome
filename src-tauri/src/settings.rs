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
        }
    }
}

impl AppSettings {
    pub fn load(app_handle: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let settings_path = Self::get_settings_path(app_handle)?;
        
        if settings_path.exists() {
            let content = fs::read_to_string(settings_path)?;
            let settings: AppSettings = serde_json::from_str(&content)?;
            Ok(settings)
        } else {
            // Return default settings if file doesn't exist
            let default_settings = AppSettings::default();
            default_settings.save(app_handle)?;
            Ok(default_settings)
        }
    }

    pub fn save(&self, app_handle: &AppHandle) -> Result<(), String> {
        let settings_path = Self::get_settings_path(app_handle)?;
        
        // Create directory if it doesn't exist
        if let Some(parent) = settings_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(settings_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

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

    fn get_settings_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
        Ok(app_dir.join("settings.json"))
    }

    fn get_api_key_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
        Ok(app_dir.join("api.key"))
    }
}
