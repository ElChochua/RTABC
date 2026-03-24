use image::{ImageBuffer, Rgba};
use std::fs;

fn main() {
    let _ = fs::create_dir_all("assets");
    let mut img = ImageBuffer::new(32, 32);
    for (_x, _y, pixel) in img.enumerate_pixels_mut() {
        *pixel = Rgba([65, 175, 255, 255]);
    }
    img.save("assets/icon.png").unwrap();
    println!("Icon successfully created at assets/icon.png");
}
