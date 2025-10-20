#![no_std]
#![no_main]
#![allow(unsafe_code)]

use uefi::prelude::*;
use uefi::proto::console::text::Color;

#[entry]
fn main() -> Status {
    uefi::helpers::init().expect("failed to initialize UEFI");

    system::with_stdout(|stdout| {
        stdout.reset(false).expect("failed to write to reset stdout");
        stdout.set_color(Color::LightRed, Color::Black).expect("failed to write to set colors");
        stdout.output_string(cstr16!("Hello, world from (almost) bare-metal UEFI!\r\n")).expect("failed to write to stdout");
    });

    wait_for_key();

    Status::SUCCESS
}

fn wait_for_key() {
    system::with_stdout(|out| {
        out.output_string(cstr16!("Press any key or wait 5s…\r\n")).expect("failed to write to stdout");
    });

    // timer event for 5 seconds (units are 100 ns)
    let timer = unsafe { boot::create_event(boot::EventType::TIMER, boot::Tpl::APPLICATION, None, None).expect("failed to create timer event") };
    boot::set_timer(&timer, boot::TimerTrigger::Relative(5 * 10_000_000)).expect("failed to set timer");

    // key event
    let key_ev = system::with_stdin(|stdin| stdin.wait_for_key_event().expect("failed to wait for key event"));

    // wait for either event
    let mut events = [key_ev, timer];
    let idx = boot::wait_for_event(&mut events).expect("failed to wait for event");
    match idx {
        0 => {
            // index 0: key event fired first
            system::with_stdin(|stdin| stdin.read_key()).expect("failed to read key");
        }
        1 => {
            // index 1: timer fired first
            system::with_stdout(|out| {
                out.output_string(cstr16!("\r\nTimeout reached. Goodbye!\r\n")).expect("failed to write to stdout");
            });
        }
        _ => unreachable!(),
    }
}
