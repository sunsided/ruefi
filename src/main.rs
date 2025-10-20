#![no_std]
#![no_main]

use uefi::prelude::*;
use uefi::proto::console::text::Color;
use uefi::table::cfg::ConfigTableEntry;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    system::with_stdout(|stdout| {
        stdout.reset(false).unwrap();
        stdout.set_color(Color::LightRed, Color::Black).unwrap();
        stdout.output_string(cstr16!("Hello, world from bare-metal UEFI (Rust)!\r\n")).unwrap();
    });

    // Sleep 3,000,000 microseconds (i.e., 3 seconds)
    boot::stall(3_000_000);

    Status::SUCCESS
}
