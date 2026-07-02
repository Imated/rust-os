#![feature(formatting_options)]
#![no_std]
#![no_main]

mod framebuffer;
pub mod types;

use core::panic::PanicInfo;
use limine::request::FramebufferRequest;
use crate::framebuffer::Framebuffer;
use crate::types::Color;

#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! { unsafe {
    if let Some(resp) = FRAMEBUFFER.response()
        && let Some(&fb) = resp.framebuffers().first()
    {
        let framebuffer = Framebuffer::new(fb);
        framebuffer.clear(Color::BACKGROUND_COLOR);
        framebuffer.put_char_at('e', 0, 0, Color::WHITE);
    }

    loop {}
}}