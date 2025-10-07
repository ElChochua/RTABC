use eframe::egui;
use eframe::egui::{CentralPanel, ViewportBuilder};
#[derive(Default)]
struct App{

}
impl eframe::App for App{
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui|  ui.heading("Que pasa calabaza"));
    }
}
impl App{

}
fn main() {
    println!("Hello, world!");
    let options = eframe::NativeOptions{
        viewport: eframe::egui::ViewportBuilder::default().
            with_resizable(true).
            with_inner_size([320.0,240.0]),
        ..Default::default()
    };
    eframe::run_native("RTABC", options, Box::new(|_cc| Ok(Box::<App>::default())));
}
