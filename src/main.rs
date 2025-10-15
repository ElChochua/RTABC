use eframe::egui;
use tokio::task;

mod app;
#[tokio::main]
async fn main() {
    let v = vec![1,2,3];
    env_logger::init();
    let native_options = eframe::NativeOptions{
        viewport: egui::ViewportBuilder::default().with_inner_size([900.0,600.0]).
            with_min_inner_size([500.0,300.0]),
        ..Default::default()
    };
    eframe::run_native("RTABC",
                       native_options,
                       Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    ).expect("TODO: panic message");
    //Just testing the tokio library
    task::spawn(async move {
        println!("vec {:?}", v);
    });
}
