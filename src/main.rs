#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::proto::console::text::Color;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    system::with_stdout(|stdout| {
        stdout.reset(false).unwrap();
        stdout.set_color(Color::LightRed, Color::Black).unwrap();
        stdout.output_string(cstr16!("Hello, world from bare-metal UEFI (Rust)!\r\n")).unwrap();
    });

    Status::SUCCESS
}
