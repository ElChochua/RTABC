fn main() {
    // Only compile the icon resource if we are on Windows
    #[cfg(target_os = "windows")]
    {
        use std::path::Path;

        // Rutas
        let png_path = "assets/icon.png";
        let ico_path = "assets/icon.ico";

        // If the PNG exists and the ICO does not exist yet (or we want to force it)
        if Path::new(png_path).exists() {
            println!("cargo:warning=Generating icon file dynamically from png...");

            // Attempt to generate or overwrite the .ico file
            // if let Ok(img) = image::open(png_path) {
            //     let _ = img.save_with_format(ico_path, ImageFormat::Ico);
            //
        }

        // Now tell winres to embed this .ico in the Windows executable
        if Path::new(ico_path).exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon(ico_path);
            if let Err(e) = res.compile() {
                println!(
                    "cargo:warning=Could not compile icon resource in Windows: {}",
                    e
                );
            }
        } else {
            println!("cargo:warning=Could not find icon to embed.");
        }
    }
}
