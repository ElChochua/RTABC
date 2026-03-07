use crate::NetworkEvent;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::RwLock;

// Definimos los puertos y las tramas de texto mágicas que usaremos
const DISCOVERY_PORT: u16 = 8888;
const PING_MESSAGE: &[u8] = b"RTABC_DISCOVERY_PING";
const PONG_MESSAGE: &[u8] = b"RTABC_DISCOVERY_PONG";

/// Inicia el servicio de descubrimiento por red local para que el celular encuentre la PC
pub async fn start_discovery_server(
    tx_ui: std::sync::mpsc::Sender<NetworkEvent>,
    mut rx_stop: tokio::sync::mpsc::Receiver<()>,
    client_addr: Arc<RwLock<Option<SocketAddr>>>,
) {
    // Escucha en todas las interfaces de red de la PC (0.0.0.0) en el puerto acordado
    let addr = format!("0.0.0.0:{}", DISCOVERY_PORT);
    let socket = match UdpSocket::bind(&addr).await {
        Ok(s) => s,
        Err(e) => {
            let _ = tx_ui.send(NetworkEvent::Error(format!("Fallo al incio de UDP: {}", e)));
            return;
        }
    };

    // Activamos permisos para que el sistema operativo nos permita retransmitir mensajes a todos
    if let Err(e) = socket.set_broadcast(true) {
        let _ = tx_ui.send(NetworkEvent::Error(format!(
            "No se pudo configurar Broadcast UDP: {}",
            e
        )));
        return;
    }

    // Le informamos a la GUI que hemos triunfado al abrir el Socket Mágico
    let _ = tx_ui.send(NetworkEvent::DiscoveryStarted(format!(
        "Esperando Ping en puerto {}",
        DISCOVERY_PORT
    )));

    // Lo metemos en un Arc (Smart Pointer) para facilitar su acceso si creamos hilos extra después
    let socket = Arc::new(socket);
    let mut buf = [0u8; 1024];

    loop {
        // tokio::select! nos permite esperar múltiples eventos del futuro y ver cuál ocurre primero
        tokio::select! {
            // Caso 1: Alguien en la red nos envió un mensaje
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
                        println!("Error leyendo paquete de descubrimiento: {}", e);
                    }
                }
            }

            // Caso 2: El loop maestro nos pide cancelar este servicio (StopServer apretado)
            _ = rx_stop.recv() => {
                println!("Apagando servidor de descubrimiento UDP...");
                break;
            }
        }
    }
}

async fn handle_incoming_discovery_packet(
    packet: &[u8],
    src_addr: SocketAddr,
    socket: &UdpSocket,
    client_addr: &Arc<RwLock<Option<SocketAddr>>>,
    tx_ui: &std::sync::mpsc::Sender<NetworkEvent>,
) {
    // Si recibimos la clave secreta del celular
    if packet == PING_MESSAGE {
        println!("¡Recibimos un Ping Mágico desde {}!", src_addr);

        // Guardamos o actualizamos la IP del cliente en la memoria compartida
        {
            let mut addr_lock = client_addr.write().await;
            *addr_lock = Some(src_addr);
        }

        let _ = tx_ui.send(NetworkEvent::ClientConnected(src_addr.to_string()));

        // Respondemos ciegamente al celular diciéndole que somos Nostotros
        if let Err(e) = socket.send_to(PONG_MESSAGE, src_addr).await {
            println!(
                "Error en Discovery al intentar responder con PONG a {}: {}",
                src_addr, e
            );
        } else {
            println!("Onda devuelta al celular exitosamente. ({})", src_addr);
        }
    }
}

/// Drena continuamente el `Ringbuf` de Audio y los encapsula en Datagramas UDP hacia el cliente
pub async fn start_audio_streamer(
    mut consumer: impl ringbuf::traits::Consumer<Item = f32> + Send + 'static,
    mut rx_stop: tokio::sync::mpsc::Receiver<()>,
    client_addr: Arc<RwLock<Option<SocketAddr>>>,
) {
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            println!("Fallo al alocar socket de audio: {}", e);
            return;
        }
    };

    let mut packet_buf = Vec::with_capacity(1024 * 4);
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(5));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let target = { *client_addr.read().await };

                if let Some(mut addr) = target {
                    // El cliente (celular) debe escuchar el audio en el puerto 5001
                    addr.set_port(5001);

                    packet_buf.clear();

                    // Extraemos los samples e intentamos empaquetarlos a un límite prudente
                    while let Some(sample) = consumer.try_pop() {
                        packet_buf.extend_from_slice(&sample.to_le_bytes());
                        // Límite de 1024 bytes por paquete (256 samples `f32` de 4 bytes) evita fragmentación
                        if packet_buf.len() >= 1024 {
                            break;
                        }
                    }

                    if !packet_buf.is_empty() {
                        let _ = socket.send_to(&packet_buf, addr).await;
                    }
                } else {
                    // Si no hay cliente, dreneamos el buffer descartándolo para evitar que se desborde la RAM
                    while let Some(_) = consumer.try_pop() {}
                }
            }

            _ = rx_stop.recv() => {
                println!("Apagando streamer UDP de audio...");
                break;
            }
        }
    }
}
