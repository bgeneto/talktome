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
    pub api_key: String,
    pub hotkeys: Hotkeys,
    pub auto_mute: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Hotkeys {
    pub push_to_talk: String,
    pub hands_free: String,
    pub emergency_stop: String,
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
            api_key: "".to_string(),
            hotkeys: Hotkeys {
                push_to_talk: "Ctrl+Shift+Space".to_string(),
                hands_free: "Ctrl+Shift+H".to_string(),
                emergency_stop: "Escape".to_string(),
            },
            auto_mute: true,
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

    fn get_settings_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
        Ok(app_dir.join("settings.json"))
    }
}
