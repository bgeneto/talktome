use reqwest;
use serde_json::{json, Value};
use crate::debug_logger::DebugLogger;

pub struct TranslationService {
    client: reqwest::Client,
    api_endpoint: String,
    api_key: String,
    model: String,
}

impl TranslationService {
    pub fn new(api_endpoint: String, api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_endpoint,
            api_key,
            model,
        }
    }

    /// Process text with optional translation - always corrects grammar and punctuation
    pub async fn process_text(&self, text: &str, source_lang: &str, target_lang: &str, translate_enabled: bool) -> Result<String, String> {
        DebugLogger::log_info("=== TRANSLATION: process_text() called ===");
        DebugLogger::log_info(&format!("TRANSLATION: Input params - text='{}', source_lang={}, target_lang={}, translate_enabled={}", 
            text, source_lang, target_lang, translate_enabled));
        
        let prompt = if translate_enabled && target_lang != "none" && target_lang != source_lang {
            // Translation + correction mode
            DebugLogger::log_info("TRANSLATION: Mode = Translation + Correction");
            if source_lang == "auto" {
                format!(
                    "Please correct any grammar, punctuation, and spelling errors, remove duplicated adjacent words in the following text, then translate it to {}. \
                    Provide only the translated text without any additional commentary:\n\n{}", 
                    self.get_language_name(target_lang), 
                    text
                )
            } else {
                format!(
                    "Please correct any grammar, punctuation, and spelling errors, remove duplicated adjacent words in the following {} text, then translate it to {}. \
                    Provide only the translated text without any additional commentary:\n\n{}", 
                    self.get_language_name(source_lang),
                    self.get_language_name(target_lang), 
                    text
                )
            }
        } else {
            // Correction only mode
            DebugLogger::log_info("TRANSLATION: Mode = Correction only");
            format!(
                "Please correct any grammar, punctuation, and spelling errors in the following text. \
                Keep the same language and meaning, just fix any errors and duplicated adjacent words. \
                Provide only the corrected text without any additional commentary:\n\n{}", 
                text
            )
        };

        DebugLogger::log_translation_request(text, source_lang, target_lang, translate_enabled, &prompt);
        
        self.send_chat_request(&prompt).await
    }

    async fn send_chat_request(&self, prompt: &str) -> Result<String, String> {
        DebugLogger::log_info("=== TRANSLATION: send_chat_request() called ===");
        DebugLogger::log_info(&format!("TRANSLATION: Prompt length: {} chars", prompt.len()));
        
        // Create the request body
        let body = json!({
            "model": self.model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 1000
        });

        // Log the full API request
        let url = format!("{}/chat/completions", self.api_endpoint);
        DebugLogger::log_api_payload(&body, &url);

        // Send request to chat completion API
        DebugLogger::log_info("TRANSLATION: Sending HTTP POST request");
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                let error_msg = format!("Request failed: {}", e);
                DebugLogger::log_pipeline_error("translation", &error_msg);
                error_msg
            })?;

        let status = response.status();
        DebugLogger::log_info(&format!("Translation API response status: {}", status));
        DebugLogger::log_info(&format!("Translation API response headers: {:?}", response.headers()));

        if response.status().is_success() {
            DebugLogger::log_info("TRANSLATION: Response is successful, reading response text");
            let response_text = response.text().await
                .map_err(|e| {
                    let error_msg = format!("Failed to read response: {}", e);
                    DebugLogger::log_pipeline_error("translation", &error_msg);
                    error_msg
                })?;
            
            DebugLogger::log_info(&format!("Translation API raw response: {}", response_text));
            
            DebugLogger::log_info("TRANSLATION: Parsing JSON response");
            let json: Value = serde_json::from_str(&response_text)
                .map_err(|e| {
                    let error_msg = format!("JSON parsing error: {}", e);
                    DebugLogger::log_pipeline_error("translation", &error_msg);
                    error_msg
                })?;
            
            DebugLogger::log_info(&format!("TRANSLATION: Parsed JSON: {}", serde_json::to_string_pretty(&json).unwrap_or_default()));
            
            if let Some(translated_text) = json["choices"][0]["message"]["content"].as_str() {
                let result = translated_text.trim().to_string();
                DebugLogger::log_info(&format!("Translation API extracted text: '{}'", result));
                Ok(result)
            } else {
                let error_msg = "No translation in response".to_string();
                DebugLogger::log_pipeline_error("translation", &error_msg);
                DebugLogger::log_info(&format!("TRANSLATION: Available JSON structure: {}", serde_json::to_string_pretty(&json).unwrap_or_default()));
                DebugLogger::log_translation_response(false, None, Some(&error_msg), Some(&response_text));
                Err(error_msg)
            }
        } else {
            DebugLogger::log_info("TRANSLATION: Response status is not successful, reading error response");
            let error_text = response.text().await.unwrap_or_default();
            let error_msg = format!("API error: {} - {}", status, error_text);
            DebugLogger::log_pipeline_error("translation", &error_msg);
            DebugLogger::log_translation_response(false, None, Some(&error_msg), Some(&error_text));
            Err(error_msg)
        }
    }

    fn get_language_name(&self, lang_code: &str) -> &str {
        match lang_code {
            "en" => "English",
            "es" => "Spanish",
            "fr" => "French",
            "de" => "German",
            "it" => "Italian",
            "pt" => "Portuguese",
            "ru" => "Russian",
            "ja" => "Japanese",
            "ko" => "Korean",
            "zh" => "Chinese",
            _ => "English", // Default to English
        }
    }
}
