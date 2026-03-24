use std::io::Cursor;

fn main() {
    let img = image::open("assets/icon.png").expect("Failed to open icon.png");
    let (width, height) = img.dimensions();
    
    // El crate image incluye un IcoEncoder
    img.save_with_format("assets/icon.ico", image::ImageFormat::Ico).expect("Failed to save ico");
    println!("PNG Successfully converted to ICO at assets/icon.ico");
}
