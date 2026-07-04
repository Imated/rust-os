use core::fmt::Write;
use log::{Level, Log, Metadata, Record};
use crate::graphics::framebuffer::Framebuffer;
use crate::types::Color;
use noto_sans_mono_bitmap::{get_raster, get_raster_width, FontWeight, RasterHeight};
use spin::{MutexGuard, Spin};
use crate::{FRAMEBUFFER, TERMINAL};

pub struct Terminal {
    row: u32,
    col: u32,
    width_in_chars: u32,
    height_in_chars: u32,
    char_width: u32,
    char_height: u32,
    pub color: Color,
}

impl Terminal {
    pub fn new() -> Self {
        let framebuffer_guard = FRAMEBUFFER.lock();

        Self {
            row: 0,
            col: 0,
            width_in_chars: framebuffer_guard.width / get_raster_width(FontWeight::Regular, RasterHeight::Size20) as u32,
            height_in_chars: framebuffer_guard.height / 16,
            char_width: get_raster_width(FontWeight::Regular, RasterHeight::Size20) as u32,
            char_height: 20,
            color: Color::WHITE,
        }
    }

    pub fn put_str(&mut self, str: &str) {
        let mut framebuffer_guard = FRAMEBUFFER.lock();

        for byte in str.bytes() {
            match byte {
                b'\n' => {
                    self.col = self.width_in_chars - 1;
                    self.inc_cursor(1, &mut framebuffer_guard);
                },
                b'\t' => self.inc_cursor(4, &mut framebuffer_guard),
                _ => {
                    self.put_char_at(byte.into(), self.col, self.row, &mut framebuffer_guard);
                    self.inc_cursor(1, &mut framebuffer_guard);
                },
            }
        }
    }

    fn put_char_at(&mut self, c: char, x: u32, y: u32, framebuffer: &mut MutexGuard<'_, Framebuffer, Spin>) {
        let Some(raster) = get_raster(c, FontWeight::Regular, RasterHeight::Size20) else {
            return;
        };

        for (row_i, row) in raster.raster().iter().enumerate() {
            let py = y * self.char_height + row_i as u32;
            if py >= framebuffer.height  {
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

    fn inc_cursor(&mut self, amount: u32, framebuffer: &mut MutexGuard<'_, Framebuffer, Spin>) {
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
        }
        else {
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
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut terminal = TERMINAL.lock();
        let color = match record.level() {
            Level::Error => Color::RED,
            Level::Warn => Color::YELLOW,
            Level::Info => Color::LIGHT_GRAY,
            Level::Debug => Color::LIGHT_GREEN,
            Level::Trace => Color::WHITE,
        };

        terminal.color = color;
        let _ = write!(terminal, "[{}] {}\n", record.level(), record.args());
    }

    fn flush(&self) {

    }
}
