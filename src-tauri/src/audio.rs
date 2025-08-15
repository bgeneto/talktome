use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use crate::debug_logger::DebugLogger;

pub struct AudioCapture {
    stream: Option<cpal::Stream>,
    tx: Option<mpsc::Sender<AudioChunk>>,
    is_recording: Arc<Mutex<bool>>,
    vad: Arc<Mutex<VoiceActivityDetector>>,
}

/// Voice Activity Detection configuration and state
pub struct VoiceActivityDetector {
    pub speech_threshold: f32,     // Energy threshold for speech detection
    pub silence_threshold: f32,    // Energy threshold for silence
    pub min_speech_duration_ms: u64,  // Minimum duration for speech chunk
    pub max_speech_duration_ms: u64,  // Maximum duration for speech chunk (0.5-1s for real-time)
    pub silence_timeout_ms: u64,   // Time to wait in silence before ending chunk
    pub overlap_ms: u64,           // Overlap to prevent word cutting
    
    // Internal state
    current_state: VadState,
    state_start_time: std::time::Instant,
    current_chunk: Vec<f32>,
    overlap_buffer: Vec<f32>,      // Buffer for overlap handling
    sample_rate: u32,
}

#[derive(Debug, Clone, PartialEq)]
enum VadState {
    Silence,
    Speech,
    SilenceAfterSpeech,
}

impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self {
            speech_threshold: 0.001,        // Lower threshold for more sensitivity
            silence_threshold: 0.0005,      // Lower silence threshold
            min_speech_duration_ms: 100,    // Shorter minimum for testing
            max_speech_duration_ms: 1000,   // 1 second maximum for real-time (was 30s)
            silence_timeout_ms: 300,        // 300ms timeout for responsiveness (was 1s)
            overlap_ms: 150,                // 150ms overlap to prevent word cutting
            
            current_state: VadState::Silence,
            state_start_time: std::time::Instant::now(),
            current_chunk: Vec::new(),
            overlap_buffer: Vec::new(),
            sample_rate: 16000,             // 16kHz instead of 48kHz (better for speech)
        }
    }
}

