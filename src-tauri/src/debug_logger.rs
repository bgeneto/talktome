use serde_json::Value;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

// Global state for debug logging
static DEBUG_ENABLED: Mutex<bool> = Mutex::new(false);
static LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub struct DebugLogger;

impl DebugLogger {
    /// Initialize debug logging to file - only if enabled in settings
    pub fn init(app_handle: &AppHandle) -> Result<(), String> {
        // Try to read debug setting from data directory or default to true for initial setup
        // This will be properly set by init_with_state when frontend syncs settings
        let debug_enabled = true; // Changed from false to true so initial logs are created

        println!(
            "DEBUG: init() called with debug_enabled = {}",
            debug_enabled
        );

        // Always set up the log path and create directory, regardless of enabled state
        let log_path = Self::get_log_path(app_handle)?;

        // Create log directory if it doesn't exist
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        // Always store log path globally so init_with_state can use it later
        if let Ok(mut path) = LOG_PATH.lock() {
            *path = Some(log_path.clone());
        }

        // Update global state AFTER setting up the path but BEFORE trying to write
        if let Ok(mut enabled) = DEBUG_ENABLED.lock() {
            *enabled = debug_enabled;
        }

        // Always try to create the log path and write a test file for debugging
        println!("DEBUG: Force-testing file creation regardless of debug_enabled state");

        // Only write initial messages if enabled
        if debug_enabled {
            println!("DEBUG: About to write initial log messages");

            // Direct file creation without going through write_log to avoid the global state check
            let startup_content = format!(
                "[{}] === TalkToMe Debug Session Started ===\n[{}] Log file: {}\n",
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC"),
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC"),
                log_path.display()
            );

            match std::fs::write(&log_path, &startup_content) {
                Ok(_) => println!(
                    "DEBUG: Successfully created log file with startup content: {}",
                    log_path.display()
                ),
                Err(e) => println!(
                    "DEBUG: Failed to create log file: {} - Error: {}",
                    log_path.display(),
                    e
                ),
            }

            println!("DEBUG: Initial log messages written to file");
        } else {
            println!("DEBUG: debug_enabled is FALSE - this is why no log file is created!");
        }

        Ok(())
    }

    /// Save a WAV (or any binary) dump alongside the log file for debugging and return the path
    pub fn save_wav_dump(label: &str, bytes: &[u8]) -> Option<std::path::PathBuf> {
        // Check if debug logging is enabled first
        if !Self::is_debug_enabled() {
            return None;
        }

        // Determine base logs directory from current log path
        let log_path = if let Ok(path) = LOG_PATH.lock() {
            if let Some(ref path) = *path {
                path.clone()
            } else {
                return None;
            }
        } else {
            return None;
        };

        let logs_dir = match log_path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => return None,
        };

        // Build filename with timestamp
        let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S%.3f");
        let filename = format!("{}_{}.wav", label, ts);
        let out_path = logs_dir.join(filename);

        // Best-effort create dir and write
        if let Err(e) = std::fs::create_dir_all(&logs_dir) {
            Self::write_log(&format!("SAVE_WAV_DUMP: Failed to ensure logs dir: {}", e));
            return None;
        }

