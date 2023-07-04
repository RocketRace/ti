//! Module responsible for formatting black & white bitmaps into Unicode braille characters.
//!
//! See the [`Cell`] documentation for more.

/// The unicode scalar value for the first ("empty") braille codepoint.
pub const BRAILLE_BASE_CODEPOINT: u32 = 0x2800;
/// The number of bytes required to encode a braille unicode character into utf-8. This is a constant value,
/// because the characters have codepoints between `U+0800` and `U+FFFF`.
pub const BRAILLE_UTF8_BYTES: usize = 3;

/// An offset into a unicode block with its bits permuted. More specifically,
/// its bits follow the following format:
/// ```txt
/// 0 1
/// 2 3
/// 4 5
/// 6 7
/// ```
///
/// That is, the top left pixel of a cell is stored into the 0th bit, the top right pixel is the 1st
/// bit, and so on.
///
/// On the contrary, the Unicode specification for braille characters provides the following format:
/// ```txt
/// 0 3
/// 1 4
/// 2 5
/// 6 7
/// ```
/// That is, the braille character with the top left and top right dots set is encoded as an 8-bit offset
/// from [`BRAILLE_BASE_CODEPOINT`] with the 0th and 3rd bits set, i.e. `0b1001`.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
#[repr(transparent)]
pub struct Cell {
    /// The internal storage bits.
    pub bits: u8,
}

impl Cell {
    /// A cell is exactly 2 pixels wide, since it consists of one braille character.
    pub const PIXEL_WIDTH: usize = 2;
    /// A cell is exactly 4 pixels tall, since it consists of one braille character.
    pub const PIXEL_HEIGHT: usize = 4;

    /// Create a new cell with the specified bits.
    pub fn new(bits: u8) -> Self {
        Self { bits }
    }

    /// Computes the Unicode codepoint offset format of the braille character.
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

    /// Encodes the cell as a sequence of UTF-8 bytes representing
    /// its braille encoded character.
    pub fn to_braille_utf8(self) -> [u8; BRAILLE_UTF8_BYTES] {
        // The braille character block contains only 3-byte utf-8 characters.
        // Since we can assert the range of code points here, the optimizer can take some shortcuts and,
        // for example, elide this `unwrap()` operation.
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

    #[test]
    fn correct_braille() {
        assert_eq!(Cell::new(0).to_braille_utf8(), [226, 160, 128]);
        assert_eq!(Cell::new(1).to_braille_utf8(), [226, 160, 129]);
        assert_eq!(Cell::new(2).to_braille_utf8(), [226, 160, 136]);
        assert_eq!(Cell::new(4).to_braille_utf8(), [226, 160, 130]);
        assert_eq!(Cell::new(8).to_braille_utf8(), [226, 160, 144]);
        assert_eq!(Cell::new(16).to_braille_utf8(), [226, 160, 132]);
        assert_eq!(Cell::new(32).to_braille_utf8(), [226, 160, 160]);
        assert_eq!(Cell::new(64).to_braille_utf8(), [226, 161, 128]);
        assert_eq!(Cell::new(128).to_braille_utf8(), [226, 162, 128]);
    }
}
