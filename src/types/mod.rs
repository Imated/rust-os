use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Ord, PartialOrd, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub _pad: u8,
}

impl Color {
    pub const BACKGROUND_COLOR: Color = Color {
        r: 0x0F,
        g: 0x0F,
        b: 0x0F,
        _pad: 0,
    };

    pub const WHITE: Color = Color {
        r: 0xDF,
        g: 0xDF,
        b: 0xDF,
        _pad: 0,
    };

    pub const BLACK: Color = Color {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        _pad: 0,
    };
}