use eframe::egui;
use ringbuf::traits::Split;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use tokio::sync::RwLock;

mod app;
mod audio;
mod network; // <--- Importamos nuestro manejador de Audio experimental
mod windows_mixer; // <--- Módulo para silenciar localmente

// Definimos los mensajes que la UI le puede enviar al Hilo de Red (Tokio)
pub enum UiCommand {
    StartServer(bool), // Contiene si silenciar o no el PC local
    StopServer,
}

// Definimos los mensajes que el Hilo de Red (Tokio) le envía a la UI
pub enum NetworkEvent {
    DiscoveryStarted(String),
    ClientConnected(String),
    Error(String),
}

fn main() -> eframe::Result {
    // 1. Crear el canal bidireccional (MPSC: Multi-Producer, Single-Consumer)
    let (tx_ui, rx_network) = mpsc::channel::<UiCommand>();
    let (tx_network, rx_ui) = mpsc::channel::<NetworkEvent>();

    // 2. Levantar el Hilo Secundario dedicado PURAMENTE a la RED (ASYNC)
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            println!("Tokio Runtime iniciado y esperando mandos gráficos...");

            // Usaremos este canal para decirle a la sub-tarea de red que muera si apagamos el server
            let mut network_stopper_disc: Option<tokio::sync::mpsc::Sender<()>> = None;
            let mut network_stopper_audio: Option<tokio::sync::mpsc::Sender<()>> = None;

            // Estado asíncrono compartido que guardará la IP del teléfono cliente cuando se conecte
            let client_addr = Arc::new(RwLock::new(None));
            // Guardamos el stream vivo de Cpal para que siga capturando
            let mut _active_audio_stream: Option<audio::AudioCapture> = None;
            // Guardián del Silencio Físico Local de Windows
            let mut active_mute_guardian: Option<windows_mixer::VolumeManager> = None;

            loop {
                if let Ok(cmd) = rx_network.try_recv() {
                    match cmd {
                        UiCommand::StartServer(mute_local) => {
                            println!(
                                "Tokio: Recibí orden de INICIAR el servidor LAN (Mute Local: {})",
                                mute_local
                            );

                            // Preparar RingBuffer (1 segundo de tolerancia aproxmada a 48000Hz * 2 canales)
                            let rb = ringbuf::HeapRb::<f32>::new(48000 * 2);
                            let (prod, _cons) = rb.split(); // Despues daremos `_cons` al enviador UDP

                            // Arrancar motor síncrono Cpal y pasarlo al guardián
                            match audio::AudioCapture::start_loopback(prod) {
                                Ok(capture) => {
                                    _active_audio_stream = Some(capture);
                                    println!("Tokio: Cpal Ringbuf linkeados OK.");
                                    
                                    // Fase 6: Activar Mute Master local si el usuario lo pidió
                                    if mute_local {
                                        match windows_mixer::VolumeManager::new() {
                                            Ok(mut mixer) => {
                                                if let Err(e) = mixer.set_mute(true) {
                                                    println!("Tokio: Error al mutear PC: {}", e);
                                                } else {
                                                    active_mute_guardian = Some(mixer);
                                                    println!("Tokio: PC Local Silenciada (Guardian Activo).");
                                                }
                                            }
                                            Err(e) => println!("Tokio: Error creando MuteGuardian: {}", e)
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = tx_network
                                        .send(NetworkEvent::Error(format!("Error audio: {}", e)));
                                    continue; // Fallo crítico, no levantar red si falló audio
                                }
                            }

                            // Creamos un hilo ligero asincrono para el UDP Discovery
                            let (tx_stop_disc, rx_stop_disc) = tokio::sync::mpsc::channel(1);
                            network_stopper_disc = Some(tx_stop_disc);

                            // Pasamos un clon del transmisor de Eventos para que `network.rs` le hable a la UI
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

                            // Creamos un hilo para el Streamer de Audio UDP
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
                            println!("Tokio: Recibí orden de DETENER el servidor");
                            // 1. Apagar red asincrona
                            if let Some(stopper) = network_stopper_disc.take() {
                                let _ = stopper.send(()).await;
                            }
                            if let Some(stopper) = network_stopper_audio.take() {
                                let _ = stopper.send(()).await;
                            }

                            // 1.5 Limpiar cliente temporal
                            {
                                let mut addr_lock = client_addr.write().await;
                                *addr_lock = None;
                            }
                            // 2. Apagar motor de hardware de audio (Cpal lo corta al destruir el struct)
                            _active_audio_stream = None;
                            
                            // 3. Restaurar Sonido Maestro en PC Local (El Trait Drop de windows_mixer lo hace por nosotros al setear a None, pero para estar seguros:)
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

    // 3. Configurar EGUI y correr en el HILO PRINCIPAL
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([500.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "RTABC Server",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc, tx_ui, rx_ui)))),
    )
}
