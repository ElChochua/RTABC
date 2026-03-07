use crate::{NetworkEvent, UiCommand};
use eframe::egui;
use local_ip_address::local_ip;
use std::sync::mpsc::{Receiver, Sender};

pub struct App {
    // Comunicadores con Tokio
    tx_ui: Sender<UiCommand>,
    rx_ui: Receiver<NetworkEvent>,

    // Estado de la UI
    server_running: bool,
    status_message: String,
    local_ip_str: String,
    label: String,
    mute_local_pc: bool,
}

impl App {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        tx_ui: Sender<UiCommand>,
        rx_ui: Receiver<NetworkEvent>,
    ) -> Self {
        // Intentar obtener la IP local de la computadora para mostrarla
        let my_local_ip = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(e) => format!("Error obteniendo IP: {}", e),
        };

        Self {
            tx_ui,
            rx_ui,
            server_running: false,
            status_message: "Servidor Detenido.".to_string(),
            local_ip_str: my_local_ip,
            label: "RTABC - Escucha en Red Local".to_owned(),
            mute_local_pc: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Le dice a la UI que se redibuje constantemente para animaciones y revisar canales
        ctx.request_repaint();

        // 1. Revisamos mensajería que venga DESDE Tokio (Red) hacia nosotros asíncronamente
        if let Ok(event) = self.rx_ui.try_recv() {
            match event {
                NetworkEvent::DiscoveryStarted(msg) => {
                    self.status_message = format!("Activado: {}", msg);
                }
                NetworkEvent::ClientConnected(ip) => {
                    self.status_message = format!("Conectado a cliente Tauri: {}", ip);
                }
                NetworkEvent::Error(err) => {
                    self.status_message = format!("Error de Red: {}", err);
                    self.server_running = false;
                }
            }
        }

        // 2. Construcción Visual
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.add_space(8.0);
                egui::widgets::global_theme_preference_buttons(ui); // Dark/Light mode nativo
                ui.add_space(8.0);
                ui.label("⚙ Configuración");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Un título grande
            ui.heading(&self.label);
            ui.separator();
            ui.add_space(20.0);

            // Caja de información IP
            ui.group(|ui| {
                ui.label("📡 Dirección de tu Computadora (IP LAN):");
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&self.local_ip_str)
                            .size(24.0)
                            .color(egui::Color32::from_rgb(100, 200, 255)),
                    );

                    if ui.button("📋 Copiar").clicked() {
                        ui.output_mut(|o| o.copied_text = self.local_ip_str.clone());
                    }
                });
            });

            ui.add_space(30.0);

            // Opción para mutear Windows
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut self.mute_local_pc,
                    "Silenciar PC Local (Envío a celular sin eco local)",
                );
            });

            ui.add_space(20.0);

            // Centrar nuestro Botón Gigante
            ui.horizontal(|ui| {
                let button_text = if self.server_running {
                    "⏹ Detener Transmisión"
                } else {
                    "▶ Iniciar Servidor Público"
                };

                let btn = ui.add_sized(
                    [250.0, 60.0],
                    egui::Button::new(egui::RichText::new(button_text).size(20.0)),
                );

                if btn.clicked() {
                    if self.server_running {
                        // Enviamos el mensaje por el tubo MPSC sin bloquear la UI
                        let _ = self.tx_ui.send(UiCommand::StopServer);
                        self.server_running = false;
                        self.status_message = "Servidor Detenido.".to_string();
                    } else {
                        let _ = self.tx_ui.send(UiCommand::StartServer(self.mute_local_pc));
                        self.server_running = true;
                        self.status_message = "Iniciando Broadcaster UDP...".to_string();
                    }
                }
            });

            ui.add_space(20.0);

            // Mensaje de Estado en tiempo real
            ui.label(
                egui::RichText::new(&self.status_message).color(if self.server_running {
                    egui::Color32::LIGHT_GREEN
                } else {
                    egui::Color32::LIGHT_RED
                }),
            );
        });
    }
}
