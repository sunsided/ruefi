#![no_std]
#![no_main]
#![allow(unsafe_code)]

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

    wait_for_key();

    Status::SUCCESS
}

fn wait_for_key() {
    system::with_stdout(|out| {
        let _ = out.output_string(cstr16!("Press any key or wait 5s…\r\n"));
    });

    // timer event for 5 seconds (units are 100 ns)
    let timer = unsafe { boot::create_event(boot::EventType::TIMER, boot::Tpl::APPLICATION, None, None).unwrap() };
    boot::set_timer(&timer, boot::TimerTrigger::Relative(5 * 10_000_000)).unwrap();

    // key event
    let key_ev = system::with_stdin(|stdin| stdin.wait_for_key_event().unwrap());

    // wait for either event
    let mut events = [key_ev, timer];
    let idx = boot::wait_for_event(&mut events).unwrap();
    match idx {
        0 => {
            // index 0: key event fired first
            let _ = system::with_stdin(|stdin| stdin.read_key());
        }
        1 => {
            // index 1: timer fired first
            system::with_stdout(|out| {
                let _ = out.output_string(cstr16!("\r\nTimeout reached. Goodbye!\r\n"));
            });
        }
        _ => unreachable!(),
    }
}
