use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use ringbuf::traits::Producer;

pub struct AudioCapture {
    // We will keep the Stream alive here so it doesn't get destroyed when exiting the function
    _stream: Stream,
}

impl AudioCapture {
    /// Starts the Loopback capture (The general Windows audio)
    pub fn start_loopback(
        mut producer: impl ringbuf::traits::Producer<Item = f32> + Send + 'static,
    ) -> Result<Self, String> {
        // 1. Get the Audio Host (WASAPI in Windows)
        let host = cpal::default_host();

        // 2. Find the Default OUTPUT Device (our speakers/headphones)
        let device = host
            .default_output_device()
            .ok_or("No output device found.")?;

        println!("CPAL: Using device: {}", device.name().unwrap_or_default());

        // 3. Get the configuration that Windows is currently using (e.g., 48000Hz, Stereo)
        let mut config: StreamConfig = device
            .default_output_config()
            .map_err(|e| format!("Error en config: {}", e))?
            .into();

        // Force low latency by requiring only 256 samples to be filled before callback
        config.buffer_size = cpal::BufferSize::Fixed(256);

        println!(
            "CPAL: Sample Rate: {} Hz, Channels: {}",
            config.sample_rate, config.channels
        );
        // NOTE: We use build_input_stream on an output device to do "Loopback"
        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Push as many f32 as fit in the Ring
                    let _ = producer.push_slice(data);
                },
                move |err| {
                    eprintln!("Error in Audio Stream: {}", err);
                },
                None, // Optional timeout
            )
            .map_err(|e| format!("Could not build Stream: {}", e))?;

        // Start the audio capture engine at the Windows level
        stream.play().map_err(|e| format!("Error playing: {}", e))?;

        Ok(Self { _stream: stream })
    }
}
