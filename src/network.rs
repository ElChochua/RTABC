use crate::NetworkEvent;
use crate::audio_codec::{self, AudioEncoder};
use crate::protocol::{self, Codec, FLAG_DISCONTINUITY, MediaKind, PacketHeader};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

// Define ports and magic text frames to be used
const DISCOVERY_PORT: u16 = 8888;
const PING_MESSAGE: &[u8] = b"RTABC_DISCOVERY_PING";
const PONG_MESSAGE: &[u8] = b"RTABC_DISCOVERY_PONG";

/// Starts the local network discovery service so the cell phone can find the PC
pub async fn start_discovery_server(
    tx_ui: std::sync::mpsc::Sender<NetworkEvent>,
    mut rx_stop: tokio::sync::mpsc::Receiver<()>,
    client_addr: Arc<RwLock<Option<(SocketAddr, std::time::Instant)>>>,
) {
    // Listen on all network interfaces of the PC (0.0.0.0) on the agreed port
    let addr = format!("0.0.0.0:{}", DISCOVERY_PORT);
    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => s,
        Err(e) => {
            let _ = tx_ui.send(NetworkEvent::Error(format!("Failed to start UDP: {}", e)));
            return;
        }
    };

    // Enable permissions for the operating system to allow us to broadcast messages to everyone
    if let Err(e) = socket.set_broadcast(true) {
        let _ = tx_ui.send(NetworkEvent::Error(format!(
            "Failed to configure UDP Broadcast: {}",
            e
        )));
        return;
    }

    // Inform the GUI that we have succeeded in opening the Magic Socket
    let _ = tx_ui.send(NetworkEvent::DiscoveryStarted(format!(
        "Waiting for Ping on port {}",
        DISCOVERY_PORT
    )));

    // Put it in an Arc (Smart Pointer) to facilitate its access if we create extra threads later
    let socket = Arc::new(socket);
    let mut buf = [0u8; 1024];

    loop {
        // tokio::select! allows us to wait for multiple future events and see which one occurs first
        tokio::select! {
            // Someone on the network sent us a message
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, src_addr)) => {
                        handle_incoming_discovery_packet(
                            &buf[..len],
                            src_addr,
                            &socket,
                            &client_addr,
                            &tx_ui,
                        )
                        .await;
                    }
                    Err(e) => {
                        println!("Error reading discovery packet: {}", e);
                    }
                }
            }

            // The main loop asks us to cancel this service (StopServer pressed)
            _ = rx_stop.recv() => {
                println!("Shutting down UDP discovery server...");
                break;
            }
        }
    }
}

async fn handle_incoming_discovery_packet(
    packet: &[u8],
    src_addr: SocketAddr,
    socket: &UdpSocket,
    client_addr: &Arc<RwLock<Option<(SocketAddr, std::time::Instant)>>>,
    tx_ui: &std::sync::mpsc::Sender<NetworkEvent>,
) {
    // If we receive the secret key from the cell phone
    if packet == PING_MESSAGE {
        // println!("Magic Ping received from {}!", src_addr);

        // Save or update the client's IP in shared memory by adding the current exact time
        {
            let mut addr_lock = client_addr.write().await;
            *addr_lock = Some((src_addr, std::time::Instant::now()));
        }

        let _ = tx_ui.send(NetworkEvent::ClientConnected(src_addr.to_string()));

        // Respond to the cell phone saying we are here
        if let Err(e) = socket.send_to(PONG_MESSAGE, src_addr).await {
            println!(
                "Error in Discovery when trying to respond with PONG to {}: {}",
                src_addr, e
            );
        } else {
            // println!(
            //     "Wave returned to the cell phone successfully. ({})",
            //     src_addr
            // );
        }
    }
}

