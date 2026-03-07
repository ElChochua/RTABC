use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};
use ringbuf::traits::Producer;

pub struct AudioCapture {
    // Mantendremos vivo el Stream aquí para que no se destruya al salir de la función
    _stream: Stream,
}

impl AudioCapture {
    /// Inicia la captura del Loopback (El audio general de Windows)
    pub fn start_loopback(
        mut producer: impl ringbuf::traits::Producer<Item = f32> + Send + 'static,
    ) -> Result<Self, String> {
        // 1. Obtener el Host de Audio (WASAPI en Windows)
        let host = cpal::default_host();

        // 2. Encontrar el Dispositivo Predeterminado de SALIDA (nuestras bocinas/auriculares)
        let device = host
            .default_output_device()
            .ok_or("No se encontró ningún dispositivo de salida.")?;

        println!(
            "CPAL: Utilizando dispositivo: {}",
            device.name().unwrap_or_default()
        );

        // 3. Obtener la configuración que Windows está usando actualmente (Ej. 48000Hz, Stereo)
        let config: StreamConfig = device
            .default_output_config()
            .map_err(|e| format!("Error en config: {}", e))?
            .into();

        println!(
            "CPAL: Tasa de Muestreo: {} Hz, Canales: {}",
            config.sample_rate, config.channels
        );
        // OJO: Usamos build_input_stream en un dispositivo de salida para hacer "Loopback"
        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Empuja tantos f32 como entren en el Anillo
                    let _ = producer.push_slice(data);
                },
                move |err| {
                    eprintln!("Error en el Stream de Audio: {}", err);
                },
                None, // Timeout opcional
            )
            .map_err(|e| format!("No se pudo construir Stream: {}", e))?;

        // Arrancamos el motor de captura de audio a nivel Windows
        stream
            .play()
            .map_err(|e| format!("Error reproduciendo: {}", e))?;

        Ok(Self { _stream: stream })
    }
}
