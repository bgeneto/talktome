// Simplified audio recording for TalkToMe with noise reduction
// This module handles basic audio recording - start/stop only, with nnnoiseless filtering
use crate::debug_logger::DebugLogger;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
use nnnoiseless::DenoiseState;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};



/// Simple downsampling function using decimation
fn downsample_audio(input: &[f32], input_rate: u32, target_rate: u32) -> Vec<f32> {
    if input_rate == target_rate {
        return input.to_vec();
    }
    
    let ratio = input_rate as f32 / target_rate as f32;
    let output_len = (input.len() as f32 / ratio) as usize;
    let mut output = Vec::with_capacity(output_len);
    
    for i in 0..output_len {
        let src_index = (i as f32 * ratio) as usize;
        if src_index < input.len() {
            output.push(input[src_index]);
        }
    }
    
    output
}

/// Noise reduction processor using nnnoiseless
pub struct NoiseReducer {
    denoise_state: DenoiseState<'static>,
    frame_buffer: Vec<f32>,
    sample_rate: u32,
}

impl NoiseReducer {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            denoise_state: *DenoiseState::new(),
            frame_buffer: Vec::new(),
            sample_rate,
        }
    }

    /// Process audio samples through nnnoiseless to reduce background noise
    /// Downsamples to 16kHz for optimal noise reduction and keeps it at 16kHz for efficiency
    pub fn process_audio(&mut self, input: &[f32]) -> Vec<f32> {
        use crate::debug_logger::DebugLogger;

        const TARGET_SAMPLE_RATE: u32 = 16000;
        const NNNOISE_FRAME_SIZE: usize = 480; // 30ms at 16kHz

        DebugLogger::log_info(&format!(
            "NOISE_REDUCER: Processing {} input samples at {}Hz, downsampling to {}Hz",
            input.len(),
            self.sample_rate,
            TARGET_SAMPLE_RATE
        ));

        // First, downsample the input to 16kHz if needed
        let downsampled_input = if self.sample_rate != TARGET_SAMPLE_RATE {
            let target_length =
                (input.len() as f32 * TARGET_SAMPLE_RATE as f32 / self.sample_rate as f32) as usize;
            let mut downsampled = Vec::with_capacity(target_length);

            // Simple decimation - take every nth sample
            let step = self.sample_rate as f32 / TARGET_SAMPLE_RATE as f32;
            for i in 0..target_length {
                let src_index = (i as f32 * step) as usize;
                if src_index < input.len() {
                    downsampled.push(input[src_index]);
                } else {
                    downsampled.push(0.0);
                }
            }

            DebugLogger::log_info(&format!(
                "NOISE_REDUCER: Downsampled from {} samples at {}Hz to {} samples at {}Hz",
                input.len(),
                self.sample_rate,
                downsampled.len(),
                TARGET_SAMPLE_RATE
            ));
            downsampled
        } else {
            input.to_vec()
        };

        // Add downsampled samples to the frame buffer
        self.frame_buffer.extend_from_slice(&downsampled_input);

        let mut output = Vec::new();
        let mut frames_processed = 0;

        // Process complete frames at 16kHz
        while self.frame_buffer.len() >= NNNOISE_FRAME_SIZE {
            // Take one frame from the buffer
            let frame: Vec<f32> = self.frame_buffer.drain(0..NNNOISE_FRAME_SIZE).collect();

            // Calculate input frame statistics
            let _input_max = frame.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
            let _input_rms = (frame.iter().map(|x| x * x).sum::<f32>() / frame.len() as f32).sqrt();

            // Apply noise reduction directly on 16kHz audio
            let mut out_frame = vec![0.0f32; NNNOISE_FRAME_SIZE];
            self.denoise_state
                .process_frame(&mut out_frame[..], &frame[..]);

            // Calculate output frame statistics
            let _output_max = out_frame.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
            let _output_rms =
                (out_frame.iter().map(|x| x * x).sum::<f32>() / out_frame.len() as f32).sqrt();

            output.extend_from_slice(&out_frame);
            frames_processed += 1;
        }

        DebugLogger::log_info(&format!(
            "NOISE_REDUCER: Processed {} frames at 16kHz, {} samples remaining in buffer, returning {} samples at 16kHz",
            frames_processed,
            self.frame_buffer.len(),
            output.len()
        ));

        output
    }

    /// Get any remaining samples in the buffer (useful for final processing)
    /// Get any remaining samples in the buffer (useful for final processing)
    /// Returns samples at 16kHz
    pub fn flush(&mut self) -> Vec<f32> {
        if self.frame_buffer.is_empty() {
            return Vec::new();
        }

        const NNNOISE_FRAME_SIZE: usize = 480; // 30ms at 16kHz

        // Pad the remaining buffer to complete frame size with zeros
        while self.frame_buffer.len() < NNNOISE_FRAME_SIZE {
            self.frame_buffer.push(0.0);
        }

        let frame = self.frame_buffer.clone();
        self.frame_buffer.clear();

        // Process the final frame at 16kHz
        let mut out_frame = vec![0.0f32; NNNOISE_FRAME_SIZE];
        self.denoise_state
            .process_frame(&mut out_frame[..], &frame[..]);
        out_frame
    }
}

