#![no_std]
#![no_main]

pub mod types;
pub mod graphics;

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use limine::request::FramebufferRequest;
use log::{debug, error, info, trace, warn};
use spin::Mutex;
use x86_64::instructions::hlt;
use crate::graphics::framebuffer::Framebuffer;
use crate::graphics::terminal::{Terminal, TerminalLogger};
use crate::types::Color;

#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

lazy_static! {
    pub static ref FRAMEBUFFER: Mutex<Framebuffer> = unsafe {
        Mutex::new(Framebuffer::new(FRAMEBUFFER_REQUEST.response().unwrap().framebuffers().first().unwrap()))
    };
}

lazy_static! {
    pub static ref TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new());
}

static LOGGER: TerminalLogger = TerminalLogger;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    log::set_logger(&LOGGER).expect("ee");
    log::set_max_level(log::LevelFilter::Trace);
    FRAMEBUFFER.lock().clear(Color::BACKGROUND_COLOR);

    trace!("trace: test trace");
    debug!("debug: test debug ({}, {})", 67, 67);
    info!("info: test info");
    warn!("warn: test warn");
    error!("error: test err");

    hlt();

    loop {}
}