impl VoiceActivityDetector {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate,
            ..Default::default()
        }
    }
    
    /// Process audio samples and return completed chunks
    pub fn process_audio(&mut self, samples: &[f32]) -> Vec<AudioChunk> {
        let mut completed_chunks = Vec::new();
        
        // Add samples to current chunk
        self.current_chunk.extend_from_slice(samples);
        
        // Calculate energy for current samples
        let energy = self.calculate_energy(samples);
        
        // Determine if current samples contain speech
        let has_speech = energy > self.speech_threshold;
        let is_silence = energy < self.silence_threshold;
        
        let now = std::time::Instant::now();
        let state_duration = now.duration_since(self.state_start_time);
        
        match self.current_state {
            VadState::Silence => {
                if has_speech {
                    // Transition to speech - include overlap from previous chunk
                    if !self.overlap_buffer.is_empty() {
                        let mut chunk_with_overlap = self.overlap_buffer.clone();
                        chunk_with_overlap.extend_from_slice(&self.current_chunk);
                        self.current_chunk = chunk_with_overlap;
                        self.overlap_buffer.clear();
                    }
                    self.current_state = VadState::Speech;
                    self.state_start_time = now;
                }
                // In silence, keep accumulating but create chunks less frequently
                if state_duration.as_millis() > 3000 && !self.current_chunk.is_empty() {
                    // Send silence chunk if we've been silent for 3 seconds
                    let chunk = AudioChunk::new(
                        self.current_chunk.clone(),
                        self.sample_rate,
                        ChunkType::SilenceChunk,
                    );
                    completed_chunks.push(chunk);
                    self.current_chunk.clear();
                }
            }
            
            VadState::Speech => {
                if is_silence {
                    // Transition to silence after speech
                    self.current_state = VadState::SilenceAfterSpeech;
                    self.state_start_time = now;
                } else if state_duration.as_millis() > self.max_speech_duration_ms as u128 {
                    // Force chunk completion if speech is too long (0.5-1s for real-time)
                    self.complete_speech_chunk(&mut completed_chunks);
                    self.current_state = VadState::Silence;
                    self.state_start_time = now;
                }
            }
            
            VadState::SilenceAfterSpeech => {
                if has_speech {
                    // Return to speech
                    self.current_state = VadState::Speech;
                    self.state_start_time = now;
                } else if state_duration.as_millis() > self.silence_timeout_ms as u128 {
                    // Complete speech chunk after silence timeout
                    if !self.current_chunk.is_empty() {
                        let total_duration = ((self.current_chunk.len() as f32 / self.sample_rate as f32) * 1000.0) as u64;
                        
                        if total_duration >= self.min_speech_duration_ms {
                            self.complete_speech_chunk(&mut completed_chunks);
                        }
                    }
                    
                    self.current_chunk.clear();
                    self.current_state = VadState::Silence;
                    self.state_start_time = now;
                }
            }
        }
        
        completed_chunks
    }
    
    /// Complete a speech chunk with overlap handling
    fn complete_speech_chunk(&mut self, completed_chunks: &mut Vec<AudioChunk>) {
        if self.current_chunk.is_empty() {
            return;
        }
        
        // Calculate overlap size in samples
        let overlap_samples = ((self.overlap_ms as f32 / 1000.0) * self.sample_rate as f32) as usize;
        
        // Create the completed chunk
        let chunk = AudioChunk::new(
            self.current_chunk.clone(),
            self.sample_rate,
            ChunkType::SpeechChunk,
        );
        completed_chunks.push(chunk);
        
        // Save overlap for next chunk (last N samples)
        if self.current_chunk.len() > overlap_samples {
            let start_idx = self.current_chunk.len() - overlap_samples;
            self.overlap_buffer = self.current_chunk[start_idx..].to_vec();
        } else {
            self.overlap_buffer = self.current_chunk.clone();
        }
        
        self.current_chunk.clear();
    }
    
    fn calculate_energy(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        
        // RMS energy calculation
        let sum_squares: f32 = samples.iter().map(|&x| x * x).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }
    
    /// Force completion of current chunk (useful when stopping recording)
    #[allow(dead_code)]
    pub fn flush(&mut self) -> Option<AudioChunk> {
        if self.current_chunk.is_empty() {
            return None;
        }
        
        let chunk_type = match self.current_state {
            VadState::Speech | VadState::SilenceAfterSpeech => ChunkType::SpeechChunk,
            VadState::Silence => ChunkType::SilenceChunk,
        };
        
        let chunk = AudioChunk::new(
            self.current_chunk.clone(),
            self.sample_rate,
            chunk_type,
        );
        
        self.current_chunk.clear();
        self.overlap_buffer.clear(); // Clear overlap on flush
        self.current_state = VadState::Silence;
        self.state_start_time = std::time::Instant::now();
        
        Some(chunk)
    }
}

#[derive(Clone)]
pub struct AudioChunk {
    pub data: Vec<f32>,
    pub sample_rate: u32,
    #[allow(dead_code)]
    pub timestamp: u64,
    pub duration_ms: u64,
    pub chunk_type: ChunkType,
}

#[derive(Debug, Clone)]
pub enum ChunkType {
    SpeechChunk,    // Contains speech activity
    SilenceChunk,   // Contains only silence
    #[allow(dead_code)]
    Mixed,          // Contains both speech and silence
}

