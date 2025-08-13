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

