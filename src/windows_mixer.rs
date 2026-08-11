use std::thread;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator, eMultimedia, eRender};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::core::Result;

pub struct VolumeManager {}

impl VolumeManager {
    pub fn new() -> std::result::Result<Self, String> {
        Ok(Self {})
    }

    pub fn set_mute(&mut self, state: bool) -> std::result::Result<(), String> {
        let handle = thread::spawn(move || {
            unsafe {
                // Initialize COM
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

                // Try to find device and invoke mute
                if let Err(e) = set_master_mute_unsafe(state) {
                    eprintln!("Native CoreAudio failure: {:?}", e);
                }

                // 3. Clean up COM
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
                let _ = set_master_mute_unsafe(false); // Restore sound always
                CoUninitialize();
            }
        });
        let _ = handle.join();
    }
}

// Internal unsafe function to consume the C++ APIs of windows-rs directly
unsafe fn set_master_mute_unsafe(mute: bool) -> Result<()> {
    unsafe {
        // A) Create the MMDevice enumerator
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        // B) Get the default Multimedia Playback endpoint (Main Speakers/Headphones)
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;

        // C) Activate the Volume interface of that single endpoint
        let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

        // D) Modify Mute property (Hardware level)
        endpoint_volume.SetMute(mute, std::ptr::null())?;

        Ok(())
    }
}
