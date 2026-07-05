#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

pub mod types;
pub mod graphics;
pub mod cpu;

use core::panic::PanicInfo;
use core::ptr::NonNull;
use lazy_static::lazy_static;
use limine::request::FramebufferRequest;
use log::{debug, error, info, trace, warn};
use spin::Mutex;
use volatile::VolatilePtr;
use x86_64::instructions::hlt;
use x86_64::instructions::interrupts::int3;
use crate::cpu::interrupts::init_interrupts;
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
fn panic(info: &PanicInfo) -> ! {
    TERMINAL.lock().clear();
    error!("Panic occurred: \n{:?}", info.message());
    hlt();
    loop {}
}

fn init() {
    init_interrupts();
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    log::set_logger(&LOGGER).expect("ee");
    log::set_max_level(log::LevelFilter::Trace);
    TERMINAL.lock().clear();

    init();

    trace!("trace: test trace");
    debug!("debug: test debug ({}, {})", 67, 67);
    info!("info: test info");
    warn!("warn: test warn");
    error!("error: test err");

    unsafe {
        let ptr = VolatilePtr::new(NonNull::new(0x0000_8000_0000_0000 as *mut u8).unwrap());
        ptr.write(67);
    };



    debug!("e no crash it work");

    hlt();

    loop {}
}