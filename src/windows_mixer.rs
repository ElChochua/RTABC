use std::thread;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator, eMultimedia, eRender};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::core::{Interface, Result};

pub struct VolumeManager {}

impl VolumeManager {
    pub fn new() -> std::result::Result<Self, String> {
        Ok(Self {})
    }

    pub fn set_mute(&mut self, state: bool) -> std::result::Result<(), String> {
        let handle = thread::spawn(move || {
            unsafe {
                // 1. Inicializar COM
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

                // 2. Intentar buscar dispositivo e invocar mute
                if let Err(e) = set_master_mute_unsafe(state) {
                    eprintln!("Fallo en CoreAudio nativo: {:?}", e);
                }

                // 3. Limpiar COM
                CoUninitialize();
            }
        });

        let _ = handle.join();
        Ok(())
    }
}

impl Drop for VolumeManager {
    fn drop(&mut self) {
        let handle = thread::spawn(|| {
            unsafe {
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                let _ = set_master_mute_unsafe(false); // Restaurar sonido siempre
                CoUninitialize();
            }
        });
        let _ = handle.join();
    }
}

// Función interna unsafe para consumir las APIs C++ de windows-rs directamente
unsafe fn set_master_mute_unsafe(mute: bool) -> Result<()> {
    unsafe {
        // A) Crear el enumerador MMDevice
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        // B) Obtener el endpoint por defecto de Reproducción Múltimedia (Parlantes/Audífonos principales)
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;

        // C) Activar la interfaz de Volumen de ese único endpoint
        let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

        // D) Modificar propiedad Mute (Hardware level)
        endpoint_volume.SetMute(mute, std::ptr::null())?;

        Ok(())
    }
}
