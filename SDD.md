## **Software Design Document (SDD)**

Project: TalkToMe – a modern cross-platform voice-to-text desktop application with live translation and smart audio features.

Target Stack: Tauri (Rust core + WebView) + Svelte (UI) + Tailwind CSS (styling).

Vibe Compatibility: This SDD is written to be Vibe-compatible. You can copy-paste specifications into a Vibe-style "prompt-to-code" workflow to generate the project skeleton, tests, and CI/CD pipeline.

------

### **1. Introduction**

#### **1.1 Purpose**

To create a lightweight, privacy-first, cross-platform desktop application that allows users to dictate, edit, format, and optionally translate text using their voice. The application must run on Windows 10+ (64-bit) and modern Linux distributions (e.g., Ubuntu 20.04+, Fedora 35+, Arch), with intelligent audio management and seamless text insertion capabilities.

#### **1.2 Scope**

| In-Scope                                                     | Out-of-Scope                                      |
| ------------------------------------------------------------ | ------------------------------------------------- |
| • Real-time speech-to-text via OpenAI-compatible Whisper API | • MacOS and Mobile (iOS/Android) support (future) |
| • **Live Translation**: Transcribe speech from a source language to a different target language via OpenAI-compatible chat completion endpoints. | • Multi-user collaboration (future)               |
| • **Advanced Hotkeys**: Configurable global shortcuts for "Push to talk" and "Hands-free" modes. | • Local/offline transcription                     |
| • **Multi-language Dictation**: User can select the language(s) for transcription. | • Custom voice models                             |
| • **Smart Audio Management**: Auto-mute music/media during dictation (configurable) | • Video transcription                             |
| • **Universal Text Insertion**: Insert transcribed/translated text into any active text input | • Browser extension integration                   |
| • **Rich System Tray**: Right-click menu with microphone selection, language switching, and settings access |                                                   |
|                                                              |                                                   |
| • Light/Dark theming with runtime toggle                     |                                                   |
| • Settings persistence (JSON in app-data)                    |                                                   |
|                                                              |                                                   |

#### **1.3 Intended Audience**

| Role                         | Reason                                                       |
| ---------------------------- | ------------------------------------------------------------ |
| **Developers (Rust, JS/TS)** | Primary readers – implement the design.                      |
| **Product Owner / PM**       | Verify that functional & non-functional requirements are covered. |
| **QA / Test Engineers**      | Build test cases from the functional spec.                   |
| **Security Auditor**         | Review privacy, data-flow, API key handling, and permission model. |

#### **1.4 Definitions & Acronyms**

| Acronym      | Definition                                                   |
| ------------ | ------------------------------------------------------------ |
| **Tauri**    | Rust-based desktop runtime that wraps a native WebView.      |
| **Svelte**   | Reactive UI framework that compiles to vanilla JS.           |
| **Tailwind** | Utility-first CSS framework.                                 |
| **STT**      | Speech-to-Text (Whisper API endpoint).                       |
| **LLM**      | Large Language Model (chat/completions endpoint for translation). |
| **Vibe**     | Prompt-driven "code-as-you-type" environment that lets LLMs generate code from high-level spec. |
| ****         |                                                              |
| **AppData**  | Platform-specific folder (`%APPDATA%/talktome` on Windows, `$XDG_DATA_HOME/talktome` on Linux). |

------

### **2. Overall Description**

#### **2.1 System Context**

The application is a standalone desktop utility with intelligent system integration. The UI, built with Svelte, runs in a Tauri-managed WebView. All heavy lifting—audio capture, transcription via OpenAI-compatible APIs, hotkey monitoring, system audio control, text insertion, and system tray management—is handled by the Rust core, exposed to the frontend via Tauri commands.

```
graph TD
    User(👤 User) -- Voice/Hotkeys --> OS
    OS -- Mic Audio & Key Events --> Tauri
    Tauri(🚀 Tauri Rust Core)
    subgraph Tauri
        A[Audio Capture]
        B[Global Hotkey Listener]
        C[Audio Control Manager]
        D[STT Service - Whisper API]
        E[Translation Service - Chat API]
        F[Text Insertion Service]
        G[System Tray Manager]
        H[File System I/O]
    end
    Tauri -- RPC (invoke/emit) --> WebView
    WebView(🖼️ Svelte UI) -- Renders Text --> User
    A --> D
    B -- Triggers --> A
    C -- Mutes/Unmutes --> SystemAudio[🔊 System Audio]
    D -- Transcribed Text --> E
    E -- API Call --> OpenAI_API(🌐 OpenAI-Compatible APIs)
    OpenAI_API -- Translated Text --> E
    F -- Inserts Text --> ActiveApp[📝 Active Application]
    G -- Manages --> SystemTray[🔖 System Tray]
    D --> Tauri
    E --> Tauri
    Tauri --> H
```

#### **2.2 Product Functions (High-Level)**

