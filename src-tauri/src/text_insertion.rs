use std::process::Command;

pub struct TextInsertionService;

impl TextInsertionService {
    pub fn new() -> Self {
        Self
    }

    pub fn insert_text(&self, text: &str) -> Result<(), String> {
        // Try to insert text into the focused application
        #[cfg(target_os = "windows")]
        {
            self.insert_text_windows(text).map_err(|e| e.to_string())?;
        }
        
        #[cfg(target_os = "linux")]
        {
            self.insert_text_linux(text).map_err(|e| e.to_string())?;
        }
        
        #[cfg(target_os = "macos")]
        {
            self.insert_text_macos(text).map_err(|e| e.to_string())?;
        }
        
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn insert_text_windows(&self, text: &str) -> Result<(), String> {
        // Use PowerShell to simulate typing
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; [System.Windows.Forms.SendKeys]::SendWait(\"{}\")",
            text.replace("\"", "\\\"").replace("{", "{{").replace("}", "}}")
        );
        
        Command::new("powershell")
            .arg("-Command")
            .arg(&script)
            .output()
            .map_err(|e| e.to_string())?;
            
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