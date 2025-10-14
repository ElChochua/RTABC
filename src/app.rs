use eframe::{egui, Storage};
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App{
    address: String,
    #[serde(skip)]
    label: String
}
impl Default for App{
    fn default() -> Self{
        Self{
            address: "0.0.0.0".to_owned(),
            label: "Real Time Audio Broadcast".to_owned(),
        }
    }
}
impl App{
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self{
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        }else{
            Default::default()
        }
    }
}
impl eframe::App for App{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                    ui.add_space(16.0);
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("{}",self.label));
            ui.separator();
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);

            });
        });

    }
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }
}