| #    | Function                     | Description                                                  |
| ---- | ---------------------------- | ------------------------------------------------------------ |
| 1    | **Start/Stop Dictation**     | Capture microphone audio using two distinct modes: "Push to talk" (active while key is held) and "Hands-free" (toggled on/off). |
| 2    | **Live Translation**         | If enabled, transcribed text is sent to OpenAI-compatible chat completion endpoint (custom API base URL can be set in settings) and replaced with the translation in a user-selected target language. |
| 3    | **Universal Text Insertion** | Insert transcribed/translated text directly into any active text input field system-wide. |
| 4    | **Smart Audio Management**   | Automatically mute/pause music and media during dictation (configurable toggle). |
| 5    | **Rich System Tray**         | Right-click menu providing quick access to microphone selection, language switching, settings, and about information. |
| 6    |                              |                                                              |
| 7    | **Settings Management**      | Configure API endpoints, microphone, dictation/translation languages, global keyboard shortcuts, and audio management preferences. |
| 8    | **Theme Switch**             | Follow system UI theme or Toggle between Light and Dark UI themes. The choice is persisted across sessions. |
|      |                              |                                                              |
|      |                              |                                                              |

------

### **3. Architecture**

#### **3.1 Architectural Style**

- **Layered Architecture:** UI (Svelte) → Bridge (Tauri) → Core (Rust) → Services (Audio, STT, Translation, System Integration).
- **Event-Driven:** Asynchronous events are used for UI updates (e.g., receiving transcribed or translated text chunks) and handling global hotkeys.
- **Service-Oriented:** Clear separation of concerns with dedicated services for audio management, text insertion, and system tray operations.

#### **3.2 Major Components**

| Component                  | Language       | Description                                                  |
| -------------------------- | -------------- | ------------------------------------------------------------ |
| **UI Layer**               | Svelte (TS)    | Main window, editor, and settings panel. Renders data from Svelte stores. |
| **Bridge Layer**           | Rust (Tauri)   | Exposes `#[tauri::command]` functions and emits events to the UI. |
| **Global Hotkey Manager**  | Rust           | Uses `tauri-plugin-global-shortcut` to listen for system-wide key presses and trigger actions. |
| **Audio Capture**          | Rust (cpal)    | Opens the selected microphone, captures the audio stream, and forwards audio data. |
| **Audio Control Manager**  | Rust           | Platform-specific audio session management to mute/unmute system audio during dictation. |
| **STT Service**            | Rust (reqwest) | Sends audio to OpenAI-compatible Whisper endpoint for transcription. |
| **Translation Service**    | Rust (reqwest) | Sends transcribed text to OpenAI-compatible chat/completions endpoint for translation. |
| **Text Insertion Service** | Rust           | Platform-specific text insertion into active applications using clipboard and key simulation. |
| **System Tray Manager**    | Rust           | Manages system tray icon, context menu, and associated actions. |
| **Command Processor**      | Rust           | Parses voice commands and translates them into structured edit operations. |
| **Persistence**            | Rust (serde)   | Saves/loads `settings.json` and manages secure API key storage using `tauri-plugin-stronghold`. |
|                            |                |                                                              |

#### **3.3 Data Flow for Transcription & Translation**

1. **User presses hotkey** (e.g., holds `Ctrl+Win+Space` for Push-to-talk).
2. **Hotkey Manager (Rust)** catches the event and invokes the `start_dictation` command.
3. **Audio Control Manager (Rust)** optionally mutes system audio if enabled in settings.
4. **Audio Capture (Rust)** starts streaming microphone data and buffers it for API transmission.
5. **STT Service (Rust)** sends audio buffer to OpenAI-compatible Whisper endpoint and receives transcribed text.
6. **Conditional Logic:**
   - **If Translation is OFF:** The transcribed text is processed for text insertion.
   - **If Translation is ON:** The text is passed to the **Translation Service (Rust)**.
7. **Translation Service** sends the source text to the OpenAI-compatible chat/completions endpoint with translation instructions.
8. **Text Insertion Service** inserts the final text (transcribed or translated) into the active text input.
9. **Audio Control Manager** restores system audio to previous state.
10. **UI (Svelte)** receives status updates and displays transcription/translation progress.

------

### **4. Detailed Design**

#### **4.1 Core Rust Modules & Types**

The Rust `src-tauri/src` directory will be organized as follows: `main.rs`, `audio/mod.rs`, `audio/capture.rs`, `audio/control.rs`, `hotkeys.rs`, `services/mod.rs`, `services/stt.rs`, `services/translation.rs`, `system/mod.rs`, `system/tray.rs`, `system/text_insertion.rs`, `settings.rs`, `cmd_processor.rs`.

**settings.rs**

