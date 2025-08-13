# Task

Your task is to implement the dictation flow as below. Note that the user can both press the Recording Button in the src/routes/+page.svelte to start talking/recording his voice or trigger the recording via hotkey combo.

### Dictation Flow 


1. User presses hotkey → Global hotkey manager catches event 
2. Audio control manager mutes system audio (if enabled) 
3. Audio capture starts recording from selected microphone 
4. Audio data sent to STT service (Whisper small model running locally)
5. If translation enabled, text sent to translation service 
6. Final text inserted into active application 
7. Audio control manager restores system audio 


### Workflow (Simplest Possible) 

```
graph TD
    A[User speaks] --> B{Tauri Frontend}
    B --> C[Capture mic: 16 kHz mono PCM]
    C --> D[Send to Rust via invoke]
    D --> E[Rust: Write PCM → temp .wav]
    E --> F[Run whisper.cpp small: --language auto --print-realtime]
    F --> G[Parse output: language + transcribed text]
    G --> H[Tauri: emit "transcription", text]
    H --> I[Show in app UI]
```