impl AudioChunk {
    pub fn new(data: Vec<f32>, sample_rate: u32, chunk_type: ChunkType) -> Self {
        let duration_ms = ((data.len() as f32 / sample_rate as f32) * 1000.0) as u64;
        Self {
            data,
            sample_rate,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            duration_ms,
            chunk_type,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Check if audio chunk has sufficient volume to process
    pub fn has_audio_activity(&self) -> bool {
        match self.chunk_type {
            ChunkType::SpeechChunk | ChunkType::Mixed => true,
            ChunkType::SilenceChunk => false,
        }
    }
}

impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            tx: None,
            is_recording: Arc::new(Mutex::new(false)),
            vad: Arc::new(Mutex::new(VoiceActivityDetector::default())),
        }
    }

    #[allow(dead_code)]
    pub fn stop_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Set recording flag to false
        *self.is_recording.lock().unwrap() = false;
        
        // Flush any remaining audio from VAD
        if let Ok(mut vad_guard) = self.vad.lock() {
            if let Some(final_chunk) = vad_guard.flush() {
                if let Some(ref tx) = self.tx {
                    eprintln!("Sending final VAD chunk: {} samples, type: {:?}", 
                             final_chunk.data.len(), final_chunk.chunk_type);
                    let _ = tx.send(final_chunk);
                }
            }
        }
        
        // Stop and drop the stream
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
        
        self.tx = None;
        Ok(())
    }

    pub fn start_capture(&mut self) -> Result<mpsc::Receiver<AudioChunk>, Box<dyn std::error::Error + Send + Sync>> {
        DebugLogger::log_info("AudioCapture::start_capture() called");
        
        let host = cpal::default_host();
        DebugLogger::log_info(&format!("Audio host: {:?}", host.id()));
        
        let device = host.default_input_device().ok_or("No input device available")?;
        DebugLogger::log_info(&format!("Input device: {:?}", device.name().unwrap_or_default()));
        
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        DebugLogger::log_info(&format!("Audio config: sample_rate={}Hz, channels={}, format={:?}", 
            sample_rate, config.channels(), config.sample_format()));
        
        // Initialize VAD with correct sample rate
        if let Ok(mut vad_guard) = self.vad.lock() {
            *vad_guard = VoiceActivityDetector::new(sample_rate);
        }
        DebugLogger::log_info(&format!("VAD initialized with sample rate: {}Hz", sample_rate));
        
        // Create a channel for sending audio chunks
        let (tx, rx) = mpsc::channel(); // Synchronous unbounded channel
        DebugLogger::log_info("Audio channel created with buffer size 50");
        
        // Set recording state to true
        *self.is_recording.lock().unwrap() = true;
        DebugLogger::log_info("Audio recording state set to true");
        
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                DebugLogger::log_info("Building F32 input stream");
                self.build_input_stream::<f32>(&device, &config.into(), tx.clone(), sample_rate)?
            },
            cpal::SampleFormat::I16 => {
                DebugLogger::log_info("Building I16 input stream");
                self.build_input_stream::<i16>(&device, &config.into(), tx.clone(), sample_rate)?
            },
            cpal::SampleFormat::U16 => {
                DebugLogger::log_info("Building U16 input stream");
                self.build_input_stream::<u16>(&device, &config.into(), tx.clone(), sample_rate)?
            },
            _ => return Err("Unsupported sample format".into()),
        };
        
        DebugLogger::log_info("Starting audio stream playback");
        stream.play()?;
        self.stream = Some(stream);
        self.tx = Some(tx);
        
        DebugLogger::log_info("Audio capture started successfully");
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
        DebugLogger::log_info(&format!("Audio stream config: channels={}, sample_rate={}Hz", 
            channels, sample_rate));
        
        // Use the VAD from self
        let vad = self.vad.clone();
        let is_recording = self.is_recording.clone();
        
        let stream = device.build_input_stream(
            config,
            {
                let tx = tx.clone();
                let vad = vad.clone();
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
                    
                    // Log the first few audio callbacks to verify we're receiving data
                    use std::sync::atomic::{AtomicU32, Ordering};
                    static CALLBACK_COUNT: AtomicU32 = AtomicU32::new(0);
                    let count = CALLBACK_COUNT.fetch_add(1, Ordering::Relaxed);
                    if count < 5 {
                        eprintln!("Audio callback #{}: received {} samples", count + 1, samples.len());
                    }
                    
                    // Process audio through VAD
                    if let Ok(mut vad_guard) = vad.lock() {
                        let completed_chunks = vad_guard.process_audio(&samples);
                        
                        // Send any completed chunks
                        for chunk in completed_chunks {
                            eprintln!("VAD produced chunk: {} samples, type: {:?}, duration: {}ms", 
                                     chunk.data.len(), chunk.chunk_type, chunk.duration_ms);
                            
                            match tx.send(chunk) {
                                Ok(_) => {
                                    eprintln!("VAD chunk sent successfully");
                                },
                                Err(_) => {
                                    // Channel is closed, stop recording to prevent infinite error loop
                                    eprintln!("Channel closed, stopping audio recording");
                                    *is_recording.lock().unwrap() = false;
                                    return;
                                }
                            }
                        }
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