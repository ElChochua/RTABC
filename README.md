# LocalAudioLink (RTABC & RTAR)

**LocalAudioLink** is an ultra-low latency, real-time local network (LAN) audio streaming application suite. It captures the system audio from a Windows PC and streams it directly to an Android device over Wi-Fi, functioning as a wireless headset replacement with near-zero latency suitable for gaming and media consumption.


<img align="center" width="500" height="329" alt="image" src="https://github.com/user-attachments/assets/04c42fb1-a64d-4bca-84db-cc352bcb99d1" />

The project is divided into two parts:
1. **RTABC (Real-Time Audio Broadcasting Client)**: The Windows PC server written in Rust using `egui` and `tokio`.
2. **RTAR (Real-Time Audio Receiver)**: The Android client application built with Tauri and Rust.

## Key Features

- **Near-Zero Latency**: Achieved by minimizing buffer sizes across the entire stack, utilizing a 2ms asynchronous UDP transmission tick, and keeping Android CPAL pre-buffering to mere 5ms.
- **Pure-Rust µ-Law Compression**: Compresses `f32` PCM audio completely into `u8` mathematically (G.711 µ-Law) before sending it over the network. This provides an exact 75% reduction in bandwidth (from ~2 Mbps to ~500 Kbps) without relying on heavy C/C++ FFI dependencies like Opus, making the Android NDK cross-compilation seamless.
- **Smart Auto-Discovery (Hole Punching)**: The Android client automatically searches for the PC server locally using UDP Broadcasts on ephemeral ports. Once found, it automatically punches holes through the NAT to keep the exact Audio path open and clear from interference.
- **Zombie Client Purging**: The PC server acts as a passive supervisor, dropping dead clients instantly if the continuous heartbeat (ping) from the Android app is lost for more than 3 seconds.
- **Silent PC Guardian**: A hardware-level Windows Master Audio muter (`windows_mixer.rs`) allows you to stream to your phone while keeping the actual PC speakers physically muted so the audio doesn't echo in the room.

##  How it Works

1. **Audio Capture**: On the Windows PC, `cpal` binds to the WASAPI Host and captures the local loopback audio (what you hear on your speakers) in high-quality 48kHz Stereo `f32` floats.
2. **Compression (Mu-Law)**: As floats arrive at the RingBuffer, Tokio consumes them and instantly logarithmically compresses each float into a standard 8-bit `u8` integer.
3. **Transport (UDP)**: The PC packs these bytes into frames up to 1400 bytes and shoots them over UDP port `5000` to the mobile device every 2 milliseconds. UDP is used instead of TCP to prioritize absolute real-time speed over guaranteed delivery.
4. **Decompression & Playback**: The Android phone (RTAR) receives the UDP datagrams, scales the `u8` values back to `f32` floats immediately, and feeds them into its own `cpal` RingBuffer. The audio host (AAudio/Oboe) plays them back to the user's headphones in real-time.

## Running the Server (Windows PC - RTABC)

**Prerequisites**: You must have Rust installed.

1. Open your terminal and navigate to the `RTABC` directory:
   ```bash
   cd path/to/RTABC
   ```
2. Run the server using cargo:
   ```bash
   cargo run --release
   ```
3. A graphical interface (`egui`) will pop up. You can check the "Mute Local PC" option if you don't want your speakers to output sound while streaming.
4. Click **"▶ Start Public Server"**. The PC is now listening for connections.


### RTABC (Windows)
- `main.rs`: The application entry point. Coordinates EGUI and spawns the detached `tokio` multi-thread runtime.
- `app.rs`: EGUI Frontend UI definitions. 
- `network.rs`: Handles the Auto-discovery UDP Server (Port 8888) and the high-speed Audio UDP Streamer (Port 5000). Contains the `f32_to_ulaw` compressor algorithm.
- `audio.rs`: Uses `cpal` to connect to Windows Loopback to passively siphon audio data.
- `windows_mixer.rs`: Manipulates the low-level Windows COM API (`IAudioEndpointVolume`) to mute the master volume on demand.

### RTAR (Android)
- `src-tauri/src/network.rs`: Periodically pings the network looking for the PC server.
- `src-tauri/src/audio.rs`: Contains the `receive_audio_udp` function which binds to Port 5000, performs NAT hole-punching, decodes the µ-Law bytes (`ulaw_to_f32`), and pumps them to the lowest level AAudio/Oboe Android driver for playback.
