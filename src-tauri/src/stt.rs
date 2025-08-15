use reqwest;
use serde_json::Value;

pub struct STTService {
    client: reqwest::Client,
    api_endpoint: String,
    api_key: String,
}

impl STTService {
    pub fn new(api_endpoint: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_endpoint,
            api_key,
        }
    }

    pub async fn transcribe_chunk(&self, audio_data: Vec<f32>, sample_rate: u32) -> Result<String, String> {
        // Convert f32 samples to i16 for WAV encoding
        let audio_bytes = self.encode_wav(&audio_data, sample_rate).map_err(|e| e.to_string())?;
        
        // Create multipart form data
        let form = reqwest::multipart::Form::new()
            .text("model", "whisper-1")
            .text("response_format", "json")
            .part("file", reqwest::multipart::Part::bytes(audio_bytes)
                .file_name("audio.wav")
                .mime_str("audio/wav").map_err(|e| e.to_string())?);

        // Send request to Whisper API
        let url = format!("{}/audio/transcriptions", self.api_endpoint);
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            let json: Value = response.json().await.map_err(|e| e.to_string())?;
            if let Some(text) = json["text"].as_str() {
                Ok(text.to_string())
            } else {
                Err("No text in response".to_string())
            }
        } else {
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            Err(format!("API error: {}", error_text))
        }
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