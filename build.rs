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
    let bytes = img.into_raw(); // RGBA8

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
