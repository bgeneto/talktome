use thiserror::Error;

#[derive(Error, Debug)]
pub enum TalkToMeError {
    #[error("API endpoint is empty or invalid")]
    InvalidApiEndpoint,

    #[error("API key is empty or invalid")]
    InvalidApiKey,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Audio capture error: {0}")]
    AudioCaptureError(String),

    #[error("STT processing error: {0}")]
    SttError(String),

    #[error("Translation error: {0}")]
    TranslationError(String),

    #[error("Text insertion error: {0}")]
    TextInsertionError(String),

    #[error("Settings error: {0}")]
    SettingsError(String),

    #[error("Hotkey error: {0}")]
    HotkeyError(String),

    #[error("System audio control error: {0}")]
    SystemAudioError(String),

    #[error("Recording already in progress")]
    RecordingInProgress,

    #[error("No active recording to stop")]
    NoActiveRecording,

    #[error("Recording timeout exceeded")]
    RecordingTimeout,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("CPAL audio error: {0}")]
    CpalError(#[from] cpal::BuildStreamError),

    #[error("Keyring error: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),

    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, TalkToMeError>;
