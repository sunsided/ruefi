#![no_std]
#![no_main]

use uefi::prelude::*;

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    // simple console output:
    uefi::println!("Hello, world from Rust UEFI!");

    system::with_stdout(|stdout| {
        use core::fmt::Write;
        let _ = writeln!(stdout, "Hello via with_stdout()");
    });

    Status::SUCCESS
}

/// Minimal panic handler: stay put if something goes wrong.
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
