use reqwest;
use serde_json::Value;
use std::time::Duration;
use crate::debug_logger::DebugLogger;

pub struct STTService {
    client: reqwest::Client,
    api_endpoint: String,
    api_key: String,
    model: String,
}

impl STTService {
    pub fn new(api_endpoint: String, api_key: String, model: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_endpoint,
            api_key,
            model,
        }
    }

    /// Transcribe audio chunk with enhanced error handling
    pub async fn transcribe_chunk(&self, audio_data: Vec<f32>, sample_rate: u32) -> Result<String, String> {
        DebugLogger::log_info("=== STT: transcribe_chunk() called ===");
        DebugLogger::log_info(&format!("STT: Input audio_data.len()={}, sample_rate={}", audio_data.len(), sample_rate));
        
        if audio_data.is_empty() {
            DebugLogger::log_pipeline_error("stt", "Empty audio data provided");
            return Err("Empty audio data".to_string());
        }

        // Check for audio quality - skip if too quiet
        let max_amplitude = audio_data.iter().map(|&x| x.abs()).fold(0.0, f32::max);
        DebugLogger::log_info(&format!("STT: Audio quality check - max_amplitude={:.6}, threshold=0.01", max_amplitude));
        if max_amplitude < 0.01 {
            DebugLogger::log_info(&format!("Audio chunk too quiet (max_amplitude: {:.6}), returning empty", max_amplitude));
            return Ok(String::new()); // Return empty string for silent audio
        }

        // Convert f32 samples to i16 for WAV encoding
        DebugLogger::log_info("STT: Converting audio to WAV format");
        let audio_bytes = self.encode_wav(&audio_data, sample_rate)
            .map_err(|e| {
                let error_msg = format!("Audio encoding error: {}", e);
                DebugLogger::log_pipeline_error("stt", &error_msg);
                error_msg
            })?;
        DebugLogger::log_info(&format!("STT: WAV encoding complete, output size={} bytes", audio_bytes.len()));

        // Skip very small audio files (less than 1 second)
        let min_size = (sample_rate * 2) as usize; // 1 second of 16-bit audio
        DebugLogger::log_info(&format!("STT: Size check - audio_bytes.len()={}, min_size={}", audio_bytes.len(), min_size));
        if audio_bytes.len() < min_size {
            DebugLogger::log_info(&format!("Audio chunk too small ({} bytes), skipping", audio_bytes.len()));
            return Ok(String::new());
        }

        DebugLogger::log_transcription_request(audio_bytes.len(), &self.api_endpoint);
        
        self.send_transcription_request(audio_bytes).await
    }

    async fn send_transcription_request(&self, audio_bytes: Vec<u8>) -> Result<String, String> {
        // Send request to Whisper API with retries
        let url = format!("{}/audio/transcriptions", self.api_endpoint);
        DebugLogger::log_info(&format!("STT: Preparing request to URL: {}", url));
        DebugLogger::log_info(&format!("STT: Audio payload size: {} bytes", audio_bytes.len()));
        
        for attempt in 1..=3 {
            DebugLogger::log_info(&format!("STT attempt {}/3 to {}", attempt, url));
            
            // Create multipart form data fresh for each attempt
            DebugLogger::log_info("STT: Creating multipart form data");
            let form = reqwest::multipart::Form::new()
                .text("model", self.model.clone())
                .text("response_format", "json")
                .text("language", "en") // Optional: specify language if known
                .part("file", reqwest::multipart::Part::bytes(audio_bytes.clone())
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| {
                        let error_msg = format!("Multipart error: {}", e);
                        DebugLogger::log_pipeline_error("stt", &error_msg);
                        error_msg
                    })?);
            DebugLogger::log_info("STT: Multipart form created successfully");

            DebugLogger::log_info("STT: Sending HTTP POST request");
            let response = self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .multipart(form)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    let status = resp.status();
                    DebugLogger::log_info(&format!("STT API response status: {}", status));
                    DebugLogger::log_info(&format!("STT API response headers: {:?}", resp.headers()));
                    
                    if resp.status().is_success() {
                        DebugLogger::log_info("STT: Response is successful, reading response text");
                        let response_text = resp.text().await
                            .map_err(|e| {
                                let error_msg = format!("Failed to read response text: {}", e);
                                DebugLogger::log_pipeline_error("stt", &error_msg);
                                error_msg
                            })?;
                        
                        DebugLogger::log_info(&format!("STT API raw response: {}", response_text));
                        
                        DebugLogger::log_info("STT: Parsing JSON response");
                        let json: Value = serde_json::from_str(&response_text)
                            .map_err(|e| {
                                let error_msg = format!("JSON parsing error: {}", e);
                                DebugLogger::log_pipeline_error("stt", &error_msg);
                                error_msg
                            })?;
                        
                        DebugLogger::log_info(&format!("STT: Parsed JSON: {}", serde_json::to_string_pretty(&json).unwrap_or_default()));
                        
                        if let Some(text) = json["text"].as_str() {
                            DebugLogger::log_info(&format!("STT extracted text: '{}'", text));
                            return Ok(text.trim().to_string());
                        } else {
                            let error_msg = "No text in API response".to_string();
                            DebugLogger::log_pipeline_error("stt", &error_msg);
                            DebugLogger::log_info(&format!("STT: Available JSON keys: {:?}", json.as_object().map(|o| o.keys().collect::<Vec<_>>())));
                            return Err(error_msg);
                        }
                    } else {
                        DebugLogger::log_info("STT: Response status is not successful, reading error response");
                        let error_text = resp.text().await.unwrap_or_default();
                        DebugLogger::log_info(&format!("STT API error response: {}", error_text));
                        
                        // Don't retry on authentication errors
                        if status.as_u16() == 401 || status.as_u16() == 403 {
                            let error_msg = format!("Authentication error: {}", error_text);
                            DebugLogger::log_pipeline_error("stt", &error_msg);
                            return Err(error_msg);
                        }
                        
                        if attempt == 3 {
                            let error_msg = format!("API error after {} attempts: {} - {}", attempt, status, error_text);
                            DebugLogger::log_pipeline_error("stt", &error_msg);
                            return Err(error_msg);
                        }
                        
                        // Wait before retry
                        let delay = Duration::from_millis(1000 * attempt);
                        DebugLogger::log_info(&format!("Retrying in {}ms...", delay.as_millis()));
                        tokio::time::sleep(delay).await;
                    }
                },
                Err(e) => {
                    DebugLogger::log_info(&format!("STT network error: {}", e));
                    
                    if attempt == 3 {
                        let error_msg = format!("Network error after {} attempts: {}", attempt, e);
                        DebugLogger::log_pipeline_error("stt", &error_msg);
                        return Err(error_msg);
                    }
                    
                    // Wait before retry
                    let delay = Duration::from_millis(1000 * attempt);
                    DebugLogger::log_info(&format!("Retrying in {}ms...", delay.as_millis()));
                    tokio::time::sleep(delay).await;
                }
            }
        }
        
        let error_msg = "Max retries exceeded".to_string();
        DebugLogger::log_pipeline_error("stt", &error_msg);
        Err(error_msg)
    }

    fn encode_wav(&self, samples: &[f32], sample_rate: u32) -> Result<Vec<u8>, String> {
        // Convert f32 samples to i16
        let mut audio_data = Vec::with_capacity(samples.len() * 2);
        for &sample in samples {
            let sample_i16 = (sample * i16::MAX as f32) as i16;
            audio_data.extend_from_slice(&sample_i16.to_le_bytes());
        }

        // Create WAV header
        let mut wav_data = Vec::new();
        
        // RIFF header
        wav_data.extend_from_slice(b"RIFF");
        wav_data.extend_from_slice(&(36 + audio_data.len() as u32).to_le_bytes()); // File size
        wav_data.extend_from_slice(b"WAVE");
        
        // Format chunk
        wav_data.extend_from_slice(b"fmt ");
        wav_data.extend_from_slice(&16u32.to_le_bytes()); // Chunk size
        wav_data.extend_from_slice(&1u16.to_le_bytes()); // Audio format (PCM)
        wav_data.extend_from_slice(&1u16.to_le_bytes()); // Number of channels
        wav_data.extend_from_slice(&sample_rate.to_le_bytes()); // Sample rate
        wav_data.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // Byte rate
        wav_data.extend_from_slice(&2u16.to_le_bytes()); // Block align
        wav_data.extend_from_slice(&16u16.to_le_bytes()); // Bits per sample
        
        // Data chunk
        wav_data.extend_from_slice(b"data");
        wav_data.extend_from_slice(&(audio_data.len() as u32).to_le_bytes()); // Data size
        wav_data.extend_from_slice(&audio_data);
        
        Ok(wav_data)
    }
}