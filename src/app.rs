use crate::{NetworkEvent, UiCommand};
use eframe::egui;
use local_ip_address::local_ip;
use std::sync::mpsc::{Receiver, Sender};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

pub struct App {
    // Communicators with Tokio
    tx_ui: Sender<UiCommand>,
    rx_ui: Receiver<NetworkEvent>,

    // UI State
    server_running: bool,
    status_message: String,
    local_ip_str: String,
    label: String,
    mute_local_pc: bool,

    // Tray Icon Manager
    tray_icon: Option<TrayIcon>,
}

fn load_icon() -> Icon {
    let img_bytes = include_bytes!("../assets/icon.png");
    if let Ok(img) = image::load_from_memory(img_bytes) {
        let img = img.into_rgba8();
        let (width, height) = img.dimensions();
        let rgba = img.into_raw();
        if let Ok(icon) = Icon::from_rgba(rgba, width, height) {
            return icon;
        }
    }

    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for _ in 0..(width * height) {
        rgba.push(65); // R
        rgba.push(175); // G
        rgba.push(255); // B
        rgba.push(255); // A
    }
    Icon::from_rgba(rgba, width, height).unwrap()
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        tx_ui: Sender<UiCommand>,
        rx_ui: Receiver<NetworkEvent>,
    ) -> Self {
        // Try to get the local IP of the computer to display it
        let my_local_ip = match local_ip() {
            Ok(ip) => ip.to_string(),
            Err(e) => format!("Error getting IP: {}", e),
        };

        let tray_menu = tray_icon::menu::Menu::new();
        let quit_i = tray_icon::menu::MenuItem::new("Quit RTABC", true, None);
        let _ = tray_menu.append(&quit_i);

        // Build the tray icon exactly when the Eframe window spins up
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("RTABC Server")
            .with_icon(load_icon())
            .with_menu(Box::new(tray_menu))
            .with_menu_on_left_click(false)
            .build()
            .ok();

        // Event handler (Hard Quit)
        let tx_ui_clone = tx_ui.clone();
        tray_icon::menu::MenuEvent::set_event_handler(Some(move |_event| {
            let _ = tx_ui_clone.send(UiCommand::StopServer);
            std::thread::sleep(std::time::Duration::from_millis(50)); // Damos 50ms a Tokio
            std::process::exit(0); // Terminamos limpiamente el proceso operativo
        }));

        let ctx_tray = cc.egui_ctx.clone();
        tray_icon::TrayIconEvent::set_event_handler(Some(move |event| {
            // println!("Tray event disparado: {:?}", event);
            if !matches!(
                event,
                tray_icon::TrayIconEvent::Move { .. }
                    | tray_icon::TrayIconEvent::Enter { .. }
                    | tray_icon::TrayIconEvent::Leave { .. }
            ) {
                unsafe {
                    use windows::Win32::UI::WindowsAndMessaging::{
                        FindWindowA, SW_RESTORE, ShowWindow,
                    };
                    use windows::core::s;
                    // Buscamos nuestra ventana estática
                    if let Ok(hwnd) = FindWindowA(None, s!("RTABC Server")) {
                        // Obligamos a Windows a devolverla a la vida físicamente
                        let _ = ShowWindow(hwnd, SW_RESTORE);
                    }
                }

                ctx_tray.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx_tray.send_viewport_cmd(egui::ViewportCommand::Focus);
                ctx_tray.request_repaint();
            }
        }));

        Self {
            tx_ui,
            rx_ui,
            server_running: false,
            status_message: "Server Stopped.".to_string(),
            local_ip_str: my_local_ip,
            label: "RTABC - Listen on Local Network".to_owned(),
            mute_local_pc: false,
            tray_icon,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Intercept Close Button (X) to ALWAYS minimize to tray hiding from taskbar completely
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }

        ctx.request_repaint();

        if let Ok(event) = self.rx_ui.try_recv() {
            match event {
                NetworkEvent::DiscoveryStarted(msg) => {
                    self.status_message = format!("Activated: {}", msg);
                }
                NetworkEvent::ClientConnected(ip) => {
                    self.status_message = format!("Connected to client Tauri: {}", ip);
                }
                NetworkEvent::Error(err) => {
                    self.status_message = format!("Error de Red: {}", err);
                    self.server_running = false;
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.add_space(8.0);
                egui::widgets::global_theme_preference_buttons(ui);
                ui.add_space(8.0);
                ui.label("⚙ Settings");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("X Quit Entirely").clicked() {
                        if self.server_running {
                            let _ = self.tx_ui.send(UiCommand::StopServer);
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        std::process::exit(0);
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // A big title
            ui.heading(&self.label);
            ui.separator();
            ui.add_space(20.0);

            // IP Information Box
            ui.group(|ui| {
                ui.label("Your Computer IP (LAN):");
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&self.local_ip_str)
                            .size(24.0)
                            .color(egui::Color32::from_rgb(100, 200, 255)),
                    );

                    if ui.button("Copy").clicked() {
                        ui.output_mut(|o| o.copied_text = self.local_ip_str.clone());
                    }
                });
            });

            ui.add_space(30.0);

            // Option to mute Windows
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut self.mute_local_pc,
                    "Mute Local PC (Send to phone without local echo)",
                );
            });

            ui.add_space(20.0);

            // Center our Giant Button
            ui.horizontal(|ui| {
                let button_text = if self.server_running {
                    "⏹ Stop Broadcast"
                } else {
                    "▶ Start Public Server"
                };

                let btn = ui.add_sized(
                    [250.0, 60.0],
                    egui::Button::new(egui::RichText::new(button_text).size(20.0)),
                );

                if btn.clicked() {
                    if self.server_running {
                        // Send the message through the MPSC tube without blocking the UI
                        let _ = self.tx_ui.send(UiCommand::StopServer);
                        self.server_running = false;
                        self.status_message = "Server Stopped.".to_string();
                    } else {
                        let _ = self.tx_ui.send(UiCommand::StartServer(self.mute_local_pc));
                        self.server_running = true;
                        self.status_message = "Starting UDP Broadcaster...".to_string();
                    }
                }
            });

            ui.add_space(20.0);

            // Real-time Status Message
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