pub struct AudioCapture {
    stream: Option<cpal::Stream>,
    is_recording: Arc<Mutex<bool>>,
    audio_buffer: Arc<Mutex<Vec<f32>>>,
    sample_rate: Arc<Mutex<u32>>,
    noise_reducer: Arc<Mutex<Option<NoiseReducer>>>,
}

/// Simple audio chunk containing raw audio data
#[derive(Clone)]
pub struct AudioChunk {
    pub data: Vec<f32>,
    pub sample_rate: u32,
}

impl AudioChunk {
    pub fn new(data: Vec<f32>, sample_rate: u32) -> Self {
        Self { data, sample_rate }
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Check if audio chunk has sufficient volume to process
    pub fn has_audio_activity(&self) -> bool {
        // Simple volume check - consider it active if any sample is above threshold
        let threshold = 0.01; // Adjust as needed
        self.data.iter().any(|&sample| sample.abs() > threshold)
    }
}
impl AudioCapture {
    pub fn new() -> Self {
        Self {
            stream: None,
            is_recording: Arc::new(Mutex::new(false)),
            audio_buffer: Arc::new(Mutex::new(Vec::new())),
            sample_rate: Arc::new(Mutex::new(16000)), // Default sample rate
            noise_reducer: Arc::new(Mutex::new(None)),
        }
    }