/// Drain the audio ringbuffer and send it to the client
pub async fn start_audio_streamer(
    mut consumer: impl ringbuf::traits::Consumer<Item = f32> + Send + 'static,
    mut rx_stop: tokio::sync::mpsc::Receiver<()>,
    client_addr: Arc<RwLock<Option<(SocketAddr, std::time::Instant)>>>,
) {
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to allocate audio socket: {}", e);
            return;
        }
    };

    let mut packet_buf = Vec::with_capacity(protocol::MAX_DATAGRAM_LEN);
    let mut frame_buf = Vec::with_capacity(audio_codec::FRAME_SAMPLES);
    let mut encoder = AudioEncoder::new();
    let stream_id = 1;
    let mut sequence = 0_u32;
    let mut discontinuity = false;
    let mut connected = false;
    // Extreme low latency: 2ms instead of 15ms.
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(2));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let target_info = { *client_addr.read().await };

                if let Some((mut addr, last_heartbeat)) = target_info {
                    if !connected {
                        encoder.reset();
                        frame_buf.clear();
                        sequence = 0;
                        discontinuity = true;
                        connected = true;
                    }

                    // The passive Supervisor: We act as a zombie collector by checking the heartbeat
                    // Relaxed to tolerate Android's "Doze Mode" (5 minutes = 300 seconds)
                    if last_heartbeat.elapsed().as_secs() > 300 {
                        println!("Heartbeat lost: Client {} disconnected due to extended inactivity (>5m Doze Mode). Purging IP...", addr);
                        let mut lock = client_addr.write().await;
                        *lock = None;
                        continue; // Skip to discard the buffer
                    }

                    // The client (cellphone) must listen to the audio on port 5000
                    addr.set_port(5000);

                    // Keep no more than 30 ms of captured audio waiting for the encoder.
                    let max_backlog = audio_codec::FRAME_SAMPLES * 3;
                    let backlog = consumer.occupied_len();
                    if backlog > max_backlog {
                        frame_buf.clear();
                        let drop_count = (backlog - max_backlog) / audio_codec::CHANNELS
                            * audio_codec::CHANNELS;
                        for _ in 0..drop_count {
                            let _ = consumer.try_pop();
                        }
                        encoder.reset();
                        discontinuity = true;
                    }

                    // Cap work per tick so encoding never monopolizes the runtime.
                    for _ in 0..4 {
                        while frame_buf.len() < audio_codec::FRAME_SAMPLES {
                            let Some(sample) = consumer.try_pop() else {
                                break;
                            };
                            frame_buf.push(sample);
                        }
                        if frame_buf.len() < audio_codec::FRAME_SAMPLES {
                            break;
                        }

                        let payload = match encoder.encode(&frame_buf) {
                            Ok(payload) => payload,
                            Err(error) => {
                                eprintln!("Opus encode failed: {error}");
                                encoder.reset();
                                frame_buf.clear();
                                discontinuity = true;
                                break;
                            }
                        };
                        frame_buf.clear();

                        let header = PacketHeader {
                            media_kind: MediaKind::Audio,
                            codec: Codec::Opus,
                            flags: if discontinuity { FLAG_DISCONTINUITY } else { 0 },
                            stream_id,
                            sequence,
                            timestamp_us: u64::from(sequence)
                                * audio_codec::FRAME_DURATION_MS as u64
                                * 1_000,
                            fragment_index: 0,
                            fragment_count: 1,
                        };
                        discontinuity = false;
                        if protocol::write_datagram(header, &payload, &mut packet_buf).is_err() {
                            break;
                        }
                        if let Err(_error) = socket.send_to(&packet_buf, addr).await {
                            // A late audio packet is worse than a dropped packet.
                        }
                        sequence = sequence.wrapping_add(1);
                    }
                } else {
                    // If there is no client, we drain the buffer discarding it to prevent RAM overflow
                    while consumer.try_pop().is_some() {}
                    if connected {
                        encoder.reset();
                        frame_buf.clear();
                        connected = false;
                    }
                }
            }

            _ = rx_stop.recv() => {
                println!("Shutting down UDP audio streamer...");
                break;
            }
        }
    }
}

/// Start the AVRCP UDP server to receive media commands from Android
pub async fn start_avrcp_receiver(mut rx_stop: tokio::sync::mpsc::Receiver<()>) {
    let addr = "0.0.0.0:8889";
    let socket = match UdpSocket::bind(addr).await {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to allocate AVRCP socket on {}: {}", addr, e);
            return;
        }
    };

    println!("AVRCP UDP Server listening for media commands on {}", addr);

    // We use enigo 0.1.2 that exposes KeyboardControllable and Key
    use enigo::{Enigo, Key, KeyboardControllable};
    let mut enigo = Enigo::new();
    let mut buf = [0u8; 128];

    loop {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, _src_addr)) => {
                        if let Ok(command) = std::str::from_utf8(&buf[..len]) {
                            let command = command.trim();
                            // println!("AVRCP Command received from {}: {:?}", src_addr, command);

                            match command {
                                "play_pause" => enigo.key_click(Key::MediaPlayPause),
                                "next" => enigo.key_click(Key::MediaNextTrack),
                                "prev" => enigo.key_click(Key::MediaPrevTrack),
                                _ => println!("Unknown AVRCP command: {}", command),
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading AVRCP packet: {}", e);
                    }
                }
            }

            _ = rx_stop.recv() => {
                println!("Shutting down AVRCP receiver...");
                break;
            }
        }
    }
}