The central configuration struct, persisted as JSON.

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DictationMode {
    PushToTalk,
    HandsFree,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiConfig {
    pub whisper_endpoint: String, // e.g., "https://api.openai.com/v1/audio/transcriptions"
    pub chat_endpoint: String,    // e.g., "https://api.openai.com/v1/chat/completions"
    pub api_key_set: bool,        // Whether API key is stored in stronghold
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AudioConfig {
    pub auto_mute_enabled: bool,
    pub selected_mic_id: Option<String>,
    pub input_gain: f32,         // Microphone gain level
    pub noise_suppression: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranslationConfig {
    pub enabled: bool,
    pub target_language: String,     // e.g., "Portuguese"
    pub source_language: String,     // e.g., "English" 
    pub model: String,              // e.g., "gpt-4" for chat completions
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HotkeyConfig {
    pub push_to_talk: String,    // e.g., "Ctrl+Win"
    pub hands_free_toggle: String, // e.g., "Ctrl+Win+Space"
    pub emergency_stop: String,   // e.g., "Escape"
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrayConfig {
    pub show_notifications: bool,
    pub quick_languages: Vec<String>, // Languages shown in tray submenu
    pub available_microphones: HashMap<String, String>, // ID -> Name mapping
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub theme: String,              // "auto" , light" or "dark"
    pub dictation_language: String, // e.g., "en-US"
    pub api: ApiConfig,
    pub audio: AudioConfig,
    pub translation: TranslationConfig,
    pub hotkeys: HotkeyConfig,
    pub tray: TrayConfig,
    pub privacy_mode: bool,
    pub text_insertion_enabled: bool,
    pub auto_save_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            theme: "light".to_string(),
            dictation_language: "en-US".to_string(),
            api: ApiConfig {
                whisper_endpoint: "https://api.openai.com/v1/audio/transcriptions".to_string(),
                chat_endpoint: "https://api.openai.com/v1/chat/completions".to_string(),
                api_key_set: false,
            },
            audio: AudioConfig {
                auto_mute_enabled: true,
                selected_mic_id: None,
                input_gain: 1.0,
                noise_suppression: true,
            },
            translation: TranslationConfig {
                enabled: false,
                target_language: "English".to_string(),
                source_language: "Auto-detect".to_string(),
                model: "gpt-4".to_string(),
            },
            hotkeys: HotkeyConfig {
                push_to_talk: "Ctrl+Shift+Space".to_string(),
                hands_free_toggle: "Ctrl+Shift+F".to_string(),
                emergency_stop: "Ctrl+Shift+Escape".to_string(),
            },
            tray: TrayConfig {
                show_notifications: true,
                quick_languages: vec!["English".to_string(), "Spanish".to_string(), "Portuguese".to_string()],
                available_microphones: HashMap::new(),
            },
            privacy_mode: false,
            text_insertion_enabled: true,
            auto_save_enabled: true,
        }
    }
}

// Tauri commands for settings management
#[tauri::command]
pub async fn load_settings() -> Result<Settings, String> { /* ... */ }

#[tauri::command] 
pub async fn save_settings(settings: Settings) -> Result<(), String> { /* ... */ }

#[tauri::command]
pub async fn get_api_key() -> Result<String, String> { /* Load from stronghold */ }

#[tauri::command]
pub async fn set_api_key(key: String) -> Result<(), String> { /* Save to stronghold */ }

#[tauri::command]
pub async fn get_available_microphones() -> Result<HashMap<String, String>, String> { /* ... */ }

#[tauri::command]
pub async fn get_available_languages() -> Result<Vec<String>, String> { /* ... */ }
```

**system/tray.rs**

Rich system tray implementation with context menus.

```rust
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_tray::{Tray, TrayIcon, TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState};

pub struct TrayManager {
    app_handle: AppHandle,
}

impl TrayManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }

    pub fn create_tray(&self) -> Result<(), Box<dyn std::error::Error>> {
        let tray = TrayIconBuilder::new()
            .icon(self.get_tray_icon())
            .tooltip("TalkToMe")
            .on_tray_icon_event(|tray, event| {
                match event {
                    TrayIconEvent::Click { button: MouseButton::Right, .. } => {
                        self.show_context_menu(tray);
                    }
                    TrayIconEvent::Click { button: MouseButton::Left, .. } => {
                        self.toggle_main_window();
                    }
                    _ => {}
                }
            })
            .build()?;

        Ok(())
    }

    fn show_context_menu(&self, tray: &TrayIcon) {
        let menu = self.build_context_menu();
        tray.set_menu(Some(menu));
    }

    fn build_context_menu(&self) -> Menu {
        // Build hierarchical menu:
        // - About
        // - Open Settings  
        // - ─────────────
        // - Change Microphone >
        //   - Default Microphone
        //   - USB Microphone (Yeti)
        //   - Bluetooth Headset
        // - Select Language >
        //   - English
        //   - Spanish  
        //   - Portuguese
        //   - More Languages...
        // - Translation Language >
        //   - English
        //   - Spanish
        //   - Portuguese 
        //   - Disable Translation
        // - ─────────────
        // - Quit
    }

    fn toggle_main_window(&self) {
        if let Some(window) = self.app_handle.get_window("main") {
            if window.is_visible().unwrap_or(false) {
                window.hide().ok();
            } else {
                window.show().ok();
                window.set_focus().ok();
            }
        }
    }
}

#[tauri::command]
pub async fn update_tray_microphone(mic_id: String, mic_name: String) -> Result<(), String> { /* ... */ }

#[tauri::command]
pub async fn update_tray_language(language: String) -> Result<(), String> { /* ... */ }

#[tauri::command]
pub async fn show_about_dialog() -> Result<(), String> { /* ... */ }
```

**audio/control.rs**

Platform-specific audio session management.

```rust
use std::sync::atomic::{AtomicBool, Ordering};

pub struct AudioControlManager {
    auto_mute_enabled: AtomicBool,
    was_audio_playing: AtomicBool,
}

impl AudioControlManager {
    pub fn new() -> Self {
        Self {
            auto_mute_enabled: AtomicBool::new(true),
            was_audio_playing: AtomicBool::new(false),
        }
    }

    pub fn set_auto_mute_enabled(&self, enabled: bool) {
        self.auto_mute_enabled.store(enabled, Ordering::Relaxed);
    }

    pub async fn mute_system_audio(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.auto_mute_enabled.load(Ordering::Relaxed) {
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            self.mute_windows_audio().await
        }

        #[cfg(target_os = "linux")]
        {
            self.mute_linux_audio().await
        }
    }

    pub async fn restore_system_audio(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.auto_mute_enabled.load(Ordering::Relaxed) {
            return Ok(());
        }

        if self.was_audio_playing.load(Ordering::Relaxed) {
            #[cfg(target_os = "windows")]
            {
                self.unmute_windows_audio().await
            }

            #[cfg(target_os = "linux")]
            {
                self.unmute_linux_audio().await
            }
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn mute_windows_audio(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Use Windows Core Audio APIs to detect and pause/mute audio sessions
        // Store previous state in was_audio_playing
    }

    #[cfg(target_os = "linux")]
    async fn mute_linux_audio(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Use PulseAudio/PipeWire APIs to detect and pause/mute audio streams
        // Store previous state in was_audio_playing
    }
}

#[tauri::command]
pub async fn set_audio_auto_mute(enabled: bool) -> Result<(), String> { /* ... */ }

#[tauri::command]
pub async fn test_audio_control() -> Result<String, String> { /* Test mute/unmute functionality */ }
```

**system/text_insertion.rs**

Cross-platform text insertion service.

```rust
use std::time::Duration;

pub struct TextInsertionService;

impl TextInsertionService {
    pub fn new() -> Self {
        Self
    }

    pub async fn insert_text(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            self.insert_text_windows(text).await
        }

        #[cfg(target_os = "linux")]
        {
            self.insert_text_linux(text).await
        }
    }

    #[cfg(target_os = "windows")]
    async fn insert_text_windows(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Use Windows APIs:
        // 1. Get active window handle
        // 2. Use clipboard + Ctrl+V simulation
        // 3. Or use SendInput for direct text input
        // 4. Handle special cases like password fields
    }

    #[cfg(target_os = "linux")]
    async fn insert_text_linux(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Use X11/Wayland APIs:
        // 1. Detect active window
        // 2. Use xdotool or wl-clipboard
        // 3. Simulate clipboard paste or direct typing
    }

    pub async fn get_active_window_info(&self) -> Result<WindowInfo, Box<dyn std::error::Error>> {
        // Get information about the currently active window
        // to determine best insertion method
    }
}

#[derive(Debug)]
pub struct WindowInfo {
    pub title: String,
    pub class: String,
    pub is_text_field: bool,
    pub supports_direct_input: bool,
}

#[tauri::command]
pub async fn insert_text_to_active_window(text: String) -> Result<(), String> { /* ... */ }

#[tauri::command]
pub async fn test_text_insertion() -> Result<(), String> { /* Insert test text */ }
```

#### **4.2 Svelte UI Design**

The frontend code lives in the `src` directory.

**SettingsModal.svelte**

Enhanced settings modal with comprehensive configuration options.

```svelte
<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { settings, loadSettings, saveSettings } from './stores/settings';
  import TabGroup from './components/TabGroup.svelte';
  import MicrophoneSelector from './components/MicrophoneSelector.svelte';
  import LanguageSelector from './components/LanguageSelector.svelte';
  import HotkeyInput from './components/HotkeyInput.svelte';
  import ApiKeyInput from './components/ApiKeyInput.svelte';

  let activeTab = 'general';
  let isVisible = false;

  const tabs = [
    { id: 'general', label: 'General', icon: '⚙️' },
    { id: 'audio', label: 'Audio', icon: '🎤' },
    { id: 'hotkeys', label: 'Hotkeys', icon: '⌨️' },
    { id: 'translation', label: 'Translation', icon: '🌐' },
    { id: 'api', label: 'API', icon: '🔑' },
    { id: 'advanced', label: 'Advanced', icon: '🔧' }
  ];

  async function handleSave() {
    try {
      await saveSettings($settings);
      await invoke('apply_settings', { settings: $settings });
      isVisible = false;
    } catch (error) {
      console.error('Failed to save settings:', error);
    }
  }

  function handleCancel() {
    loadSettings(); // Reload from disk
    isVisible = false;
  }
</script>

{#if isVisible}
<div class="modal-overlay" on:click={handleCancel}>
  <div class="modal-content" on:click|stopPropagation>
    <div class="modal-header">
      <h2>TalkToMe Settings</h2>
      <button class="close-btn" on:click={handleCancel}>×</button>
    </div>
    
    <div class="modal-body">
      <TabGroup {tabs} bind:activeTab>
        <!-- General Tab -->
        {#if activeTab === 'general'}
        <div class="settings-section">
          <h3>Appearance</h3>
          <label class="setting-item">
            <span>Theme</span>
            <select bind:value={$settings.theme}>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
              <option value="system">System</option>
            </select>
          </label>

          <h3>Behavior</h3>
          <label class="setting-item checkbox">
            <input type="checkbox" bind:checked={$settings.text_insertion_enabled} />
            <span>Insert text into active applications</span>
          </label>
          
          <label class="setting-item checkbox">
            <input type="checkbox" bind:checked={$settings.auto_save_enabled} />
            <span>Auto-save transcriptions</span>
          </label>
          
          <label class="setting-item checkbox">
            <input type="checkbox" bind:checked={$settings.privacy_mode} />
            <span>Privacy mode (disable all network requests)</span>
          </label>
        </div>
        {/if}

        <!-- Audio Tab -->
        {#if activeTab === 'audio'}
        <div class="settings-section">
          <h3>Microphone</h3>
          <MicrophoneSelector bind:selectedId={$settings.audio.selected_mic_id} />
          
          <label class="setting-item">
            <span>Input Gain</span>
            <input type="range" min="0" max="2" step="0.1" bind:value={$settings.audio.input_gain} />
            <span class="gain-value">{$settings.audio.input_gain.toFixed(1)}x</span>
          </label>

          <h3>Audio Management</h3>
          <label class="setting-item checkbox">
            <input type="checkbox" bind:checked={$settings.audio.auto_mute_enabled} />
            <span>Auto-mute music during dictation</span>
          </label>
          
          <label class="setting-item checkbox">
            <input type="checkbox" bind:checked={$settings.audio.noise_suppression} />
            <span>Enable noise suppression</span>
          </label>
        </div>
        {/if}

        <!-- Hotkeys Tab -->
        {#if activeTab === 'hotkeys'}
        <div class="settings-section">
          <h3>Global Shortcuts</h3>
          <HotkeyInput 
            label="Push to Talk"
            bind:value={$settings.hotkeys.push_to_talk}
            description="Hold this key combination while speaking"
          />
          
          <HotkeyInput 
            label="Hands-free Toggle"
            bind:value={$settings.hotkeys.hands_free_toggle}
            description="Toggle continuous listening mode"
          />
          
          <HotkeyInput 
            label="Emergency Stop"
            bind:value={$settings.hotkeys.emergency_stop}
            description="Immediately stop all dictation"
          />
          
          <button class="reset-btn" on:click={() => invoke('reset_hotkeys_to_default')}>
            Reset to Defaults
          </button>
        </div>
        {/if}

        <!-- Translation Tab -->
        {#if activeTab === 'translation'}
        <div class="settings-section">
          <label class="setting-item checkbox">
            <input type="checkbox" bind:checked={$settings.translation.enabled} />
            <span>Enable live translation</span>
          </label>
          
          {#if $settings.translation.enabled}
          <LanguageSelector 
            label="Source Language"
            bind:value={$settings.translation.source_language}
            includeAutoDetect={true}
          />
          
          <LanguageSelector 
            label="Target Language"
            bind:value={$settings.translation.target_language}
          />
          
          <label class="setting-item">
            <span>Translation Model</span>
            <select bind:value={$settings.translation.model}>
              <option value="gpt-4">GPT-4</option>
              <option value="gpt-3.5-turbo">GPT-3.5 Turbo</option>
              <option value="claude-3-sonnet">Claude 3 Sonnet</option>
            </select>
          </label>
          {/if}
        </div>
        {/if}

        <!-- API Tab -->
        {#if activeTab === 'api'}
        <div class="settings-section">
          <h3>OpenAI-Compatible Endpoints</h3>
          <label class="setting-item">
            <span>Whisper (STT) Endpoint</span>
            <input type="url" bind:value={$settings.api.whisper_endpoint} />
          </label>
          
          <label class="setting-item">
            <span>Chat Completions Endpoint</span>
            <input type="url" bind:value={$settings.api.chat_endpoint} />
          </label>
          
          <ApiKeyInput />
          
          <button class="test-btn" on:click={() => invoke('test_api_connection')}>
            Test Connection
          </button>
        </div>
        {/if}

        <!-- Advanced Tab -->
        {#if activeTab === 'advanced'}
        <div class="settings-section">
          <h3>System Tray</h3>
          <label class="setting-item checkbox">
            <input type="checkbox" bind:checked={$settings.tray.show_notifications} />
            <span>Show system notifications</span>
          </label>
          
          <h3>Dictation Language</h3>
          <LanguageSelector 
            label="Primary Language"
            bind:value={$settings.dictation_language}
          />
          
          <h3>Quick Actions</h3>
          <label class="setting-item">
            <span>Quick Languages (for tray menu)</span>
            <!-- Multi-select for quick_languages -->
          </label>
          
          <h3>Data & Privacy</h3>
          <button class="danger-btn" on:click={() => invoke('clear_all_data')}>
            Clear All Data
          </button>
        </div>
        {/if}
      </TabGroup>
    </div>
    
    <div class="modal-footer">
      <button class="btn-secondary" on:click={handleCancel}>Cancel</button>
      <button class="btn-primary" on:click={handleSave}>Save Changes</button>
    </div>
  </div>
</div>
{/if}

<style>
  /* Modal styling with Tailwind classes */
  .modal-overlay {
    @apply fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50;
  }
  
  .modal-content {
    @apply bg-white dark:bg-gray-800 rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] overflow-hidden;
  }
  
  .modal-header {
    @apply flex justify-between items-center p-6 border-b border-gray-200 dark:border-gray-700;
  }
  
  .modal-body {
    @apply overflow-y-auto max-h-[60vh];
  }
  
  .settings-section {
    @apply p-6 space-y-4;
  }
  
  .setting-item {
    @apply flex items-center justify-between gap-4 p-3 border border-gray-200 dark:border-gray-700 rounded-lg;
  }
  
  .setting-item.checkbox {
    @apply flex-row-reverse justify-end;
  }
  
  /* Additional component styles */
</style>
```

**Svelte Stores (`src/stores`)**

**settings.ts**
```typescript
import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/tauri';

export interface Settings {
  theme: string;
  dictation_language: string;
  api: {
    whisper_endpoint: string;
    chat_endpoint: string;
    api_key_set: boolean;
  };
  audio: {
    auto_mute_enabled: boolean;
    selected_mic_id?: string;
    input_gain: number;
    noise_suppression: boolean;
  };
  translation: {
    enabled: boolean;
    target_language: string;
    source_language: string;
    model: string;
  };
  hotkeys: {
    push_to_talk: string;
    hands_free_toggle: string;
    emergency_stop: string;
  };
  tray: {
    show_notifications: boolean;
    quick_languages: string[];
    available_microphones: Record<string, string>;
  };
  privacy_mode: boolean;
  text_insertion_enabled: boolean;
  auto_save_enabled: boolean;
}

export const settings = writable<Settings>();

export async function loadSettings() {
  try {
    const loadedSettings = await invoke<Settings>('load_settings');
    settings.set(loadedSettings);
    return loadedSettings;
  } catch (error) {
    console.error('Failed to load settings:', error);
    throw error;
  }
}

export async function saveSettings(newSettings: Settings) {
  try {
    await invoke('save_settings', { settings: newSettings });
    settings.set(newSettings);
  } catch (error) {
    console.error('Failed to save settings:', error);
    throw error;
  }
}
```

**dictation.ts**
```typescript
import { writable, derived } from 'svelte/store';

export interface DictationState {
  isActive: boolean;
  mode: 'push-to-talk' | 'hands-free' | 'idle';
  isTranscribing: boolean;
  isTranslating: boolean;
  currentText: string;
  error?: string;
}

export const dictationState = writable<DictationState>({
  isActive: false,
  mode: 'idle',
  isTranscribing: false,
  isTranslating: false,
  currentText: '',
});

export const isListening = derived(
  dictationState,
  ($state) => $state.isActive && $state.mode !== 'idle'
);

export const statusText = derived(
  dictationState,
  ($state) => {
    if ($state.error) return `Error: ${$state.error}`;
    if ($state.isTranslating) return 'Translating...';
    if ($state.isTranscribing) return 'Transcribing...';
    if ($state.isActive) return `Listening (${$state.mode})`;
    return 'Ready';
  }
);
```

**Event Handling (`App.svelte`)**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { invoke } from '@tauri-apps/api/tauri';
  import { dictationState } from './stores/dictation';
  import { settings, loadSettings } from './stores/settings';
  import { editorContent } from './stores/editor';

  onMount(async () => {
    // Load settings on startup
    await loadSettings();

    // Listen for global hotkey events from Rust
    await listen('push_to_talk_start', () => {
      dictationState.update(state => ({ ...state, isActive: true, mode: 'push-to-talk' }));
      invoke('start_dictation');
    });

    await listen('push_to_talk_stop', () => {
      dictationState.update(state => ({ ...state, isActive: false, mode: 'idle' }));
      invoke('stop_dictation');
    });

    await listen('hands_free_toggle', () => {
      dictationState.update(state => ({
        ...state,
        isActive: !state.isActive,
        mode: state.isActive ? 'idle' : 'hands-free'
      }));
      invoke('toggle_dictation');
    });

    await listen('emergency_stop', () => {
      dictationState.update(state => ({
        ...state,
        isActive: false,
        mode: 'idle',
        error: undefined
      }));
      invoke('emergency_stop_dictation');
    });

    // Listen for transcription/translation results
    await listen('transcription_start', () => {
      dictationState.update(state => ({ ...state, isTranscribing: true }));
    });

    await listen('transcription_complete', (event) => {
      dictationState.update(state => ({
        ...state,
        isTranscribing: false,
        currentText: event.payload as string
      }));
    });

    await listen('translation_start', () => {
      dictationState.update(state => ({ ...state, isTranslating: true }));
    });

    await listen('translation_complete', (event) => {
      dictationState.update(state => ({
        ...state,
        isTranslating: false,
        currentText: event.payload as string
      }));
    });

    await listen('text_inserted', (event) => {
      // Text was successfully inserted into active application
      if ($settings.auto_save_enabled) {
        editorContent.update(content => content + (event.payload as string) + '\n');
      }
    });

    await listen('dictation_error', (event) => {
      dictationState.update(state => ({
        ...state,
        isActive: false,
        mode: 'idle',
        isTranscribing: false,
        isTranslating: false,
        error: event.payload as string
      }));
    });

    // Listen for system tray events
    await listen('tray_microphone_changed', (event) => {
      const { micId, micName } = event.payload as { micId: string; micName: string };
      settings.update(s => ({
        ...s,
        audio: { ...s.audio, selected_mic_id: micId }
      }));
    });

    await listen('tray_language_changed', (event) => {
      const language = event.payload as string;
      settings.update(s => ({ ...s, dictation_language: language }));
    });

    await listen('tray_translation_language_changed', (event) => {
      const language = event.payload as string;
      settings.update(s => ({
        ...s,
        translation: { ...s.translation, target_language: language }
      }));
    });
  });
</script>
```

------

### **5. Functional Requirements (User Stories)**

| ID         | As a…      | I want to…                                                   | So that I…                                                   |
| ---------- | ---------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| **FR-001** | User       | Start dictating by holding down a global hotkey ("Push to talk"). | can quickly dictate short phrases while in any application.  |
| **FR-002** | User       | Toggle dictation on and off by pressing a global hotkey ("Hands-free"). | can dictate long-form content without holding a key.         |
| **FR-003** | User       | Configure separate keyboard shortcuts for "Push to talk", "Hands-free", and "Emergency stop" modes in the settings. | can customize the controls to my preference and avoid conflicts. |
| **FR-004** | User       | Select my spoken language from a list in the settings.       | get accurate transcription for my native language or the language I'm using. |
| **FR-005** | User       | Enable a "Live Translation" mode with source and target language selection. | can speak in one language and see the text appear in another. |
| **FR-006** | User       | Configure OpenAI-compatible API endpoints and securely store my API key. | can use my preferred speech and translation services while keeping my credentials safe. |
| **FR-007** | User       | Have transcribed/translated text automatically inserted into the currently active text input. | don't need to manually copy and paste the results.           |
| **FR-008** | User       | Enable auto-mute of music and media during dictation.        | can dictate clearly without background noise interfering with transcription. |
| **FR-009** | User       | Right-click the system tray icon to quickly change microphones. | can easily switch between different audio input devices.     |
| **FR-010** | User       | Right-click the system tray icon to quickly change dictation or translation languages. | can adapt to different conversation contexts without opening settings. |
| **FR-011** | User       | Access an "About" dialog and open settings from the system tray. | can get app information and access configuration options quickly. |
|            |            |                                                              |                                                              |
|            |            |                                                              |                                                              |
| **FR-012** | User       | Switch between light and dark themes.                        | can use the app comfortably in different lighting conditions. |
| **FR-013** | User       | Save, open, and export my dictated documents.                | can preserve and share my transcribed content.               |
| **FR-014** | Power User | Test microphone levels and API connectivity from settings.   | can verify my configuration is working correctly.            |
| **FR-015** | Power User | Adjust microphone gain and enable noise suppression.         | can optimize audio quality for better transcription accuracy. |
| **FR-016** | Power User | Configure which languages appear in the quick-access tray menu. | can streamline my workflow for frequently used languages.    |

------

### **6. Non-Functional Requirements**

| Category                         | Requirement                                                  |
| -------------------------------- | ------------------------------------------------------------ |
| **Performance**                  | Total latency (speech-to-translated-text) should be under 2000ms for good UX. Local audio processing latency ≤ 100ms. Text insertion latency ≤ 50ms. |
| **Security & Privacy**           | When Privacy Mode is ON, no network activity occurs. API keys must be stored using `tauri-plugin-stronghold`. Audio data is never stored permanently. |
| **Reliability**                  | The application must gracefully handle API errors (invalid key, network issues, rate limits) by showing notifications and falling back to previous text. Audio control failures should not crash the app. |
| **Usability**                    | The distinction between dictation, translation, and command modes must be clear through visual indicators. System tray menu should be intuitive and responsive. Settings should be well-organized and searchable. |
| **Accessibility**                | The app should work with screen readers. Keyboard navigation should be fully supported. High contrast modes should be respected. |
| **Cross-Platform Compatibility** | Audio management features must work on both Windows (WASAPI) and Linux (PulseAudio/PipeWire). Text insertion must work with all major applications on both platforms. |
| **Resource Usage**               | Memory usage should stay under 150MB during normal operation. CPU usage should be minimal when idle. Audio processing should not introduce audible latency. |
| **Internationalization (I18n)**  | The UI text itself (buttons, labels) should be localizable via JSON files. This is separate from the dictation/translation language settings. |
| **Network Resilience**           | The app should handle network failures gracefully, queue failed requests for retry, and provide clear status indicators for network operations. |
| **System Integration**           | The app should integrate seamlessly with the OS notification system, respect system audio device changes, and handle display scaling properly. |

------

### **7. Edge Cases & Error Handling**

| Edge Case                                   | Expected Behavior                                            |
| ------------------------------------------- | ------------------------------------------------------------ |
| **No microphone available**                 | Show clear error message, disable dictation features, allow settings configuration for when mic becomes available. |
| **API key invalid/expired**                 | Show notification, log specific error, fall back to non-translated transcription if possible. |
| **Network connectivity lost**               | Queue requests for retry, show offline indicator, continue with local features that don't require network. |
| **Target app doesn't accept text input**    | Detect and notify user, offer to copy to clipboard as fallback. |
| **Audio device unplugged during dictation** | Gracefully stop dictation, notify user, attempt to switch to default device. |
| **Hotkey conflicts with other apps**        | Detect conflicts during registration, notify user, provide suggestions for alternative combinations. |
| **Translation API rate limiting**           | Implement exponential backoff, show rate limit warnings, allow user to configure retry behavior. |
| **Very long audio segments**                | Chunk audio into manageable segments, show progress indicators, handle partial failures gracefully. |
| **Text insertion into password fields**     | Detect password fields and either warn user or disable auto-insertion for security. |
| **System audio control fails**              | Log warning but continue with dictation, notify user that auto-mute is temporarily disabled. |
| **Settings file corrupted**                 | Load default settings, backup corrupted file, notify user of reset. |
| **Simultaneous hotkey presses**             | Implement proper state management to handle overlapping input gracefully. |
| **App loses focus during dictation**        | Continue dictation but make status clearly visible, ensure text insertion still works. |
| **System sleep/hibernate during operation** | Properly clean up audio resources, restore state on wake.    |
| **Multiple instances launched**             | Prevent multiple instances or gracefully handle shared resources. |

------

### **8. System Tray Menu Structure**

```
TalkToMe (Tray Icon)
├── About TalkToMe
├── Open Settings
├── ─────────────────────
├── Change Microphone ►
│   ├── 🎤 Default Microphone
│   ├── 🎤 USB Microphone (Yeti Blue) ✓
│   ├── 🎤 Bluetooth Headset (AirPods)
│   └── 🎤 Refresh Device List
├── Select Language ►
│   ├── 🇺🇸 English ✓
│   ├── 🇪🇸 Spanish
│   ├── 🇵🇹 Portuguese
│   ├── 🇫🇷 French
│   ├── 🇩🇪 German
│   └── More Languages...
├── Translation Language ►
│   ├── 🇺🇸 English
│   ├── 🇪🇸 Spanish ✓
│   ├── 🇵🇹 Portuguese
│   ├── 🇫🇷 French
│   ├── 🇩🇪 German
│   ├── ─────────────────
│   └── ❌ Disable Translation
├── ─────────────────────
└── Quit TalkToMe
```

------

### **9. Development & Test Plan**

#### **9.1 Sprint Breakdown (Enhanced)**

- **Sprint 1:** Project Scaffolding & Basic UI
  - Set up Tauri + Svelte + Tailwind project structure
  - Create basic main window and settings modal
  - Implement theme switching and settings persistence
- **Sprint 2:** Audio Foundation & System Integration
  - Implement audio capture with microphone selection
  - Create system tray with basic menu structure
  - Add audio control manager for mute/unmute functionality
- **Sprint 3:** Global Hotkey Integration & Dictation Flow
  - Implement `tauri-plugin-global-shortcut` integration
  - Create hotkey settings UI with conflict detection
  - Build push-to-talk and hands-free mode state management
- **Sprint 4:** STT Service & OpenAI Integration
  - Implement Whisper API client with audio streaming
  - Add API configuration (api_base and api_key) and testing utilities
  - Create secure API key storage with Stronghold
- **Sprint 5:** Translation Service & Live Translation
  - Implement chat/completions API client for translation
  - Build translation settings UI and language selection
  - Integrate translation pipeline with STT results
- **Sprint 6:** Text Insertion & Cross-Platform Features
  - Implement platform-specific text insertion services
  - Add clipboard fallback and target application detection
  - Create text insertion testing and validation tools
- **Sprint 7:** Enhanced System Tray & Quick Actions
  - Build hierarchical context menus for tray icon
  - Implement quick language and microphone switching
  - Add about dialog and settings shortcuts
- **Sprint 8:** Testing, Polish & Documentation
  - Comprehensive testing across Windows and Linux
  - Performance optimization and error handling refinement
  - User documentation and developer guides
- **Sprint 9:** Packaging & Distribution
  - Create installers for Windows and Linux
  - Final security audit and code review

#### **9.2 Testing Strategy**

| Test Category           | Coverage                               | Tools                                     |
| ----------------------- | -------------------------------------- | ----------------------------------------- |
| **Unit Tests**          | Core Rust modules (80%+ coverage)      | `cargo test` with mock audio/API services |
| **Integration Tests**   | API interactions, settings persistence | Test with mock OpenAI endpoints           |
| **UI Tests**            | Svelte components and stores           | Vitest + Testing Library                  |
| **E2E Tests**           | Complete user workflows                | Tauri's WebDriver integration             |
| **Platform Tests**      | Windows/Linux specific features        | Automated CI on both platforms            |
| **Performance Tests**   | Audio latency, memory usage            | Custom benchmarking tools                 |
| **Security Tests**      | API key storage, network isolation     | Manual security review                    |
| **Accessibility Tests** | Screen reader compatibility            | Manual testing with assistive tools       |

#### **9.3 CI/CD Pipeline**

```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: 18
      - name: Install dependencies
        run: |
          npm install
          cargo build
      - name: Run tests
        run: |
          cargo test
          npm test
      - name: Build application
        run: npm run tauri build
      
  release:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: test
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - name: Build and Release
        run: npm run tauri build
      - name: Upload Release Assets
        uses: softprops/action-gh-release@v1
        with:
          files: src-tauri/target/release/bundle/**/*
```

------

### **10. Security Considerations**

| Security Aspect           | Implementation                                               | Risk Level |
| ------------------------- | ------------------------------------------------------------ | ---------- |
| **API Key Storage**       | Use `tauri-plugin-stronghold` for encrypted storage, never log keys | High       |
| **Network Communication** | HTTPS only, certificate validation, timeout handling         | Medium     |
| **Audio Data**            | Process in memory only, never write to disk, clear buffers immediately | High       |
| **Text Insertion**        | Validate target applications, provide password field detection | Medium     |
| **Privilege Escalation**  | Run with minimal required permissions, no unnecessary system access | Low        |
| **Update Mechanism**      | Verify signatures on updates, use secure channels            | Medium     |
| **Configuration Data**    | Sanitize all user inputs, validate settings format           | Low        |
| **System Integration**    | Limit system API usage to required functionality only        | Low        |

------

### **11. Deployment & Distribution**

#### **11.1 Supported Platforms**

| Platform | Version                         | Architecture | Package Format              |
| -------- | ------------------------------- | ------------ | --------------------------- |
| Windows  | 10+ (1809+)                     | x64          | MSI Installer, Portable EXE |
| Linux    | Ubuntu 20.04+, Fedora 35+, Arch | x64          | AppImage, .deb, .rpm        |

#### **11.2 Installation Requirements**

- **Windows:** Visual C++ Redistributable 2019+, Windows 10 1809+
- **Linux:** PulseAudio or PipeWire, X11 or Wayland, glibc 2.31+
- **Network:** Internet connection for transcription and translation (optional)
- **Hardware:** Microphone access, 4GB RAM, 100MB disk space

------

