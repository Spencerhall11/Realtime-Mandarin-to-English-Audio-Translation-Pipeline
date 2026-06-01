use std::thread;
use std::time::Duration;
use crossbeam_channel::Sender;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct AudioRecorder {
    target_sample_rate: u32,
}

impl AudioRecorder {
    pub fn new(sample_rate: u32) -> Self {
        Self { target_sample_rate: sample_rate }
    }

    /// Spawns a dedicated background thread to ingest raw microphone data via hardware interrupts
    pub fn spawn_capture_loop(&self, tx: Sender<f32>) -> Result<(), Box<dyn std::error::Error>> {
        let target_rate = self.target_sample_rate;
        
        thread::spawn(move || {
            let host = cpal::default_host();
            let device = host.default_input_device()
                .expect("No default audio input hardware device detected");

            //Query configuration the audio driver actually demands
            let supported_config = device.default_input_config()
                .expect("Failed to query default microphone audio configuration");

            let native_sample_rate = supported_config.sample_rate().0;
            let native_channels = supported_config.channels();

            // Match the hardware rules perfectly to bypass StreamConfigNotSupported panics
            let config = cpal::StreamConfig {
                channels: native_channels,
                sample_rate: supported_config.sample_rate(),
                buffer_size: cpal::BufferSize::Default,
            };

            println!(
                "Microphone hardware initialized natively at {}Hz, {} Channel(s).",
                native_sample_rate, native_channels
            );
            
            if native_sample_rate != target_rate {
                println!("[DSP] Linear downsampler activated: Resampling {}Hz -> {}Hz.", native_sample_rate, target_rate);
            }

            let stream_tx = tx;
            
            // Calculate the fractional index step size for downsampling
            let step = native_sample_rate as f32 / target_rate as f32;

            let audio_stream = device.build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // 1. Demux channels if hardware is stereo, stripping it down to Mono (Left Channel)
                    let mut channel_filtered = Vec::with_capacity(data.len() / native_channels as usize);
                    for chunk in data.chunks(native_channels as usize) {
                        if !chunk.is_empty() {
                            channel_filtered.push(chunk[0]); 
                        }
                    }

                    // Real-time Digital Signal Processing (DSP) 
                    if native_sample_rate == target_rate {
                        // Hardware matches 16kHz natively; push raw samples immediately
                        for sample in channel_filtered {
                            let _ = stream_tx.send(sample);
                        }
                    } else {
                        // Hardware is running at 44.1kHz or 48kHz. Map it
                        let mut idx = 0.0;
                        while (idx as usize) < channel_filtered.len() {
                            let base_idx = idx as usize;
                            let frac = idx - base_idx as f32;
                            
                            let sample = if base_idx + 1 < channel_filtered.len() {
                                // Linear blend between the two closest sample boundaries
                                (1.0 - frac) * channel_filtered[base_idx] + frac * channel_filtered[base_idx + 1]
                            } else {
                                channel_filtered[base_idx]
                            };

                            let _ = stream_tx.send(sample);
                            idx += step;
                        }
                    }
                },
                |err| {
                    eprintln!("Audio input interface thread exception context: {}", err);
                },
                None
            ).expect("Failed to initialize system audio stream handler");

            audio_stream.play().expect("Failed to run audio stream callback pipelines");

            // Keep the background thread alive and retain ownership of audio_stream
            loop {
                thread::sleep(Duration::from_millis(100));
            }
        });

        Ok(())
    }
}