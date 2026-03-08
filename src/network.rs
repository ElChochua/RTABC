use crate::NetworkEvent;
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
            // Case 1: Someone on the network sent us a message
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

            // Case 2: The main loop asks us to cancel this service (StopServer pressed)
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
        println!("Magic Ping received from {}!", src_addr);

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
            println!(
                "Wave returned to the cell phone successfully. ({})",
                src_addr
            );
        }
    }
}

/// Drena continuamente el `Ringbuf` de Audio y los encapsula en Datagramas UDP hacia el cliente
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

    let mut packet_buf = Vec::with_capacity(1024 * 4);
    // Extreme low latency: 2ms instead of 15ms.
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(2));

    // Pure helper function in Rust to compress float (f32) to µ-Law (u8)
    // Drastically reduces network weight by 75% (-24 bits of width)
    #[inline]
    fn f32_to_ulaw(sample: f32) -> u8 {
        let pcm = (sample.clamp(-1.0, 1.0) * 32767.0) as i32;
        let sign = if pcm < 0 { 0x80 } else { 0 };
        let mag = pcm.abs().clamp(0, 32635) + 132;
        let exponent = if mag >= 0x4000 {
            7
        } else if mag >= 0x2000 {
            6
        } else if mag >= 0x1000 {
            5
        } else if mag >= 0x0800 {
            4
        } else if mag >= 0x0400 {
            3
        } else if mag >= 0x0200 {
            2
        } else if mag >= 0x0100 {
            1
        } else {
            0
        };
        let mantissa = (mag >> (exponent + 3)) & 0x0F;
        !(sign | (exponent << 4) | mantissa as u8)
    }

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let target_info = { *client_addr.read().await };

                if let Some((mut addr, last_heartbeat)) = target_info {
                    // The passive Supervisor: We act as a zombie collector by checking the heartbeat
                    if last_heartbeat.elapsed().as_secs() > 3 {
                        println!("Heartbeat lost: Client {} disconnected due to inactivity (>3s). Purging IP...", addr);
                        let mut lock = client_addr.write().await;
                        *lock = None;
                        continue; // Skip to discard the buffer
                    }

                    // The client (cellphone) must listen to the audio on port 5000
                    addr.set_port(5000);

                    packet_buf.clear();

                    // Extract samples and pack them
                    loop {
                        packet_buf.clear();

                        // Fill the temporary buffer until we have a juicy frame (e.g. 960 samples)
                        while let Some(sample) = consumer.try_pop() {
                            packet_buf.push(f32_to_ulaw(sample));
                            if packet_buf.len() == 1400 { // Maximize to MTU directly to reduce WiFi overhead
                                break;
                            }
                        }

                        if !packet_buf.is_empty() {
                            if let Err(_e) = socket.send_to(&packet_buf, addr).await {
                                // Silently ignore network errors
                            }

                            // If we took less than 1400, it means the RingBuffer emptied
                            if packet_buf.len() < 1400 {
                                break;
                            }
                        } else {
                            // The ring buffer emptied completely
                            break;
                        }
                    }
                } else {
                    // If there is no client, we drain the buffer discarding it to prevent RAM overflow
                    while let Some(_) = consumer.try_pop() {}
                }
            }

            _ = rx_stop.recv() => {
                println!("Shutting down UDP audio streamer...");
                break;
            }
        }
    }
}
