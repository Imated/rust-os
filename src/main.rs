#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

pub mod cpu;
pub mod graphics;
pub mod mem;
pub mod types;

extern crate alloc;

use crate::cpu::gdt;
use crate::cpu::interrupts;
use crate::graphics::init_graphics;
use crate::graphics::terminal::{Terminal, TerminalLogger};
use alloc::boxed::Box;
use core::panic::PanicInfo;
use log::{debug, error, info, trace, warn};
use x86_64::instructions::hlt;
use x86_64::instructions::interrupts::int3;

static LOGGER: TerminalLogger = TerminalLogger;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        Terminal::with_forced(|terminal| {
            terminal.clear();
        })
    };
    error!("\n{:?}", info.message());
    x86_64::instructions::interrupts::disable();
    loop {
        hlt();
    }
}

fn init() {
    mem::init();
    gdt::init();
    interrupts::init();
    init_graphics();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    x86_64::instructions::interrupts::disable();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    init();
    int3();

    trace!("trace: test trace");
    debug!("debug: test debug ({}, {})", 67, 67);
    info!("info: test info");
    warn!("warn: test warn");
    error!("error: test err");

    let e = Box::new(67);
    trace!("eeee heap works!! {e}");

    loop {
        hlt();
    }
}