        match std::fs::write(&out_path, bytes) {
            Ok(_) => {
                Self::write_log(&format!(
                    "SAVE_WAV_DUMP: Wrote {} bytes to {}",
                    bytes.len(),
                    out_path.display()
                ));
                Some(out_path)
            }
            Err(e) => {
                Self::write_log(&format!("SAVE_WAV_DUMP: Failed to write WAV: {}", e));
                None
            }
        }
    }

    /// Initialize debug logging with explicit state
    pub fn init_with_state(app_handle: &AppHandle, enabled: bool) -> Result<(), String> {
        // Update global state
        if let Ok(mut debug_enabled) = DEBUG_ENABLED.lock() {
            *debug_enabled = enabled;
        }

        if enabled {
            let log_path = Self::get_log_path(app_handle)?;

            // Create log directory if it doesn't exist
            if let Some(parent) = log_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }

            // Store log path globally
            if let Ok(mut path) = LOG_PATH.lock() {
                *path = Some(log_path.clone());
            }

            // Write initial log message only if not already initialized
            if !log_path.exists() || std::fs::metadata(&log_path).map(|m| m.len()).unwrap_or(0) == 0
            {
                Self::write_log(&format!("=== TalkToMe Debug Session Started ==="));
                Self::write_log(&format!("Log file: {}", log_path.display()));
            }
            Self::write_log(&format!("Debug logging state changed to: enabled"));
        } else {
            Self::write_log(&format!("Debug logging state changed to: disabled"));
        }

        Ok(())
    }

    /// Write a message directly to the log file
    fn write_log(message: &str) {
        // Check if logging is enabled
        let enabled = if let Ok(enabled) = DEBUG_ENABLED.lock() {
            *enabled
        } else {
            return;
        };

        if !enabled {
            return;
        }

        // Get log path
        let log_path = if let Ok(path) = LOG_PATH.lock() {
            if let Some(ref path) = *path {
                path.clone()
            } else {
                return;
            }
        } else {
            return;
        };

        // Format message with timestamp
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC");
        let formatted_message = format!("[{}] {}\n", timestamp, message);

        // Write to file
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            let _ = file.write_all(formatted_message.as_bytes());
            let _ = file.flush();
        }
    }

    /// Log audio chunk processing
    pub fn log_audio_chunk(
        data_len: usize,
        sample_rate: u32,
        has_activity: bool,
        max_amplitude: f32,
    ) {
        Self::write_log(&format!(
            "AUDIO_CHUNK: length={} samples, rate={}Hz, has_activity={}, max_amplitude={:.6}",
            data_len, sample_rate, has_activity, max_amplitude
        ));

        if !has_activity {
            Self::write_log("AUDIO_CHUNK: Skipping silent chunk (max_amplitude < 0.01)");
        }
    }

    /// Log transcription request details
    pub fn log_transcription_request(audio_size: usize, endpoint: &str) {
        Self::write_log(&format!("STT_REQUEST: Sending audio to Whisper API"));
        Self::write_log(&format!(
            "STT_REQUEST: audio_size={} bytes, endpoint={}",
            audio_size, endpoint
        ));
    }

    /// Log transcription response
    pub fn log_transcription_response(success: bool, text: Option<&str>, error: Option<&str>) {
        if success {
            if let Some(text) = text {
                Self::write_log(&format!("STT_RESPONSE: SUCCESS - '{}'", text));
                Self::write_log(&format!(
                    "STT_RESPONSE: transcript_length={} chars",
                    text.len()
                ));
            }
        } else {
            if let Some(error) = error {
                Self::write_log(&format!("STT_RESPONSE: ERROR - {}", error));
            }
        }
    }

    /// Log translation/processing request
    pub fn log_translation_request(
        original_text: &str,
        source_lang: &str,
        target_lang: &str,
        translation_enabled: bool,
        prompt: &str,
    ) {
        Self::write_log(&format!("TRANSLATION_REQUEST: Processing text"));
        Self::write_log(&format!(
            "TRANSLATION_REQUEST: original='{}', source_lang={}, target_lang={}, enabled={}",
            original_text, source_lang, target_lang, translation_enabled
        ));
        Self::write_log(&format!("TRANSLATION_REQUEST: Full prompt: '{}'", prompt));
    }

    /// Log translation API request payload
    pub fn log_api_payload(payload: &Value, endpoint: &str) {
        Self::write_log(&format!("API_REQUEST: Sending request to {}", endpoint));
        Self::write_log(&format!(
            "API_REQUEST: Full payload: {}",
            serde_json::to_string_pretty(payload).unwrap_or_default()
        ));

        // Log specific important fields
        if let Some(messages) = payload["messages"].as_array() {
            for (i, msg) in messages.iter().enumerate() {
                if let (Some(role), Some(content)) = (msg["role"].as_str(), msg["content"].as_str())
                {
                    Self::write_log(&format!(
                        "API_REQUEST: Message[{}] role={}, content_length={}",
                        i,
                        role,
                        content.len()
                    ));
                    Self::write_log(&format!(
                        "API_REQUEST: Message[{}] content: '{}'",
                        i, content
                    ));
                }
            }
        }

        if let Some(model) = payload["model"].as_str() {
            Self::write_log(&format!("API_REQUEST: Using model: {}", model));
        }
    }

    /// Log translation response
    pub fn log_translation_response(
        success: bool,
        processed_text: Option<&str>,
        error: Option<&str>,
        raw_response: Option<&str>,
    ) {
        if success {
            if let Some(text) = processed_text {
                Self::write_log(&format!("TRANSLATION_RESPONSE: SUCCESS - '{}'", text));
                Self::write_log(&format!(
                    "TRANSLATION_RESPONSE: processed_length={} chars",
                    text.len()
                ));
            }
        } else {
            Self::write_log(&format!(
                "TRANSLATION_RESPONSE: ERROR - {}",
                error.unwrap_or("Unknown error")
            ));
        }

        if let Some(raw) = raw_response {
            Self::write_log(&format!("TRANSLATION_RESPONSE: Raw API response: {}", raw));
        }
    }

    /// Log text insertion
    pub fn log_text_insertion(text: &str, success: bool, error: Option<&str>) {
        Self::write_log(&format!("TEXT_INSERTION: Inserting text: '{}'", text));

        if success {
            Self::write_log("TEXT_INSERTION: SUCCESS");
        } else {
            Self::write_log(&format!(
                "TEXT_INSERTION: ERROR - {}",
                error.unwrap_or("Unknown error")
            ));
        }
    }

    /// Log pipeline errors
    pub fn log_pipeline_error(stage: &str, error: &str) {
        Self::write_log(&format!(
            "PIPELINE_ERROR: Stage '{}' failed: {}",
            stage, error
        ));
    }

    /// Log general info
    pub fn log_info(message: &str) {
        Self::write_log(message);
    }

    /// Get log file path
    fn get_log_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let data_dir = Self::get_portable_data_dir(app_handle)?;
        Ok(data_dir.join("logs").join("talktome.log"))
    }

    /// Get portable data directory - same logic as settings
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

    /// Get current log file path for frontend
    pub fn get_log_file_path(app_handle: &AppHandle) -> Result<String, String> {
        let path = Self::get_log_path(app_handle)?;
        Ok(path.to_string_lossy().to_string())
    }

    /// Read recent log entries
    pub fn get_recent_logs(app_handle: &AppHandle, _lines: usize) -> Result<String, String> {
        let log_path = Self::get_log_path(app_handle)?;

        if !log_path.exists() {
            return Ok("Log file does not exist yet".to_string());
        }

        let content = std::fs::read_to_string(&log_path).map_err(|e| e.to_string())?;
        let lines: Vec<&str> = content.lines().collect();
        let recent_lines: Vec<&str> = lines.iter().rev().take(100).copied().collect();
        let recent_lines: Vec<&str> = recent_lines.iter().rev().copied().collect();

        Ok(recent_lines.join("\n"))
    }

    /// Clear log file
    pub fn clear_log(app_handle: &AppHandle) -> Result<(), String> {
        let log_path = Self::get_log_path(app_handle)?;
        std::fs::write(&log_path, "").map_err(|e| e.to_string())?;
        Self::write_log("=== Log file cleared ===");
        Ok(())
    }

    /// Check if debug logging is currently enabled
    pub fn is_debug_enabled() -> bool {
        if let Ok(enabled) = DEBUG_ENABLED.lock() {
            *enabled
        } else {
            false
        }
    }
}
