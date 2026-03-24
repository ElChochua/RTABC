#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use ringbuf::traits::Split;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use tokio::sync::RwLock;

// Main Entry Point

mod app;
mod audio;
mod network; // <--- Import our experimental audio handler
mod windows_mixer; // <--- Module to silence locally

// Define the messages that the UI can send to the Network Thread (Tokio)
pub enum UiCommand {
    StartServer(bool), // Contains whether to silence the local PC
    StopServer,
}

// Define the messages that the Network Thread (Tokio) sends to the UI
pub enum NetworkEvent {
    DiscoveryStarted(String),
    ClientConnected(String),
    Error(String),
}

fn main() -> eframe::Result {
    // 1. Create the bidirectional channel (MPSC: Multi-Producer, Single-Consumer)
    let (tx_ui, rx_network) = mpsc::channel::<UiCommand>();
    let (tx_network, rx_ui) = mpsc::channel::<NetworkEvent>();

    // 2. Spawn the Secondary Thread dedicated PURELY to the NETWORK (ASYNC)
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
           // println!("Tokio Runtime started and waiting for graphic commands...");

            // We will use this channel to tell the network sub-task to die if we turn off the server
            let mut network_stopper_disc: Option<tokio::sync::mpsc::Sender<()>> = None;
            let mut network_stopper_audio: Option<tokio::sync::mpsc::Sender<()>> = None;
            let mut network_stopper_avrcp: Option<tokio::sync::mpsc::Sender<()>> = None;

            // Shared asynchronous state that will store the client phone's IP and its last sighting (Heartbeat)
            let client_addr: Arc<RwLock<Option<(std::net::SocketAddr, std::time::Instant)>>> = Arc::new(RwLock::new(None));
            // We save the live Cpal stream so it keeps capturing
            let mut _active_audio_stream: Option<audio::AudioCapture> = None;
            // Guardian of the Local Physical Silence of Windows
            let mut active_mute_guardian: Option<windows_mixer::VolumeManager> = None;

            loop {
                if let Ok(cmd) = rx_network.try_recv() {
                    match cmd {
                        UiCommand::StartServer(mute_local) => {
                            println!(
                                "Tokio: Received order to START the LAN server (Local Mute: {})",
                                mute_local
                            );

                            // Prepare RingBuffer (1 second of approximate tolerance at 48000Hz * 2 channels)
                            let rb = ringbuf::HeapRb::<f32>::new(48000 * 2);
                            let (prod, _cons) = rb.split(); // We will give `_cons` to the UDP sender later

                            // Start synchronous Cpal motor and pass it to the guardian
                            match audio::AudioCapture::start_loopback(prod) {
                                Ok(capture) => {
                                    _active_audio_stream = Some(capture);
                                    println!("Tokio: Cpal Ringbuf linked OK.");
                                    
                                    // Phase 6: Activate local Master Mute if the user requested it
                                    if mute_local {
                                        match windows_mixer::VolumeManager::new() {
                                            Ok(mut mixer) => {
                                                if let Err(e) = mixer.set_mute(true) {
                                                    println!("Tokio: Error muting PC: {}", e);
                                                } else {
                                                    active_mute_guardian = Some(mixer);
                                                    println!("Tokio: PC Local Silenced (Guardian Active).");
                                                }
                                            }
                                            Err(e) => println!("Tokio: Error creating MuteGuardian: {}", e)
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx_network
                                        .send(NetworkEvent::Error(format!("Error audio: {}", e)));
                                    continue; // Critical failure, do not raise network if audio failed
                                }
                            }

                            // Create a lightweight asynchronous thread for UDP Discovery
                            let (tx_stop_disc, rx_stop_disc) = tokio::sync::mpsc::channel(1);
                            network_stopper_disc = Some(tx_stop_disc);

                            // Pass a clone of the Event transmitter so `network.rs` can talk to the UI
                            let tx_net_clone = tx_network.clone();
                            let client_addr_disc = client_addr.clone();

                            tokio::spawn(async move {
                                network::start_discovery_server(
                                    tx_net_clone,
                                    rx_stop_disc,
                                    client_addr_disc,
                                )
                                .await;
                            });

                            // Create a thread for the UDP AVRCP Receiver
                            let (tx_stop_avrcp, rx_stop_avrcp) = tokio::sync::mpsc::channel(1);
                            network_stopper_avrcp = Some(tx_stop_avrcp);

                            tokio::spawn(async move {
                                network::start_avrcp_receiver(rx_stop_avrcp).await;
                            });

                            // Create a thread for the UDP Audio Streamer
                            let (tx_stop_audio, rx_stop_audio) = tokio::sync::mpsc::channel(1);
                            network_stopper_audio = Some(tx_stop_audio);
                            let client_addr_audio = client_addr.clone();

                            tokio::spawn(async move {
                                network::start_audio_streamer(
                                    _cons,
                                    rx_stop_audio,
                                    client_addr_audio,
                                )
                                .await;
                            });
                        }
                        UiCommand::StopServer => {
                            println!("Tokio: Received order to STOP the server");
                            // 1. Shut down the async network threads
                            if let Some(stopper) = network_stopper_disc.take() {
                                let _ = stopper.send(()).await;
                            }
                            if let Some(stopper) = network_stopper_audio.take() {
                                let _ = stopper.send(()).await;
                            }
                            if let Some(stopper) = network_stopper_avrcp.take() {
                                let _ = stopper.send(()).await;
                            }

                            // 1.5 Clear temporary client
                            {
                                let mut addr_lock = client_addr.write().await;
                                *addr_lock = None;
                            }
                            // 2. Turn off the audio hardware motor (Cpal cuts it off when destroying the struct)
                            _active_audio_stream = None;
                            
                            // 3. Restore Local Master Sound (The Drop Trait of windows_mixer does it for us by setting to None, but just in case:)
                            if let Some(mut guardian) = active_mute_guardian.take() {
                                let _ = guardian.set_mute(false);
                            }
                        }
                    }
                }

                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
    });

    // 3. Configure EGUI and run on the MAIN THREAD
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([900.0, 600.0])
        .with_min_inner_size([500.0, 300.0]);

    // Cargar icono dinámico main incrustado al binario
    let img_bytes = include_bytes!("../assets/icon.png");
    if let Ok(icon) = eframe::icon_data::from_png_bytes(img_bytes) {
        viewport = viewport.with_icon(Arc::new(icon));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "RTABC Server",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc, tx_ui, rx_ui)))),
    )
}
