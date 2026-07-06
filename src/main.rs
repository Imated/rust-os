#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

pub mod cpu;
pub mod graphics;
pub mod types;

use crate::cpu::gdt;
use crate::cpu::interrupts;
use crate::graphics::framebuffer::Framebuffer;
use crate::graphics::terminal::{Terminal, TerminalLogger};
use core::panic::PanicInfo;
use core::ptr::NonNull;
use lazy_static::lazy_static;
use limine::request::FramebufferRequest;
use log::{debug, error, info, trace, warn};
use spin::Mutex;
use x86_64::instructions::hlt;
use x86_64::instructions::interrupts::without_interrupts;

#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

lazy_static! {
    pub static ref FRAMEBUFFER: Mutex<Framebuffer> = unsafe {
        Mutex::new(Framebuffer::new(
            FRAMEBUFFER_REQUEST
                .response()
                .unwrap()
                .framebuffers()
                .first()
                .unwrap(),
        ))
    };
}

lazy_static! {
    pub static ref TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new());
}

static LOGGER: TerminalLogger = TerminalLogger;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    without_interrupts(|| TERMINAL.lock().clear());
    error!("Panic occurred: \n{:?}", info.message());
    loop {
        hlt();
    }
}

fn init() {
    gdt::init();
    interrupts::init();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn _start() -> ! {
    x86_64::instructions::interrupts::disable();
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);
    TERMINAL.lock().clear();

    init();

    trace!("trace: test trace");
    debug!("debug: test debug ({}, {})", 67, 67);
    info!("info: test info");
    warn!("warn: test warn");
    error!("error: test err");

    loop {
        hlt();
    }
}
