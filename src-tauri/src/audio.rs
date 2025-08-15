use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct AudioCapture {
    stream: Option<cpal::Stream>,
    tx: Option<mpsc::Sender<AudioChunk>>,
    is_recording: Arc<Mutex<bool>>,
}

#[derive(Clone)]
pub struct AudioChunk {
    pub data: Vec<f32>,
    pub sample_rate: u32,
}

impl AudioChunk {
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Check if audio chunk has sufficient volume to process
    pub fn has_audio_activity(&self) -> bool {
        let max_amplitude = self.data.iter().map(|&x| x.abs()).fold(0.0, f32::max);
        max_amplitude > 0.01
    }
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            tx: None,
            is_recording: Arc::new(Mutex::new(false)),
        }
    }

    pub fn start_capture(&mut self) -> Result<mpsc::Receiver<AudioChunk>, Box<dyn std::error::Error + Send + Sync>> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No input device available")?;
        
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        
        // Create a channel for sending audio chunks
        let (tx, rx) = mpsc::channel(50); // Increased buffer size
        
        // Set recording state to true
        *self.is_recording.lock().unwrap() = true;
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => self.build_input_stream::<f32>(&device, &config.into(), tx.clone(), sample_rate)?,
            cpal::SampleFormat::I16 => self.build_input_stream::<i16>(&device, &config.into(), tx.clone(), sample_rate)?,
            cpal::SampleFormat::U16 => self.build_input_stream::<u16>(&device, &config.into(), tx.clone(), sample_rate)?,
            _ => return Err("Unsupported sample format".into()),
        };
        
        stream.play()?;
        self.stream = Some(stream);
        self.tx = Some(tx);
        
        Ok(rx)
    }
    fn build_input_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        tx: mpsc::Sender<AudioChunk>,
        sample_rate: u32,
    ) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>>
    where
        T: Sample + cpal::SizedSample + Send + 'static,
        f32: FromSample<T>,
    {
        let channels = config.channels as usize;
        let chunk_duration_ms = 2000; // 2 seconds per chunk
        let chunk_size = (sample_rate as f32 * chunk_duration_ms as f32 / 1000.0) as usize;
        let chunk_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(chunk_size * 2)));
        let is_recording = self.is_recording.clone();
        
        let stream = device.build_input_stream(
            config,
            {
                let tx = tx.clone();
                let chunk_buffer = chunk_buffer.clone();
                let is_recording = is_recording.clone();
                
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    // Only process if we're recording
                    if !*is_recording.lock().unwrap() {
                        return;
                    }

                    // Convert samples to f32 and take only first channel (mono)
                    let samples: Vec<f32> = data.chunks(channels)
                        .map(|chunk| chunk[0].to_sample())
                        .collect();
                    
                    // Add to chunk buffer
                    let mut should_send = false;
                    let mut chunk_to_send = AudioChunk {
                        data: Vec::new(),
                        sample_rate,
                    };
                    
                    if let Ok(mut buffer) = chunk_buffer.lock() {
                        buffer.extend_from_slice(&samples);
                        
                        // When we have enough audio data, send it
                        if buffer.len() >= chunk_size {
                            chunk_to_send.data = buffer.clone();
                            buffer.clear();
                            should_send = true;
                        }
                    }
                    
                    if should_send && !chunk_to_send.data.is_empty() {
                        // Send chunk asynchronously
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            if let Err(e) = tx_clone.send(chunk_to_send).await {
                                eprintln!("Failed to send audio chunk: {}", e);
                            }
                        });
                    }
                }
            },
            move |err| {
                eprintln!("Audio input error: {}", err);
            },
            None,
        )?;
        
        Ok(stream)
    }
}