use crate::debug_logger::DebugLogger;
use reqwest;
use serde_json::{Value, json};
use std::time::Duration;

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
    pub async fn process_text(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
        translate_enabled: bool,
    ) -> Result<String, String> {
        DebugLogger::log_info("=== TRANSLATION: process_text() called ===");
        DebugLogger::log_info(&format!(
            "TRANSLATION: Input params - text='{}', source_lang={}, target_lang={}, translate_enabled={}",
            text, source_lang, target_lang, translate_enabled
        ));

        let prompt = if translate_enabled && target_lang != "none" && target_lang != source_lang {
            // Translation + correction mode
            DebugLogger::log_info("TRANSLATION: Mode = Translation + Correction");
            if source_lang == "auto" {
                format!(
                    "Please correct any grammar, punctuation, or spelling errors, remove any adjacent duplicates, \
                     and render the text in native-level {}. Return only the edited translation, with no extra commentary:\n\n{}",
                    self.get_language_name(target_lang),
                    text
                )
            } else {
                format!(
                    "Please translate the following text from {} to {}, then correct any grammar, punctuation, or spelling errors, \
                     remove any adjacent duplicates, and render the text in native-level {}. Return only the edited translation, \
                     with no extra commentary:\n\n{}",
                    self.get_language_name(source_lang),
                    self.get_language_name(target_lang),
                    self.get_language_name(target_lang),
                    text
                )
            }
        } else {
            // Correction only mode
            DebugLogger::log_info("TRANSLATION: Mode = Correction only");
            format!(
                "Please correct any grammar, punctuation, and spelling errors in the following text. \
                Keep the same language and meaning, just fix any errors, remove duplicated adjacent words and normalize spaces. \
                Provide only the corrected text without any additional commentary:\n\n{}",
                text
            )
        };

        DebugLogger::log_translation_request(
            text,
            source_lang,
            target_lang,
            translate_enabled,
            &prompt,
        );

        self.send_chat_request(&prompt).await
    }

    async fn send_chat_request(&self, prompt: &str) -> Result<String, String> {
        DebugLogger::log_info("=== TRANSLATION: send_chat_request() called ===");
        DebugLogger::log_info(&format!(
            "TRANSLATION: Prompt length: {} chars",
            prompt.len()
        ));

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

        // Log the API request details (without sensitive data)
        let url = format!("{}/chat/completions", self.api_endpoint);
        DebugLogger::log_info(&format!("TRANSLATION: Sending request to {}", url));
        DebugLogger::log_info(&format!("TRANSLATION: Model: {}, Temperature: 0.3, Max tokens: 1000", self.model));

        let max_retries = 3;
        let mut last_error: Option<String> = None;

        for attempt in 1..=max_retries {
            DebugLogger::log_info(&format!("TRANSLATION attempt {}/{} to {}", attempt, max_retries, url));

            // Send request to chat completion API
            DebugLogger::log_info("TRANSLATION: Sending HTTP POST request");
            let response = self
                .client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    DebugLogger::log_info(&format!("Translation API response status: {}", status));

                    if resp.status().is_success() {
                        DebugLogger::log_info("TRANSLATION: Response is successful, reading response text");
                        let response_text = resp.text().await.map_err(|e| {
                            let error_msg = format!("Failed to read response body: {}", e);
                            DebugLogger::log_pipeline_error("translation", &error_msg);
                            error_msg
                        })?;

                        DebugLogger::log_info(&format!("Translation API raw response (truncated): {}...", &response_text[..response_text.len().min(200)]));

                        DebugLogger::log_info("TRANSLATION: Parsing JSON response");
                        let json: Value = serde_json::from_str(&response_text).map_err(|e| {
                            let error_msg = format!("Invalid JSON response: {}", e);
                            DebugLogger::log_pipeline_error("translation", &error_msg);
                            error_msg
                        })?;

                        // Extract the translated text from the response
                        if let Some(choices) = json["choices"].as_array() {
                            if let Some(choice) = choices.get(0) {
                                if let Some(message) = choice["message"]["content"].as_str() {
                                    let result = message.trim().to_string();
                                    if result.is_empty() {
                                        let error_msg = "API returned empty translation text";
                                        DebugLogger::log_pipeline_error("translation", error_msg);
                                        return Err(error_msg.to_string());
                                    }
                                    DebugLogger::log_info(&format!("TRANSLATION: Successfully extracted text ({} chars)", result.len()));
                                    return Ok(result);
                                }
                            }
                        }

                        let error_msg = format!("API response missing expected content. Available structure: {:?}",
                            json.as_object().map(|o| o.keys().collect::<Vec<_>>()).unwrap_or_default());
                        DebugLogger::log_pipeline_error("translation", &error_msg);
                        DebugLogger::log_translation_response(false, None, Some(&error_msg), Some(&response_text));
                        return Err(error_msg);
                    } else {
                        DebugLogger::log_info("TRANSLATION: Response status indicates error, reading error details");

                        let error_text = resp.text().await.unwrap_or_else(|_| "Unable to read error response".to_string());
                        DebugLogger::log_info(&format!("Translation API error response: {}", error_text));

                        // Categorize errors for better handling
                        let error_msg = match status.as_u16() {
                            400 => format!("Bad request (400): Invalid parameters or prompt too long - {}", error_text),
                            401 => format!("Authentication failed (401): Invalid API key - {}", error_text),
                            403 => format!("Access forbidden (403): Insufficient permissions - {}", error_text),
                            404 => format!("API endpoint not found (404): Check API endpoint URL - {}", error_text),
                            413 => format!("Request too large (413): Prompt exceeds token limit - {}", error_text),
                            429 => format!("Rate limit exceeded (429): Too many requests - {}", error_text),
                            500..=599 => format!("Server error ({}): API service unavailable - {}", status, error_text),
                            _ => format!("API error ({}): {} - {}", status, status.canonical_reason().unwrap_or("Unknown"), error_text)
                        };

                        // Don't retry on client errors (4xx) except rate limits
                        if status.is_client_error() && status.as_u16() != 429 {
                            DebugLogger::log_pipeline_error("translation", &error_msg);
                            DebugLogger::log_translation_response(false, None, Some(&error_msg), Some(&error_text));
                            return Err(error_msg);
                        }

                        // Store error for potential retry
                        last_error = Some(error_msg.clone());
                        DebugLogger::log_info(&format!("TRANSLATION: Error is retryable, attempt {}/{}", attempt, max_retries));
                    }
                }
                Err(e) => {
                    let error_category = if e.is_timeout() {
                        "timeout"
                    } else if e.is_connect() {
                        "connection"
                    } else if e.is_request() {
                        "request"
                    } else {
                        "network"
                    };

                    let error_msg = format!("{} error: {}", error_category, e);
                    DebugLogger::log_info(&format!("TRANSLATION: {} error on attempt {}", error_category, attempt));
                    last_error = Some(error_msg);

                    // Don't retry on request errors (malformed requests)
                    if e.is_request() {
                        DebugLogger::log_pipeline_error("translation", &last_error.as_ref().unwrap());
                        DebugLogger::log_translation_response(false, None, Some(&last_error.as_ref().unwrap()), None);
                        return Err(last_error.unwrap());
                    }
                }
            }

            // Don't retry on the last attempt
            if attempt == max_retries {
                break;
            }

            // Exponential backoff with jitter
            let base_delay = Duration::from_millis(500);
            let exponential_delay = base_delay * (2_u32.pow(attempt - 1));
            let max_delay = Duration::from_secs(5);
            let delay = std::cmp::min(exponential_delay, max_delay);

            // Add small random jitter to prevent thundering herd
            let jitter = Duration::from_millis(fastrand::u64(0..100));
            let total_delay = delay + jitter;

            DebugLogger::log_info(&format!("TRANSLATION: Retrying in {}ms (attempt {}/{})", total_delay.as_millis(), attempt + 1, max_retries));
            tokio::time::sleep(total_delay).await;
        }

        // All retries exhausted
        let final_error = last_error.unwrap_or_else(|| "Unknown error after all retries".to_string());
        let error_msg = format!("Translation request failed after {} attempts: {}", max_retries, final_error);
        DebugLogger::log_pipeline_error("translation", &error_msg);
        DebugLogger::log_translation_response(false, None, Some(&error_msg), None);
        Err(error_msg)
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
