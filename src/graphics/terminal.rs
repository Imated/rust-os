use crate::graphics::framebuffer::Framebuffer;
use crate::types::Color;
use core::fmt::Write;
use log::{Level, Log, Metadata, Record};
use noto_sans_mono_bitmap::{FontWeight, RasterHeight, get_raster, get_raster_width};
use spin::{Mutex, once::Once};
use x86_64::instructions::interrupts::without_interrupts;

pub static TERMINAL: Once<Mutex<Terminal>> = Once::new();
pub struct Terminal {
    row: u32,
    col: u32,
    width_in_chars: u32,
    height_in_chars: u32,
    char_width: u32,
    char_height: u32,
    pub color: Color,
}

impl Default for Terminal {
    fn default() -> Self {
        Framebuffer::with(|fb| Self {
            row: 0,
            col: 0,
            width_in_chars: fb.width
                / get_raster_width(FontWeight::Regular, RasterHeight::Size20) as u32,
            height_in_chars: fb.height / 20,
            char_width: get_raster_width(FontWeight::Regular, RasterHeight::Size20) as u32,
            char_height: 20,
            color: Color::WHITE,
        })
    }
}

impl Terminal {
    pub fn with<R>(f: impl FnOnce(&mut Terminal) -> R) -> R {
        let term = unsafe { &mut TERMINAL.get_unchecked().lock() };
        without_interrupts(|| f(term))
    }

    pub unsafe fn with_forced<R>(f: impl FnOnce(&mut Terminal) -> R) -> R {
        let term = unsafe { &mut TERMINAL.get_unchecked() };
        unsafe { term.force_unlock() };
        without_interrupts(|| f(&mut term.lock()))
    }

    pub fn put_str(&mut self, str: &str) {
        Framebuffer::with(|fb| {
            for byte in str.bytes() {
                match byte {
                    b'\n' => {
                        self.col = self.width_in_chars - 1;
                        self.inc_cursor(1, fb);
                    }
                    b'\t' => self.inc_cursor(4, fb),
                    b'\x08' => {
                        // backspace
                        self.dec_cursor(1);
                        self.put_char_at(' ', self.col, self.row, fb);
                    }
                    _ => {
                        self.put_char_at(byte.into(), self.col, self.row, fb);
                        self.inc_cursor(1, fb);
                    }
                }
            }
        })
    }

    pub fn clear(&mut self) {
        Framebuffer::with(|fb| {
            fb.clear(Color::BACKGROUND_COLOR);
            self.col = 0;
            self.row = 0;
        })
    }

    fn put_char_at(&mut self, c: char, x: u32, y: u32, framebuffer: &mut Framebuffer) {
        let Some(raster) = get_raster(c, FontWeight::Regular, RasterHeight::Size20) else {
            return;
        };

        for (row_i, row) in raster.raster().iter().enumerate() {
            let py = y * self.char_height + row_i as u32;
            if py >= framebuffer.height {
                break;
            }

            for (col_i, &intensity) in row.iter().enumerate() {
                let px = x * self.char_width + col_i as u32;
                if px >= framebuffer.width {
                    continue;
                }

                let mut blended = Color {
                    r: ((self.color.r as u32 * intensity as u32) / 255) as u8,
                    g: ((self.color.g as u32 * intensity as u32) / 255) as u8,
                    b: ((self.color.b as u32 * intensity as u32) / 255) as u8,
                    _pad: 0,
                };

                if blended == Color::BLACK {
                    blended = Color::BACKGROUND_COLOR;
                }

                framebuffer.set_pixel(px, py, blended);
            }
        }
    }

    fn inc_cursor(&mut self, amount: u32, framebuffer: &mut Framebuffer) {
        self.col += amount;
        if self.col >= self.width_in_chars {
            self.col = 0;
            self.row += 1;

            if self.row >= self.height_in_chars {
                framebuffer.scroll(1);
                self.row = self.height_in_chars - 1;
            }
        }
    }

    fn dec_cursor(&mut self, amount: u32) {
        if (self.col as i32 - amount as i32) < 0 {
            self.col = self.width_in_chars - 1;
            if self.row > 0 {
                self.row -= 1;
            }
        } else {
            self.col -= amount;
        }
    }
}

impl Write for Terminal {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.put_str(s);
        Ok(())
    }
}

pub struct TerminalLogger;

impl Log for TerminalLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        Terminal::with(|term| {
            let color = match record.level() {
                Level::Error => Color::RED,
                Level::Warn => Color::YELLOW,
                Level::Info => Color::LIGHT_GRAY,
                Level::Debug => Color::LIGHT_GREEN,
                Level::Trace => Color::WHITE,
            };

            term.color = color;
            let _ = writeln!(term, "[{}] {}", record.level(), record.args());
        });
    }

    fn flush(&self) {}
}
