use crate::types::Color;
use bytemuck::cast;
use core::convert::Into;
use core::slice::from_raw_parts_mut;
use noto_sans_mono_bitmap::{get_raster, FontWeight, RasterHeight};

pub struct Framebuffer {
    front_buffer: *mut u32,
    pixels_per_scanline: u64,
    width: u32,
    height: u32,
}

impl Framebuffer {
    pub fn new(framebuffer: &limine::framebuffer::Framebuffer) -> Self {
        Self {
            front_buffer: framebuffer.address() as *mut u32,
            pixels_per_scanline: framebuffer.pitch / 4,
            width: framebuffer.width as u32,
            height: framebuffer.height as u32,
        }
    }

    pub unsafe fn put_char_at(&self, c: char, x: u32, y: u32, color: Color) {
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

    pub unsafe fn set_pixel(&self, x: u64, y: u64, color: Color) {
        self.front_buffer.offset((y * self.pixels_per_scanline + x) as isize).write(cast(color));
    }

    pub unsafe fn clear(&self, color: Color) {
        let slice = from_raw_parts_mut(self.front_buffer, (self.width * self.height) as usize);
        slice.fill(cast(color));
    }
}