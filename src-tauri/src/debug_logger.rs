use log::{debug, error, info};
use serde_json::Value;
use std::io::Write;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub struct DebugLogger;

impl DebugLogger {
    /// Initialize debug logging to file - only if enabled in settings
    pub fn init(app_handle: &AppHandle) -> Result<(), String> {
        use crate::settings::get_setting_bool;
        
        // Check if debug logging is enabled
        let debug_enabled = get_setting_bool(app_handle, "debug_logging").unwrap_or(false);
        if !debug_enabled {
            return Ok(()); // Debug logging disabled, do nothing
        }
        
        let log_path = Self::get_log_path(app_handle)?;
        
        // Create log directory if it doesn't exist
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }

        // Initialize env_logger with custom format
        let target = Box::new(std::fs::File::create(&log_path).map_err(|e| e.to_string())?);
        
        env_logger::Builder::from_default_env()
            .target(env_logger::Target::Pipe(target))
            .format(|buf, record| {
                writeln!(
                    buf,
                    "[{}] [{}] [{}:{}] {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f UTC"),
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    record.args()
                )
            })
            .init();

        info!("=== TalkToMe Debug Session Started ===");
        info!("Log file: {}", log_path.display());
        
        Ok(())
    }

    /// Log audio chunk processing
    pub fn log_audio_chunk(data_len: usize, sample_rate: u32, has_activity: bool, max_amplitude: f32) {
        // Only log if debug logging is enabled (we can't easily check settings here, so we'll rely on logger being initialized)
        debug!("AUDIO_CHUNK: length={} samples, rate={}Hz, has_activity={}, max_amplitude={:.6}", 
               data_len, sample_rate, has_activity, max_amplitude);
        
        if !has_activity {
            debug!("AUDIO_CHUNK: Skipping silent chunk (max_amplitude < 0.01)");
        }
    }

    /// Log transcription request details
    pub fn log_transcription_request(audio_size: usize, endpoint: &str) {
        info!("STT_REQUEST: Sending audio to Whisper API");
        debug!("STT_REQUEST: audio_size={} bytes, endpoint={}", audio_size, endpoint);
    }

    /// Log transcription response
    pub fn log_transcription_response(success: bool, text: Option<&str>, error: Option<&str>) {
        if success {
            if let Some(text) = text {
                info!("STT_RESPONSE: SUCCESS - '{}'", text);
                debug!("STT_RESPONSE: transcript_length={} chars", text.len());
            }
        } else {
            if let Some(error) = error {
                error!("STT_RESPONSE: ERROR - {}", error);
            }
        }
    }

    /// Log translation/processing request
    pub fn log_translation_request(
        original_text: &str, 
        source_lang: &str, 
        target_lang: &str, 
        translation_enabled: bool,
        prompt: &str
    ) {
        info!("TRANSLATION_REQUEST: Processing text");
        debug!("TRANSLATION_REQUEST: original='{}', source_lang={}, target_lang={}, enabled={}", 
               original_text, source_lang, target_lang, translation_enabled);
        debug!("TRANSLATION_REQUEST: Full prompt: '{}'", prompt);
    }

    /// Log translation API request payload
    pub fn log_api_payload(payload: &Value, endpoint: &str) {
        info!("API_REQUEST: Sending request to {}", endpoint);
        debug!("API_REQUEST: Full payload: {}", serde_json::to_string_pretty(payload).unwrap_or_default());
        
        // Log specific important fields
        if let Some(messages) = payload["messages"].as_array() {
            for (i, msg) in messages.iter().enumerate() {
                if let (Some(role), Some(content)) = (msg["role"].as_str(), msg["content"].as_str()) {
                    debug!("API_REQUEST: Message[{}] role={}, content_length={}", i, role, content.len());
                    debug!("API_REQUEST: Message[{}] content: '{}'", i, content);
                }
            }
        }
        
        if let Some(model) = payload["model"].as_str() {
            debug!("API_REQUEST: Using model: {}", model);
        }
    }

    /// Log translation response
    pub fn log_translation_response(success: bool, processed_text: Option<&str>, error: Option<&str>, raw_response: Option<&str>) {
        if success {
            if let Some(text) = processed_text {
                info!("TRANSLATION_RESPONSE: SUCCESS - '{}'", text);
                debug!("TRANSLATION_RESPONSE: processed_length={} chars", text.len());
            }
        } else {
            error!("TRANSLATION_RESPONSE: ERROR - {}", error.unwrap_or("Unknown error"));
        }
        
        if let Some(raw) = raw_response {
            debug!("TRANSLATION_RESPONSE: Raw API response: {}", raw);
        }
    }

    /// Log text insertion
    pub fn log_text_insertion(text: &str, success: bool, error: Option<&str>) {
        info!("TEXT_INSERTION: Inserting text: '{}'", text);
        
        if success {
            info!("TEXT_INSERTION: SUCCESS");
        } else {
            error!("TEXT_INSERTION: ERROR - {}", error.unwrap_or("Unknown error"));
        }
    }

    /// Log pipeline errors
    pub fn log_pipeline_error(stage: &str, error: &str) {
        error!("PIPELINE_ERROR: Stage '{}' failed: {}", stage, error);
    }

    /// Log general info
    pub fn log_info(message: &str) {
        info!("{}", message);
    }

    /// Get log file path
    fn get_log_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
        let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
        Ok(app_dir.join("talktome.log"))
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
        info!("=== Log file cleared ===");
        Ok(())
    }
}
