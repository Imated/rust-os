use crate::types::Color;
use core::slice::from_raw_parts_mut;
use limine::request::FramebufferRequest;
use spin::{Mutex, Once};
use x86_64::instructions::interrupts::without_interrupts;

#[unsafe(link_section = ".requests")]
pub static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();
pub static FRAMEBUFFER: Once<Mutex<Framebuffer>> = Once::new();

pub struct Framebuffer {
    front_buffer: &'static mut [Color],
    pixels_per_scanline: u32,
    pub width: u32,
    pub height: u32,
}

impl Framebuffer {
    pub fn new(framebuffer: &limine::framebuffer::Framebuffer) -> Self {
        unsafe {
            let front_buffer = from_raw_parts_mut(
                framebuffer.address() as *mut Color,
                (framebuffer.width * framebuffer.height) as usize,
            );

            Self {
                front_buffer,
                pixels_per_scanline: framebuffer.pitch as u32 / size_of::<Color>() as u32,
                width: framebuffer.width as u32,
                height: framebuffer.height as u32,
            }
        }
    }

    pub fn with<R>(f: impl FnOnce(&mut Framebuffer) -> R) -> R {
        let fb = unsafe { &mut FRAMEBUFFER.get_unchecked().lock() };
        without_interrupts(|| f(fb))
    }

    pub unsafe fn with_forced<R>(f: impl FnOnce(&mut Framebuffer) -> R) -> R {
        let fb = unsafe { &mut FRAMEBUFFER.get_unchecked() };
        unsafe { fb.force_unlock() };
        without_interrupts(|| f(&mut fb.lock()))
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        self.front_buffer[(y * self.pixels_per_scanline + x) as usize] = color;
    }

    pub fn clear(&mut self, color: Color) {
        self.front_buffer.fill(color);
    }

    pub fn scroll(&mut self, lines: u32) {
        let scroll_px = 20 * lines;
        if scroll_px >= self.height {
            self.clear(Color::BACKGROUND_COLOR);
            return;
        }

        let copy_len = (self.height - scroll_px) * self.pixels_per_scanline;
        let src_start = scroll_px * self.pixels_per_scanline;

        self.front_buffer
            .copy_within(src_start as usize..(src_start + copy_len) as usize, 0);
        self.front_buffer
            [copy_len as usize..(copy_len + scroll_px * self.pixels_per_scanline) as usize]
            .fill(Color::BACKGROUND_COLOR);
    }
}
