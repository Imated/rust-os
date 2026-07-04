use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Ord, PartialOrd, Eq, PartialEq)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
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

    pub const RED: Color = Color {
        r: 0xAA,
        g: 0x00,
        b: 0x00,
        _pad: 0,
    };

    pub const YELLOW: Color = Color {
        r: 0xFF,
        g: 0xFF,
        b: 0x55,
        _pad: 0,
    };

    pub const LIGHT_GRAY: Color = Color {
        r: 0xAA,
        g: 0xAA,
        b: 0xAA,
        _pad: 0,
    };

    pub const LIGHT_GREEN: Color = Color {
        r: 0x55,
        g: 0xFF,
        b: 0x55,
        _pad: 0,
    };
}
