use reqwest;
use serde_json::{json, Value};

pub struct TranslationService {
    client: reqwest::Client,
    api_endpoint: String,
    api_key: String,
}

impl TranslationService {
    pub fn new(api_endpoint: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_endpoint,
            api_key,
        }
    }

    pub async fn translate_text(&self, text: &str, source_lang: &str, target_lang: &str) -> Result<String, String> {
        // Create the prompt for translation
        let prompt = if source_lang == "auto" {
            format!("Translate the following text to {}:\n\n{}", 
                    self.get_language_name(target_lang), text)
        } else {
            format!("Translate the following text from {} to {}:\n\n{}", 
                    self.get_language_name(source_lang), 
                    self.get_language_name(target_lang), 
                    text)
        };

        // Create the request body
        let body = json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 1000
        });

        // Send request to chat completion API
        let url = format!("{}/chat/completions", self.api_endpoint);
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let json: Value = response.json().await.map_err(|e| e.to_string())?;
            if let Some(translated_text) = json["choices"][0]["message"]["content"].as_str() {
                Ok(translated_text.trim().to_string())
            } else {
                Err("No translation in response".to_string())
            }
        } else {
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            Err(format!("API error: {}", error_text))
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