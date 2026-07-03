use crate::types::Color;
use bytemuck::cast;
use core::convert::Into;
use core::slice::from_raw_parts_mut;
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};

pub struct Framebuffer<'a> {
    front_buffer: &'a mut [Color],
    pixels_per_scanline: u64,
    width: u32,
    height: u32,
}

impl<'a> Framebuffer<'a> {
    pub unsafe fn new(framebuffer: &limine::framebuffer::Framebuffer) -> Self { unsafe {
        Self {
            front_buffer: from_raw_parts_mut(framebuffer.address() as *mut Color, (framebuffer.width * framebuffer.height) as usize),
            pixels_per_scanline: framebuffer.pitch / 4,
            width: framebuffer.width as u32,
            height: framebuffer.height as u32,
        }
    }}

    pub fn put_char_at(&mut self, c: char, x: u32, y: u32, color: Color) {
        let Some(raster) = get_raster(c, FontWeight::Regular, RasterHeight::Size16) else {
            return;
        };

        for (row_i, row) in raster.raster().iter().enumerate() {
            let py = y + row_i as u32;
            if py >= self.height  {
                break;
            }

            for (col_i, intensity) in row.iter().enumerate() {
                let px = x + col_i as u32;
                if px >= self.width {
                    continue;
                }

                let mut blended = Color {
                    r: ((color.r as u32 * *intensity as u32) / 255) as u8,
                    g: ((color.g as u32 * *intensity as u32) / 255) as u8,
                    b: ((color.b as u32 * *intensity as u32) / 255) as u8,
                    _pad: 0,
                };

                if blended == Color::BLACK {
                    blended = Color::BACKGROUND_COLOR;
                }

                self.set_pixel(px.into(), py.into(), blended);
            }
        }
    }

    pub fn set_pixel(&mut self, x: u64, y: u64, color: Color) {
        self.front_buffer[(y * self.pixels_per_scanline + x) as usize] = color;
    }

    pub fn clear(&mut self, color: Color) {
        self.front_buffer.fill(color);
    }
}