//! Module responsible for formatting black & white bitmaps into Unicode braille characters

use crate::color::Color;

pub const BRAILLE_BASE_CODEPOINT: u32 = 0x2800;
pub const BRAILLE_UTF8_BYTES: usize = 3;
// Pixel format
// 0 1
// 2 3
// 4 5
// 6 7
//
// Offset format
// 0 3
// 1 4
// 2 5
// 6 7

#[derive(Clone, Copy, Default)]
pub struct Cell {
    pub bits: u8,
    pub fg: Color,
    pub bg: Color,
}

impl Cell {
    pub const PIXEL_WIDTH: usize = 2;
    pub const PIXEL_HEIGHT: usize = 4;

    pub fn new(bits: u8) -> Self {
        Self {
            bits,
            fg: Color::None,
            bg: Color::None,
        }
    }

    pub const fn braille_offset(self) -> u8 {
        (self.bits & 0b11100001)
            | ((self.bits & 0b10) << 2)
            | ((self.bits & 0b100) >> 1)
            | ((self.bits & 0b1000) << 1)
            | ((self.bits & 0b10000) >> 2)
        // inverse operation:
        //   (offset & 0b11100001)
        // | ((offset & 0b10) << 1)
        // | ((offset & 0b100) << 2)
        // | ((offset & 0b1000) >> 2)
        // | ((offset & 0b10000) >> 1)
    }

    // The braille character block contains only 3-byte utf-8 characters.
    // Since we can assert the range of code points here, the optimizer can take some shortcuts.
    pub fn to_braille_utf8(self) -> [u8; BRAILLE_UTF8_BYTES] {
        let c = char::from_u32(BRAILLE_BASE_CODEPOINT + self.braille_offset() as u32).unwrap();
        let mut b = [0; BRAILLE_UTF8_BYTES];
        c.encode_utf8(&mut b);
        b
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn unique_offset() {
        let map: HashSet<_> = (0u8..=255).map(|n| Cell::new(n).braille_offset()).collect();
        assert_eq!(map.len(), 256)
    }
}