    /// Start recording audio from the default microphone
    pub fn start_capture(
        &mut self,
        _audio_chunking_enabled: bool,
    ) -> Result<mpsc::Receiver<AudioChunk>, Box<dyn std::error::Error + Send + Sync>> {
        DebugLogger::log_info("AudioCapture::start_capture() called");

        // Check if already recording
        {
            let recording = self.is_recording.lock().unwrap();
            if *recording {
                return Err("Already recording".into());
            }
        }

        let host = cpal::default_host();
        DebugLogger::log_info(&format!("Audio host: {:?}", host.id()));

        let device = host
            .default_input_device()
            .ok_or("No input device available")?;
        DebugLogger::log_info(&format!(
            "Input device: {:?}",
            device.name().unwrap_or_default()
        ));

        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0;
        DebugLogger::log_info(&format!(
            "Audio config: sample_rate={}Hz, channels={}, format={:?}",
            sample_rate,
            config.channels(),
            config.sample_format()
        ));

        // Store sample rate
        {
            let mut sr = self.sample_rate.lock().unwrap();
            *sr = sample_rate;
        }

        // Initialize noise reducer
        {
            let mut noise_reducer = self.noise_reducer.lock().unwrap();
            *noise_reducer = Some(NoiseReducer::new(sample_rate));
            DebugLogger::log_info(&format!(
                "Noise reducer initialized for {}Hz (nnnoiseless works best at 16kHz)",
                sample_rate
            ));

            // Warn if sample rate is not optimal for nnnoiseless
            if sample_rate != 16000 {
                DebugLogger::log_info(&format!(
                    "WARNING: Sample rate is {}Hz, but nnnoiseless is optimized for 16kHz. Noise reduction may be less effective.",
                    sample_rate
                ));
            }
        }

        // Clear audio buffer
        {
            let mut buffer = self.audio_buffer.lock().unwrap();
            buffer.clear();
        }

        // Create a channel for sending the final audio chunk when recording stops
        let (tx, rx) = mpsc::channel();

        // Set recording state to true
        {
            let mut recording = self.is_recording.lock().unwrap();
            *recording = true;
        }
        DebugLogger::log_info("Audio recording state set to true");

        // Build the audio stream
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                DebugLogger::log_info("Building F32 input stream");
                self.build_input_stream::<f32>(&device, &config.into(), sample_rate)?
            }
            cpal::SampleFormat::I16 => {
                DebugLogger::log_info("Building I16 input stream");
                self.build_input_stream::<i16>(&device, &config.into(), sample_rate)?
            }
            cpal::SampleFormat::U16 => {
                DebugLogger::log_info("Building U16 input stream");
                self.build_input_stream::<u16>(&device, &config.into(), sample_rate)?
            }
            _ => return Err("Unsupported sample format".into()),
        };

        DebugLogger::log_info("Starting audio stream");
        stream.play()?;
        self.stream = Some(stream);

        // Spawn a thread to monitor for stop and send the final audio chunk
        let audio_buffer = self.audio_buffer.clone();
        let is_recording = self.is_recording.clone();
        let sample_rate_arc = self.sample_rate.clone();
        let noise_reducer_arc = self.noise_reducer.clone();

