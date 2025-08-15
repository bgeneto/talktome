use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct AudioCapture {
    stream: Option<cpal::Stream>,
    tx: Option<mpsc::Sender<Vec<f32>>>,
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            tx: None,
        }
    }

    pub fn start_capture(&mut self) -> Result<mpsc::Receiver<Vec<f32>>, Box<dyn std::error::Error + Send + Sync>> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No input device available")?;
        
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        
        // Create a channel for sending audio chunks
        let (tx, rx) = mpsc::channel(100);
        
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
        tx: mpsc::Sender<Vec<f32>>,
        sample_rate: u32,
    ) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>>
    where
        T: Sample + cpal::SizedSample + Send + 'static,
        f32: FromSample<T>,
    {
        let channels = config.channels as usize;
        let chunk_size = sample_rate as usize * 2; // 2 seconds of audio
        let chunk_buffer = Arc::new(Mutex::new(Vec::<f32>::with_capacity(chunk_size)));
        
        let stream = device.build_input_stream(
            config,
            {
                let tx = tx.clone();
                let chunk_buffer = chunk_buffer.clone();
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    // Convert samples to f32 and take only first channel (mono)
                    let samples: Vec<f32> = data.chunks(channels)
                        .map(|chunk| chunk[0].to_sample())
                        .collect();
                    
                    // Add to chunk buffer
                    let mut should_send = false;
                    let mut chunk_to_send = Vec::new();
                    
                    if let Ok(mut buffer) = chunk_buffer.lock() {
                        buffer.extend_from_slice(&samples);
                        
                        // When we have enough audio data, send it
                        if buffer.len() >= chunk_size {
                            chunk_to_send = buffer.clone();
                            buffer.clear();
                            should_send = true;
                        }
                    }
                    
                    if should_send && !chunk_to_send.is_empty() {
                        // Send chunk in a blocking way since we're in a callback
                        let tx_clone = tx.clone();
                        std::thread::spawn(move || {
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async {
                                let _ = tx_clone.send(chunk_to_send).await;
                            });
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