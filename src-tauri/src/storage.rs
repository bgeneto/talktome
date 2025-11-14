use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PersistentSettings {
    pub spoken_language: String,
    pub translation_language: String,
    pub audio_device: String,
    pub theme: String,
    pub api_endpoint: String,
    pub stt_model: String,
    pub translation_model: String,
    pub hands_free_hotkey: String,
    pub auto_mute: bool,
    pub translation_enabled: bool,
    pub debug_logging: bool,
    pub text_insertion_enabled: bool,
    pub max_recording_time_minutes: u32,
}

impl Default for PersistentSettings {
    fn default() -> Self {
        Self {
            spoken_language: "auto".to_string(),
            translation_language: "none".to_string(),
            audio_device: "default".to_string(),
            theme: "auto".to_string(),
            api_endpoint: "https://api.openai.com/v1".to_string(),
            stt_model: "whisper-large-v3".to_string(),
            translation_model: "gpt-3.5-turbo".to_string(),
            hands_free_hotkey: "Ctrl+Shift+Space".to_string(),
            auto_mute: true,
            translation_enabled: false,
            debug_logging: false,
            text_insertion_enabled: true,
            max_recording_time_minutes: 2,
        }
    }
}

pub struct SettingsStore;

impl SettingsStore {
    const STORE_NAME: &'static str = "talktome-settings";
    const SETTINGS_KEY: &'static str = "app-settings";

    pub fn load(app: &AppHandle) -> Result<PersistentSettings, String> {
        let store = app
            .store(Self::STORE_NAME)
            .map_err(|e| format!("Failed to open store: {}", e))?;

        match store.get(Self::SETTINGS_KEY) {
            Some(value) => {
                let settings = serde_json::from_value::<PersistentSettings>(value)
                    .map_err(|e| format!("Failed to deserialize settings: {}", e))?;
                crate::debug_logger::DebugLogger::log_info(&format!("Loaded persistent settings from store: spoken_language={}, translation_language={}", settings.spoken_language, settings.translation_language));
                Ok(settings)
            }
            None => {
                crate::debug_logger::DebugLogger::log_info("No persistent settings found in store, using defaults");
                Ok(PersistentSettings::default())
            }
        }
    }

    pub fn save(app: &AppHandle, settings: &PersistentSettings) -> Result<(), String> {
        let store = app
            .store(Self::STORE_NAME)
            .map_err(|e| format!("Failed to open store: {}", e))?;

        let value = serde_json::to_value(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        store.set(Self::SETTINGS_KEY.to_string(), value);

        store
            .save()
            .map_err(|e| format!("Failed to sync store: {}", e))?;

        crate::debug_logger::DebugLogger::log_info(&format!("Saved persistent settings to store: spoken_language={}, translation_language={}", settings.spoken_language, settings.translation_language));
        Ok(())
    }

    pub fn update_field(
        app: &AppHandle,
        field: &str,
        value: serde_json::Value,
    ) -> Result<(), String> {
        let mut settings = Self::load(app)?;

        match field {
            "spoken_language" => {
                if let Some(s) = value.as_str() {
                    settings.spoken_language = s.to_string();
                }
            }
            "translation_language" => {
                if let Some(s) = value.as_str() {
                    settings.translation_language = s.to_string();
                }
            }
            "audio_device" => {
                if let Some(s) = value.as_str() {
                    settings.audio_device = s.to_string();
                }
            }
            "theme" => {
                if let Some(s) = value.as_str() {
                    settings.theme = s.to_string();
                }
            }
            "api_endpoint" => {
                if let Some(s) = value.as_str() {
                    settings.api_endpoint = s.to_string();
                }
            }
            "stt_model" => {
                if let Some(s) = value.as_str() {
                    settings.stt_model = s.to_string();
                }
            }
            "translation_model" => {
                if let Some(s) = value.as_str() {
                    settings.translation_model = s.to_string();
                }
            }
            "hands_free_hotkey" => {
                if let Some(s) = value.as_str() {
                    settings.hands_free_hotkey = s.to_string();
                }
            }
            "auto_mute" => {
                if let Some(b) = value.as_bool() {
                    settings.auto_mute = b;
                }
            }
            "translation_enabled" => {
                if let Some(b) = value.as_bool() {
                    settings.translation_enabled = b;
                }
            }
            "debug_logging" => {
                if let Some(b) = value.as_bool() {
                    settings.debug_logging = b;
                }
            }
            "text_insertion_enabled" => {
                if let Some(b) = value.as_bool() {
                    settings.text_insertion_enabled = b;
                }
            }
            "max_recording_time_minutes" => {
                if let Some(n) = value.as_u64() {
                    settings.max_recording_time_minutes = n as u32;
                }
            }
            _ => return Err(format!("Unknown field: {}", field)),
        }

        Self::save(app, &settings)?;
        Ok(())
    }
}