        std::thread::spawn(move || {
            // Wait for recording to stop
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let recording = is_recording.lock().unwrap();
                if !*recording {
                    break;
                }
            }

            // Get the final audio data
            let final_audio = {
                let buffer = audio_buffer.lock().unwrap();
                buffer.clone()
            };

            let sr = {
                let sample_rate = sample_rate_arc.lock().unwrap();
                *sample_rate
            };

            if !final_audio.is_empty() {
                DebugLogger::log_info(&format!(
                    "Processing {} samples through noise reduction",
                    final_audio.len()
                ));

                // Save original audio to WAV if debug is enabled
                if DebugLogger::is_debug_enabled() {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    DebugLogger::save_wav_dump(&format!("original_{}", timestamp), &final_audio);
                    DebugLogger::log_info(&format!(
                        "Saved original audio recording: {} samples",
                        final_audio.len()
                    ));
                }

                // Apply noise reduction to the final audio with downsampling
                let processed_audio = {
                    let mut noise_reducer_guard = noise_reducer_arc.lock().unwrap();
                    if let Some(ref mut noise_reducer) = noise_reducer_guard.as_mut() {
                        DebugLogger::log_info("NOISE_REDUCTION: Applying noise reduction filter");
                        let mut processed = noise_reducer.process_audio(&final_audio);
                        // Flush any remaining samples
                        let remaining = noise_reducer.flush();
                        processed.extend_from_slice(&remaining);
                        processed
                    } else {
                        // If no noise reducer, just downsample
                        DebugLogger::log_info("NOISE_REDUCTION: No noise reducer available, downsampling only");
                        downsample_audio(&final_audio, sr, 16000)
                    }
                };

                // Log comparison for debugging
                let original_samples = final_audio.len();
                let processed_samples = processed_audio.len();
                let original_max = final_audio.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
                let processed_max = processed_audio.iter().map(|x| x.abs()).fold(0.0f32, f32::max);

                DebugLogger::log_info(&format!(
                    "NOISE_REDUCTION: Original {} samples (max amplitude: {:.6}), Processed {} samples (max amplitude: {:.6})",
                    original_samples, original_max, processed_samples, processed_max
                ));

                // Check if they're actually different
                let are_identical = original_samples == processed_samples
                    && final_audio
                        .iter()
                        .zip(processed_audio.iter())
                        .all(|(a, b)| (a - b).abs() < 1e-10);

                DebugLogger::log_info(&format!(
                    "NOISE_REDUCTION: Arrays are identical: {}",
                    are_identical
                ));

                // Save noise-reduced audio to WAV if debug is enabled
                if DebugLogger::is_debug_enabled() {
                    let timestamp = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    DebugLogger::save_wav_dump(&format!("noiseless_{}", timestamp), &processed_audio);
                    DebugLogger::log_info(&format!(
                        "Saved noise-reduced audio recording: {} samples at 16kHz",
                        processed_audio.len()
                    ));
                }

                DebugLogger::log_info(&format!(
                    "Sending noise-reduced audio chunk: {} samples, 16000Hz (downsampled from {}Hz)",
                    processed_audio.len(),
                    sr
                ));
                
                // Check if the main pipeline is still expecting chunks
                // (This is a best-effort check - the send could still fail due to race conditions)
                let chunk = AudioChunk::new(processed_audio, 16000); // Output is always 16kHz after noise reduction
                let send_result = tx.send(chunk);
                if send_result.is_ok() {
                    DebugLogger::log_info("AUDIO_CHUNK_SENT: Successfully sent processed audio chunk to main pipeline");
                } else {
                    // This is expected during shutdown - the main pipeline may have closed the receiver
                    DebugLogger::log_info("AUDIO_CHUNK_SEND_EXPECTED: Main pipeline receiver closed during shutdown (this is normal)");
                }
            } else {
                DebugLogger::log_info("No audio data recorded");
            }
        });

        DebugLogger::log_info("Audio capture started successfully");
        Ok(rx)
    }

    /// Stop recording and clean up
    pub fn stop_recording(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        DebugLogger::log_info("AudioCapture::stop_recording() called");

        // Set recording flag to false
        {
            let mut recording = self.is_recording.lock().unwrap();
            *recording = false;
        }
        DebugLogger::log_info("Recording state set to false");

        // Stop and drop the stream
        if let Some(stream) = self.stream.take() {
            drop(stream);
            DebugLogger::log_info("Audio stream stopped and dropped");
        }

        // Note: We don't clean up the noise reducer here because the background thread
        // might still be processing the final audio chunk. The noise reducer will be
        // replaced when start_capture() is called again.

        Ok(())
    }

    fn build_input_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        sample_rate: u32,
    ) -> Result<cpal::Stream, Box<dyn std::error::Error + Send + Sync>>
    where
        T: Sample + cpal::SizedSample + Send + 'static,
        f32: FromSample<T>,
    {
        let channels = config.channels as usize;
        DebugLogger::log_info(&format!(
            "Audio stream config: channels={}, sample_rate={}Hz",
            channels, sample_rate
        ));

        let is_recording = self.is_recording.clone();
        let audio_buffer = self.audio_buffer.clone();

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                // Only process if we're recording
                if !*is_recording.lock().unwrap() {
                    return;
                }

                // Convert samples to f32 and take only first channel (mono)
                let samples: Vec<f32> = data
                    .chunks(channels)
                    .map(|chunk| chunk[0].to_sample())
                    .collect();

                // Append to buffer
                {
                    let mut buffer = audio_buffer.lock().unwrap();
                    buffer.extend_from_slice(&samples);
                }
            },
            move |err| {
                eprintln!("Audio input error: {}", err);
                DebugLogger::log_info(&format!("Audio input error: {}", err));
            },
            None,
        )?;

        Ok(stream)
    }
}
