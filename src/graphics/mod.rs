use spin::Mutex;

use crate::graphics::{
    framebuffer::{FRAMEBUFFER, FRAMEBUFFER_REQUEST, Framebuffer},
    terminal::{TERMINAL, Terminal},
};

pub mod framebuffer;
pub mod terminal;

pub fn init_graphics() {
    FRAMEBUFFER.call_once(|| {
        Mutex::new(Framebuffer::new(
            FRAMEBUFFER_REQUEST
                .response()
                .unwrap()
                .framebuffers()
                .first()
                .unwrap(),
        ))
    });
    TERMINAL.call_once(|| Mutex::new(Terminal::default()));

    Terminal::with(|term| {
        term.clear();
    });
}
