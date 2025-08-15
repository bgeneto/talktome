use reqwest;
use serde_json::Value;
use std::time::Duration;

pub struct STTService {
    client: reqwest::Client,
    api_endpoint: String,
    api_key: String,
}

impl STTService {
    pub fn new(api_endpoint: String, api_key: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            api_endpoint,
            api_key,
        }
    }

    /// Transcribe audio chunk with enhanced error handling
    pub async fn transcribe_chunk(&self, audio_data: Vec<f32>, sample_rate: u32) -> Result<String, String> {
        if audio_data.is_empty() {
            return Err("Empty audio data".to_string());
        }

        // Check for audio quality - skip if too quiet
        let max_amplitude = audio_data.iter().map(|&x| x.abs()).fold(0.0, f32::max);
        if max_amplitude < 0.01 {
            return Ok(String::new()); // Return empty string for silent audio
        }

        // Convert f32 samples to i16 for WAV encoding
        let audio_bytes = self.encode_wav(&audio_data, sample_rate)
            .map_err(|e| format!("Audio encoding error: {}", e))?;

        // Skip very small audio files (less than 1 second)
        if audio_bytes.len() < (sample_rate * 2) as usize {
            return Ok(String::new());
        }

        self.send_transcription_request(audio_bytes).await
    }

    async fn send_transcription_request(&self, audio_bytes: Vec<u8>) -> Result<String, String> {
        // Send request to Whisper API with retries
        let url = format!("{}/audio/transcriptions", self.api_endpoint);
        
        for attempt in 1..=3 {
            // Create multipart form data fresh for each attempt
            let form = reqwest::multipart::Form::new()
                .text("model", "whisper-1")
                .text("response_format", "json")
                .text("language", "en") // Optional: specify language if known
                .part("file", reqwest::multipart::Part::bytes(audio_bytes.clone())
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| format!("Multipart error: {}", e))?);

            let response = self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .multipart(form)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let json: Value = resp.json().await
                            .map_err(|e| format!("JSON parsing error: {}", e))?;
                        
                        if let Some(text) = json["text"].as_str() {
                            return Ok(text.trim().to_string());
                        } else {
                            return Err("No text in API response".to_string());
                        }
                    } else {
                        let status = resp.status();
                        let error_text = resp.text().await.unwrap_or_default();
                        
                        // Don't retry on authentication errors
                        if status.as_u16() == 401 || status.as_u16() == 403 {
                            return Err(format!("Authentication error: {}", error_text));
                        }
                        
                        if attempt == 3 {
                            return Err(format!("API error after {} attempts: {} - {}", attempt, status, error_text));
                        }
                        
                        // Wait before retry
                        tokio::time::sleep(Duration::from_millis(1000 * attempt)).await;
                    }
                },
                Err(e) => {
                    if attempt == 3 {
                        return Err(format!("Network error after {} attempts: {}", attempt, e));
                    }
                    
                    // Wait before retry
                    tokio::time::sleep(Duration::from_millis(1000 * attempt)).await;
                }
            }
        }
        
        Err("Max retries exceeded".to_string())
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