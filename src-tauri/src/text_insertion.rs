use arboard::Clipboard;
use enigo::{Enigo, Key, Keyboard, Settings};
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
        DebugLogger::log_info("TEXT_INSERTION: Windows - Using native Rust clipboard + enigo keyboard simulation");
        
        DebugLogger::log_info(&format!("TEXT_INSERTION: Windows - Setting clipboard content for text: '{}'", text));
        
        // Try native Rust approach first (much faster and more reliable)
        match self.insert_text_native(text) {
            Ok(()) => {
                DebugLogger::log_info("TEXT_INSERTION: Windows - Native Rust method succeeded");
                return Ok(());
            }
            Err(e) => {
                DebugLogger::log_info(&format!("TEXT_INSERTION: Windows - Native method failed: {}, trying PowerShell fallback", e));
                // Continue to PowerShell fallback below
            }
        }
        
        // Fallback to PowerShell if native approach fails
        DebugLogger::log_info("TEXT_INSERTION: Windows - Using PowerShell fallback method");
        self.insert_text_windows_powershell_fallback(text)
    }

    // Native Rust implementation (primary method)
    fn insert_text_native(&self, text: &str) -> Result<(), String> {
        // Step 1: Set clipboard content using arboard (much faster than PowerShell)
        let mut clipboard = Clipboard::new().map_err(|e| {
            format!("Failed to initialize clipboard: {}", e)
        })?;
        
        clipboard.set_text(text).map_err(|e| {
            format!("Failed to set clipboard content: {}", e)
        })?;
        
        DebugLogger::log_info("TEXT_INSERTION: Native - Clipboard content set successfully with arboard");
        
        // Step 2: Send Ctrl+V keystroke using enigo (much more reliable than SendKeys)
        DebugLogger::log_info("TEXT_INSERTION: Native - Sending keystroke with enigo");
        
        // Small delay to ensure clipboard is ready
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        let mut enigo = Enigo::new(&Settings::default()).map_err(|e| {
            format!("Failed to initialize enigo keyboard: {}", e)
        })?;
        
        // Send Ctrl+V key combination (Cmd+V on macOS)
        #[cfg(target_os = "macos")]
        let modifier_key = Key::Meta;
        #[cfg(not(target_os = "macos"))]
        let modifier_key = Key::Control;
        
        enigo.key(modifier_key, enigo::Direction::Press).map_err(|e| {
            format!("Failed to press modifier key: {}", e)
        })?;
        
        enigo.key(Key::Unicode('v'), enigo::Direction::Click).map_err(|e| {
            format!("Failed to click V key: {}", e)
        })?;
        
        enigo.key(modifier_key, enigo::Direction::Release).map_err(|e| {
            format!("Failed to release modifier key: {}", e)
        })?;
        
        DebugLogger::log_info("TEXT_INSERTION: Native - Keystroke sent successfully with enigo");
        
        // Small delay to ensure paste operation completes
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        DebugLogger::log_info("TEXT_INSERTION: Native - Text insertion completed successfully");
        Ok(())
    }

    // PowerShell fallback method (only used if native method fails)
    #[cfg(target_os = "windows")]
    fn insert_text_windows_powershell_fallback(&self, text: &str) -> Result<(), String> {
        use std::process::Command;
        
        // Escape text for PowerShell
        let escaped_text = text
            .replace("`", "``")
            .replace("\"", "`\"")
            .replace("'", "''")
            .replace("\r\n", "`r`n")
            .replace("\n", "`n")
            .replace("\r", "`r");
        
        let script = format!(r#"
            try {{
                Set-Clipboard -Value "{}"
                Start-Sleep -Milliseconds 100
                Add-Type -AssemblyName System.Windows.Forms
                [System.Windows.Forms.SendKeys]::SendWait("^v")
                Start-Sleep -Milliseconds 50
                exit 0
            }} catch {{
                Write-Error "PowerShell fallback failed: $_"
                exit 1
            }}
        "#, escaped_text);
        
        let output = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-Command")
            .arg(&script)
            .output()
            .map_err(|e| format!("PowerShell execution failed: {}", e))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("PowerShell fallback failed: {}", stderr));
        }
        
        DebugLogger::log_info("TEXT_INSERTION: Windows - PowerShell fallback completed successfully");
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn insert_text_linux(&self, text: &str) -> Result<(), String> {
        DebugLogger::log_info("TEXT_INSERTION: Linux - Using native Rust clipboard + enigo keyboard simulation");
        DebugLogger::log_info(&format!("TEXT_INSERTION: Linux - Setting clipboard content for text: '{}'", text));
        
        // Use the shared native implementation
        self.insert_text_native(text)
    }

    #[cfg(target_os = "macos")]
    fn insert_text_macos(&self, text: &str) -> Result<(), String> {
        DebugLogger::log_info("TEXT_INSERTION: macOS - Using native Rust clipboard + enigo keyboard simulation");
        DebugLogger::log_info(&format!("TEXT_INSERTION: macOS - Setting clipboard content for text: '{}'", text));
        
        // Use the shared native implementation
        self.insert_text_native(text)
    }

    // Test function for debugging text insertion
    pub fn test_insert(&self, test_text: &str) -> Result<(), String> {
        DebugLogger::log_info(&format!("=== TEXT_INSERTION_TEST: Testing with text='{}' ===", test_text));
        self.insert_text(test_text)
    }
}