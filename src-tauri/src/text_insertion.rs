use std::process::Command;
use crate::debug_logger::DebugLogger;

pub struct TextInsertionService;

impl TextInsertionService {
    pub fn new() -> Self {
        Self
    }

    pub fn insert_text(&self, text: &str) -> Result<(), String> {
        DebugLogger::log_info("=== TEXT_INSERTION: insert_text() called ===");
        DebugLogger::log_info(&format!("TEXT_INSERTION: Input text='{}', length={} chars", text, text.len()));
        
        // Try to insert text into the focused application
        #[cfg(target_os = "windows")]
        {
            DebugLogger::log_info("TEXT_INSERTION: Using Windows implementation");
            self.insert_text_windows(text).map_err(|e| {
                let error_msg = format!("Windows text insertion failed: {}", e);
                DebugLogger::log_pipeline_error("text_insertion", &error_msg);
                error_msg
            })?;
        }
        
        #[cfg(target_os = "linux")]
        {
            DebugLogger::log_info("TEXT_INSERTION: Using Linux implementation");
            self.insert_text_linux(text).map_err(|e| {
                let error_msg = format!("Linux text insertion failed: {}", e);
                DebugLogger::log_pipeline_error("text_insertion", &error_msg);
                error_msg
            })?;
        }
        
        #[cfg(target_os = "macos")]
        {
            DebugLogger::log_info("TEXT_INSERTION: Using macOS implementation");
            self.insert_text_macos(text).map_err(|e| {
                let error_msg = format!("macOS text insertion failed: {}", e);
                DebugLogger::log_pipeline_error("text_insertion", &error_msg);
                error_msg
            })?;
        }
        
        DebugLogger::log_info("TEXT_INSERTION: insert_text() completed successfully");
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn insert_text_windows(&self, text: &str) -> Result<(), String> {
        DebugLogger::log_info("TEXT_INSERTION: Windows - Preparing PowerShell script");
        // Use PowerShell to simulate typing
        let escaped_text = text.replace("\"", "\\\"").replace("{", "{{").replace("}", "}}");
        DebugLogger::log_info(&format!("TEXT_INSERTION: Windows - Escaped text: '{}'", escaped_text));
        
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait(\"{}\")",
            escaped_text
        );
        DebugLogger::log_info(&format!("TEXT_INSERTION: Windows - PowerShell script: '{}'", script));
        
        DebugLogger::log_info("TEXT_INSERTION: Windows - Executing PowerShell command");
        let output = Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .map_err(|e| {
                let error_msg = format!("PowerShell execution failed: {}", e);
                DebugLogger::log_pipeline_error("text_insertion_windows", &error_msg);
                error_msg
            })?;
            
        DebugLogger::log_info(&format!("TEXT_INSERTION: Windows - PowerShell output: stdout='{}', stderr='{}'", 
            String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)));
        DebugLogger::log_info(&format!("TEXT_INSERTION: Windows - PowerShell exit status: {}", output.status));
        
        if !output.status.success() {
            let error_msg = format!("PowerShell script failed with status: {}", output.status);
            DebugLogger::log_pipeline_error("text_insertion_windows", &error_msg);
            return Err(error_msg);
        }
        
        DebugLogger::log_info("TEXT_INSERTION: Windows - PowerShell execution completed successfully");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn insert_text_linux(&self, text: &str) -> Result<(), String> {
        // Try xdotool first (X11)
        let output = Command::new("xdotool")
            .arg("type")
            .arg("--delay")
            .arg("12")
            .arg(text)
            .output();
            
        if output.is_ok() {
            return Ok(());
        }
        
        // Try wtype (Wayland)
        let output = Command::new("wtype")
            .arg(text)
            .output();
            
        if output.is_ok() {
            return Ok(());
        }
        
        // Try ydotool (alternative for Wayland)
        let output = Command::new("ydotool")
            .arg("type")
            .arg(text)
            .output();
            
        if output.is_ok() {
            return Ok(());
        }
        
        Err("No text insertion tool available".to_string())
    }

    #[cfg(target_os = "macos")]
    fn insert_text_macos(&self, text: &str) -> Result<(), String> {
        // Use AppleScript to simulate typing
        let script = format!(
            "tell application \"System Events\" to keystroke \"{}\"",
            text.replace("\"", "\\\"")
        );
        
        Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .map_err(|e| e.to_string())?;
            
        Ok(())
    }
}