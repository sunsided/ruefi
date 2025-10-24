use image::ImageReader;
use std::{env, fs, path::PathBuf};

fn main() {
    // Input asset(s)
    let in_png = PathBuf::from("assets/ruefi.png");
    println!("cargo:rerun-if-changed={}", in_png.display());

    // Load + decode PNG using the `image` crate
    let img = ImageReader::open(&in_png)
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8();
    let (w, h) = img.dimensions();
    let mut bytes = img.into_raw(); // RGBA8

    // Premultiply alpha towards black: RGB' = RGB * A/255 (leave A unchanged)
    // This makes simple RGB copying behave like blending over black for semi-transparent pixels.
    for px in bytes.chunks_mut(4) {
        // RGBA order
        let a = px[3] as u16;
        px[0] = ((px[0] as u16 * a) / 255) as u8; // R'
        px[1] = ((px[1] as u16 * a) / 255) as u8; // G'
        px[2] = ((px[2] as u16 * a) / 255) as u8; // B'
        // px[3] unchanged
    }

    // Write raw bytes for include_bytes!
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let raw_path = out_dir.join("logo.rgba"); // width*w, height*h, 4*bpp
    fs::write(&raw_path, &bytes).unwrap();

    // Generate a tiny Rust module that references the bytes
    let generated = format!(
        r#"
        // Auto-generated. Do not edit.
        #[allow(non_upper_case_globals)]
        pub const LOGO_WIDTH: usize = {w};
        #[allow(non_upper_case_globals)]
        pub const LOGO_HEIGHT: usize = {h};
        pub const LOGO_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/logo.rgba"));
        "#,
        w = w as usize,
        h = h as usize
    );
    fs::write(out_dir.join("assets_gen.rs"), generated).unwrap();
}
