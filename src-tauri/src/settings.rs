use serde::{Deserialize, Serialize};
use serde_json::json;
use keyring::Entry;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreBuilder;
use serde_json::Value;

/// Helper function to convert JSON value to u64
fn as_u64(v: &Value) -> Option<u64> {
    v.as_u64()
}

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
    // It's handled separately via secure storage (backend only)
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
    /// Load settings from persistent Tauri store
    pub fn load(app_handle: &AppHandle) -> Result<Self, String> {
        let store = StoreBuilder::new(app_handle, ".settings.dat").build()
            .map_err(|e| format!("Failed to build store: {}", e))?;

        let settings = Self::default();

        // Load each field from store with fallback to default
        let mut loaded_settings = Self {
            spoken_language: store.get("spoken_language")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| settings.spoken_language),
            translation_language: store.get("translation_language")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| settings.translation_language),
            audio_device: store.get("audio_device")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| settings.audio_device),
            theme: store.get("theme")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| settings.theme),
            auto_save: store.get("auto_save")
                .and_then(|v| v.as_bool())
                .unwrap_or(settings.auto_save),
            api_endpoint: store.get("api_endpoint")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| settings.api_endpoint),
            stt_model: store.get("stt_model")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| settings.stt_model),
            translation_model: store.get("translation_model")
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| settings.translation_model),
            hotkeys: Hotkeys {
                hands_free: store.get("hotkeys_hands_free")
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| settings.hotkeys.hands_free),
            },
            auto_mute: store.get("auto_mute")
                .and_then(|v| v.as_bool())
                .unwrap_or(settings.auto_mute),
            translation_enabled: store.get("translation_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(settings.translation_enabled),
            debug_logging: store.get("debug_logging")
                .and_then(|v| v.as_bool())
                .unwrap_or(settings.debug_logging),
            text_insertion_enabled: store.get("text_insertion_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(settings.text_insertion_enabled),
            audio_chunking_enabled: store.get("audio_chunking_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(settings.audio_chunking_enabled),
            max_recording_time_minutes: store.get("max_recording_time_minutes")
                .and_then(|v| as_u64(&v))
                .unwrap_or(settings.max_recording_time_minutes as u64) as u32,
        };

        // Always force audio_chunking_enabled to false for reliability
        loaded_settings.audio_chunking_enabled = false;

        Ok(loaded_settings)
    }

    /// Save settings to persistent Tauri store
    pub fn save(&self, app_handle: &AppHandle) -> Result<(), String> {
        let store = StoreBuilder::new(app_handle, ".settings.dat").build()
            .map_err(|e| format!("Failed to build store: {}", e))?;

        // Save each field to store
        store.set("spoken_language", serde_json::json!(self.spoken_language.clone()));
        store.set("translation_language", serde_json::json!(self.translation_language.clone()));
        store.set("audio_device", serde_json::json!(self.audio_device.clone()));
        store.set("theme", serde_json::json!(self.theme.clone()));
        store.set("auto_save", serde_json::json!(self.auto_save));
        store.set("api_endpoint", serde_json::json!(self.api_endpoint.clone()));
        store.set("stt_model", serde_json::json!(self.stt_model.clone()));
        store.set("translation_model", serde_json::json!(self.translation_model.clone()));
        store.set("hotkeys_hands_free", serde_json::json!(self.hotkeys.hands_free.clone()));
        store.set("auto_mute", serde_json::json!(self.auto_mute));
        store.set("translation_enabled", serde_json::json!(self.translation_enabled));
        store.set("debug_logging", serde_json::json!(self.debug_logging));
        store.set("text_insertion_enabled", serde_json::json!(self.text_insertion_enabled));
        // Always save audio_chunking_enabled as false for reliability
        store.set("audio_chunking_enabled", serde_json::json!(false));
        store.set("max_recording_time_minutes", serde_json::json!(self.max_recording_time_minutes));

        // Save the store to disk
        store.save()
            .map_err(|e| format!("Failed to save store: {}", e))?;

        Ok(())
    }

    /// Get API key from secure storage
    pub fn get_api_key(&self, _app_handle: &AppHandle) -> Result<String, String> {
        // Try OS keyring first
        let service = "talktome_api_key";
        let username = whoami::username();
        let entry = Entry::new(service, &username);

        match entry.get_password() {
            Ok(pw) => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();

        // Test that default settings are as expected
        assert_eq!(settings.spoken_language, "auto");
        assert_eq!(settings.translation_language, "none");
        assert_eq!(settings.audio_device, "default");
        assert_eq!(settings.api_endpoint, "https://api.openai.com/v1");
        assert_eq!(settings.stt_model, "whisper-large-v3");
        assert_eq!(settings.translation_model, "gpt-3.5-turbo");
        assert_eq!(settings.hotkeys.hands_free, "Ctrl+Shift+Space");
        assert_eq!(settings.auto_mute, true);
        assert_eq!(settings.translation_enabled, false);
        assert_eq!(settings.debug_logging, false);
        assert_eq!(settings.text_insertion_enabled, true);
        assert_eq!(settings.audio_chunking_enabled, false); // Should always be false
        assert_eq!(settings.max_recording_time_minutes, 5);
    }

    #[test]
    fn test_api_key_field_not_serialized() {
        let settings = AppSettings::default();

        // Test that API key is not included in the main struct serialization
        let serialized = serde_json::to_string(&settings).unwrap();

        // The API key should not be a field in the main settings struct
        // (It's handled separately via secure storage)
        assert!(!serialized.contains("api_key"));
    }

    #[test]
    fn test_hotkey_serialization() {
        let settings = AppSettings::default();
        let hotkeys = settings.hotkeys;

        assert_eq!(hotkeys.hands_free, "Ctrl+Shift+Space");

        // Test serialization of hotkeys
        let serialized = serde_json::to_string(&hotkeys).unwrap();
        assert!(serialized.contains("hands_free"));
        assert!(serialized.contains("Ctrl+Shift+Space"));
    }

    #[test]
    fn test_settings_json_roundtrip() {
        let original = AppSettings::default();

        // Serialize to JSON
        let json = serde_json::to_string(&original).unwrap();

        // Deserialize back from JSON
        let deserialized: AppSettings = serde_json::from_str(&json).unwrap();

        // Test that all fields match
        assert_eq!(original.spoken_language, deserialized.spoken_language);
        assert_eq!(original.translation_language, deserialized.translation_language);
        assert_eq!(original.audio_device, deserialized.audio_device);
        assert_eq!(original.hotkeys.hands_free, deserialized.hotkeys.hands_free);
        assert_eq!(original.audio_chunking_enabled, deserialized.audio_chunking_enabled);
    }
}
