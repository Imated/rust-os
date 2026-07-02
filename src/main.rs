#![feature(formatting_options)]
#![no_std]
#![no_main]

use core::ffi::c_char;
use core::fmt;
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::null_mut;
use limine::request::FramebufferRequest;

#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER: FramebufferRequest = FramebufferRequest::new();

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    if let Some(resp) = FRAMEBUFFER.response()
        && let Some(fb) = resp.framebuffers().first()
    {
        for x in 0..fb.width {
            for y in 0..fb.height {
                unsafe {
                    (fb.address() as *mut i32).offset((y * (fb.pitch / 4) + x) as isize).write(0x0000FF);
                }
            }
        }
    }

    loop {}
}