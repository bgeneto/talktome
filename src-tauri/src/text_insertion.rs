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
        DebugLogger::log_info("TEXT_INSERTION: Windows - Using clipboard-paste method");
        
        // Step 1: Set clipboard content
        DebugLogger::log_info("TEXT_INSERTION: Windows - Setting clipboard content");
        let escaped_text = text.replace("\"", "\\\"").replace("`", "``");
        let clipboard_script = format!(
            "Set-Clipboard -Value \"{}\"",
            escaped_text
        );
        DebugLogger::log_info(&format!("TEXT_INSERTION: Windows - Clipboard script: '{}'", clipboard_script));
        
        let clipboard_output = Command::new("powershell")
            .arg("-Command")
            .arg(&clipboard_script)
            .output()
            .map_err(|e| {
                let error_msg = format!("Clipboard set failed: {}", e);
                DebugLogger::log_pipeline_error("text_insertion_clipboard", &error_msg);
                error_msg
            })?;
            
        if !clipboard_output.status.success() {
            let error_msg = format!("Clipboard set failed with status: {}", clipboard_output.status);
            DebugLogger::log_pipeline_error("text_insertion_clipboard", &error_msg);
            return Err(error_msg);
        }
        
        DebugLogger::log_info("TEXT_INSERTION: Windows - Clipboard content set successfully");
        
        // Step 2: Send Ctrl+V keystroke
        DebugLogger::log_info("TEXT_INSERTION: Windows - Sending Ctrl+V keystroke");
        let paste_script = "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait(\"^v\")";
        
        let paste_output = Command::new("powershell")
            .arg("-Command")
            .arg(paste_script)
            .output()
            .map_err(|e| {
                let error_msg = format!("Paste keystroke failed: {}", e);
                DebugLogger::log_pipeline_error("text_insertion_paste", &error_msg);
                error_msg
            })?;
            
        DebugLogger::log_info(&format!("TEXT_INSERTION: Windows - Paste output: stdout='{}', stderr='{}'", 
            String::from_utf8_lossy(&paste_output.stdout), String::from_utf8_lossy(&paste_output.stderr)));
        DebugLogger::log_info(&format!("TEXT_INSERTION: Windows - Paste exit status: {}", paste_output.status));
        
        if !paste_output.status.success() {
            let error_msg = format!("Paste keystroke failed with status: {}", paste_output.status);
            DebugLogger::log_pipeline_error("text_insertion_paste", &error_msg);
            return Err(error_msg);
        }
        
        DebugLogger::log_info("TEXT_INSERTION: Windows - Clipboard-paste insertion completed successfully");
        
        // Small delay to ensure paste operation completes
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn insert_text_linux(&self, text: &str) -> Result<(), String> {
        DebugLogger::log_info("TEXT_INSERTION: Linux - Using clipboard-paste method");
        
        // Step 1: Set clipboard content
        DebugLogger::log_info("TEXT_INSERTION: Linux - Setting clipboard content");
        
        // Try xclip first (X11)
        let xclip_result = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(text.as_bytes())?;
                }
                child.wait()
            });
            
        if xclip_result.is_ok() {
            DebugLogger::log_info("TEXT_INSERTION: Linux - Clipboard set with xclip");
            
            // Send Ctrl+V using xdotool
            let paste_result = Command::new("xdotool")
                .arg("key")
                .arg("ctrl+v")
                .output();
                
            if paste_result.is_ok() {
                DebugLogger::log_info("TEXT_INSERTION: Linux - Paste sent with xdotool");
                // Small delay to ensure paste operation completes
                std::thread::sleep(std::time::Duration::from_millis(50));
                return Ok(());
            }
        }
        
        // Try wl-copy (Wayland)
        let wl_copy_result = Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(text.as_bytes())?;
                }
                child.wait()
            });
            
        if wl_copy_result.is_ok() {
            DebugLogger::log_info("TEXT_INSERTION: Linux - Clipboard set with wl-copy");
            
            // Try wtype for paste (Wayland)
            let wtype_result = Command::new("wtype")
                .arg("-M")
                .arg("ctrl")
                .arg("-k")
                .arg("v")
                .output();
                
            if wtype_result.is_ok() {
                DebugLogger::log_info("TEXT_INSERTION: Linux - Paste sent with wtype");
                // Small delay to ensure paste operation completes
                std::thread::sleep(std::time::Duration::from_millis(50));
                return Ok(());
            }
            
            // Try ydotool for paste (alternative Wayland)
            let ydotool_result = Command::new("ydotool")
                .arg("key")
                .arg("29:1")  // Ctrl down
                .arg("47:1")  // V down
                .arg("47:0")  // V up
                .arg("29:0")  // Ctrl up
                .output();
                
            if ydotool_result.is_ok() {
                DebugLogger::log_info("TEXT_INSERTION: Linux - Paste sent with ydotool");
                // Small delay to ensure paste operation completes
                std::thread::sleep(std::time::Duration::from_millis(50));
                return Ok(());
            }
        }
        
        DebugLogger::log_pipeline_error("text_insertion_linux", "No clipboard/paste tools available");
        Err("No clipboard or paste tools available (tried xclip+xdotool, wl-copy+wtype, wl-copy+ydotool)".to_string())
    }

    #[cfg(target_os = "macos")]
    fn insert_text_macos(&self, text: &str) -> Result<(), String> {
        DebugLogger::log_info("TEXT_INSERTION: macOS - Using clipboard-paste method");
        
        // Step 1: Set clipboard content using pbcopy
        DebugLogger::log_info("TEXT_INSERTION: macOS - Setting clipboard content");
        let clipboard_result = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.as_mut() {
                    stdin.write_all(text.as_bytes())?;
                }
                child.wait()
            });
            
        clipboard_result.map_err(|e| {
            let error_msg = format!("Clipboard set failed: {}", e);
            DebugLogger::log_pipeline_error("text_insertion_clipboard_macos", &error_msg);
            error_msg
        })?;
        
        DebugLogger::log_info("TEXT_INSERTION: macOS - Clipboard content set successfully");
        
        // Step 2: Send Cmd+V keystroke using AppleScript
        DebugLogger::log_info("TEXT_INSERTION: macOS - Sending Cmd+V keystroke");
        let paste_script = "tell application \"System Events\" to key code 9 using {command down}";
        
        let paste_result = Command::new("osascript")
            .arg("-e")
            .arg(paste_script)
            .output()
            .map_err(|e| {
                let error_msg = format!("Paste keystroke failed: {}", e);
                DebugLogger::log_pipeline_error("text_insertion_paste_macos", &error_msg);
                error_msg
            })?;
            
        if !paste_result.status.success() {
            let error_msg = format!("Paste keystroke failed with status: {}", paste_result.status);
            DebugLogger::log_pipeline_error("text_insertion_paste_macos", &error_msg);
            return Err(error_msg);
        }
        
        DebugLogger::log_info("TEXT_INSERTION: macOS - Clipboard-paste insertion completed successfully");
        
        // Small delay to ensure paste operation completes
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        Ok(())
    }
}