use tauri_app_lib::error::{TalkToMeError, Result};

#[cfg(test)]
mod tests {
    use super::*;
    use tauri_app_lib::validate_api_credentials;

    #[test]
    fn test_validate_api_credentials_valid() {
        let result = validate_api_credentials("https://api.example.com", "sk-valid-api-key-1234567890");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_api_credentials_empty_endpoint() {
        let result = validate_api_credentials("", "sk-valid-api-key-1234567890");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TalkToMeError::InvalidApiEndpoint));
    }

    #[test]
    fn test_validate_api_credentials_invalid_endpoint() {
        let result = validate_api_credentials("invalid-endpoint", "sk-valid-api-key-1234567890");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TalkToMeError::InvalidApiEndpoint));
    }

    #[test]
    fn test_validate_api_credentials_empty_api_key() {
        let result = validate_api_credentials("https://api.example.com", "");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TalkToMeError::InvalidApiKey));
    }

    #[test]
    fn test_validate_api_credentials_whitespace_endpoint() {
        let result = validate_api_credentials("   ", "sk-valid-api-key-1234567890");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TalkToMeError::InvalidApiEndpoint));
    }

    #[test]
    fn test_validate_api_credentials_whitespace_api_key() {
        let result = validate_api_credentials("https://api.example.com", "   ");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), TalkToMeError::InvalidApiKey));
    }

    #[test]
    fn test_talk_to_me_error_display() {
        let err = TalkToMeError::InvalidApiEndpoint;
        assert_eq!(err.to_string(), "API endpoint is empty or invalid");

        let err = TalkToMeError::InvalidApiKey;
        assert_eq!(err.to_string(), "API key is empty or invalid");

        let err = TalkToMeError::RecordingInProgress;
        assert_eq!(err.to_string(), "Recording already in progress");

        let err = TalkToMeError::NoActiveRecording;
        assert_eq!(err.to_string(), "No active recording to stop");

        let err = TalkToMeError::RecordingTimeout;
        assert_eq!(err.to_string(), "Recording timeout exceeded");
    }

    #[test]
    fn test_error_conversion() {
        // Test that std::io::Error converts to TalkToMeError
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let talktome_err: TalkToMeError = io_err.into();
        assert!(matches!(talktome_err, TalkToMeError::IoError(_)));

        // Test that serde_json::Error converts to TalkToMeError
        let json_str = "{ invalid json }";
        let json_result: std::result::Result<serde_json::Value, serde_json::Error> = serde_json::from_str(json_str);
        assert!(json_result.is_err());
        if let Err(e) = json_result {
            let talktome_err: TalkToMeError = e.into();
            assert!(matches!(talktome_err, TalkToMeError::JsonError(_)));
        }
    }
}
