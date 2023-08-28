//! A [`Cell`] is the atomic unit of rendering, and represents a bitmap drawn to the terminal.
//!
//! See the [`Cell`] documentation for more.

use std::fmt::{Debug, Display};

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
#[derive(Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Cell {
    /// The internal storage bits.
    pub bits: u8,
}

impl Debug for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Cell::from_braille")
            .field(&std::str::from_utf8(&self.to_braille_utf8()).unwrap())
            .finish()
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(std::str::from_utf8(&self.to_braille_utf8()).unwrap())
    }
}

/// Represents the kind of alignment that a cell may be in. Because the pixel grid is higher resolution
/// than the cell grid, i.e. a cell contains more than 1 pixel, trying to draw cells at precise pixel
/// coordinates will sometimes cause the cell to be offset from the grid.
#[derive(Debug, Clone, Copy)]
pub enum OffsetCell {
    /// The cell is aligned to the grid.
    Aligned { cell: Cell },
    /// The cell is horizontally misaligned, i.e. occupies space in two horizontally adjacent cells.
    Horizontal { left: Cell, right: Cell },
    /// The cell is vertically misaligned, i.e. occupies space in two vetically adjacent cells.
    Vertical { up: Cell, down: Cell },
    /// The cell is both horizontally and vertically misaligned, i.e. occupies space in four adjacent cells,
    /// around a corner.
    Corner {
        ul: Cell,
        ur: Cell,
        dl: Cell,
        dr: Cell,
    },
}

/// A cell is exactly 2 pixels wide, since it consists of one braille character.
pub const PIXEL_WIDTH: u8 = 2;
/// A cell is exactly 4 pixels tall, since it consists of one braille character.
pub const PIXEL_HEIGHT: u8 = 4;
/// A cell has exactly 2 * 4 = 8 positions.
pub const PIXEL_OFFSETS: u8 = PIXEL_WIDTH * PIXEL_HEIGHT;

impl Cell {
    /// Creates a new empty cell.
    pub const fn empty() -> Self {
        Self::new(0)
    }

    /// Returns true whenever the cell is empty.
    pub const fn is_empty(&self) -> bool {
        self.bits == 0
    }

    /// Creates a new full cell.
    pub const fn full() -> Self {
        Self::new(0xff)
    }

    /// Returns true whenever the cell is full.
    pub const fn is_full(&self) -> bool {
        self.bits == 0xff
    }

    /// Create a new cell with the specified internal bits.
    pub const fn new(bits: u8) -> Self {
        Self { bits }
    }

    /// Create a new cell with a single bit set in the specified position.
    ///
    /// Returns `Some(Self)` when the bit positions fit within a single cell,
    /// `None` otherwise.
    pub const fn from_bit_position(x: u8, y: u8) -> Option<Self> {
        if x < PIXEL_WIDTH && y < PIXEL_HEIGHT {
            Some(Self::new(1 << (PIXEL_WIDTH * y + x)))
        } else {
            None
        }
    }

    /// Computes the Unicode codepoint offset format of the braille character.
    pub const fn braille_offset(self) -> u8 {
        (self.bits & 0b1110_0001)
            | ((self.bits & 0b10) << 2)
            | ((self.bits & 0b100) >> 1)
            | ((self.bits & 0b1000) << 1)
            | ((self.bits & 0b10000) >> 2)
    }

    /// Creates a cell from its corresponding braille character.
    /// If the given character is not a braille character, returns `None`.
    pub fn from_braille(c: char) -> Option<Self> {
        let codepoint = c as u32;
        if (BRAILLE_BASE_CODEPOINT..BRAILLE_BASE_CODEPOINT + 256).contains(&codepoint) {
            let offset = (codepoint - BRAILLE_BASE_CODEPOINT) as u8;
            let bits = (offset & 0b1110_0001)
                | ((offset & 0b10) << 1)
                | ((offset & 0b100) << 2)
                | ((offset & 0b1000) >> 2)
                | ((offset & 0b10000) >> 1);
            Some(Self::new(bits))
        } else {
            None
        }
    }

    /// Returns the braille character represented by this cell.
    pub fn to_braille_char(self) -> char {
        // Codepoints are always valid
        char::from_u32(BRAILLE_BASE_CODEPOINT + self.braille_offset() as u32).unwrap()
    }

    /// Encodes the cell as a sequence of UTF-8 bytes representing
    /// its Braille-encoded character.
    pub fn to_braille_utf8(self) -> [u8; BRAILLE_UTF8_BYTES] {
        let c = self.to_braille_char();
        let mut b = [0; BRAILLE_UTF8_BYTES];
        // The braille character block contains only 3-byte utf-8 characters, so this never panics.
        c.encode_utf8(&mut b);
        b
    }

    const fn compute_x_offset(self, x_offset: u8) -> (Cell, Cell) {
        let mask = 0b0101_0101;
        let first = (self.bits & mask) << (PIXEL_WIDTH - x_offset);
        let second = (self.bits & !mask) >> x_offset;
        (Cell::new(first), Cell::new(second))
    }

    const fn compute_y_offset(self, y_offset: u8) -> (Cell, Cell) {
        let y_offset = PIXEL_HEIGHT - y_offset;
        let stride = PIXEL_WIDTH;
        let mask = (1 << (stride * y_offset)) - 1;
        let first = (self.bits & mask) << (stride * (PIXEL_HEIGHT - y_offset));
        let second = (self.bits & !mask) >> (stride * y_offset);
        (Cell::new(first), Cell::new(second))
    }

    /// Computes the alignment that this cell will end up in as a result of the given pixel offsets.
    /// The parameters `x_offset` and `y_offset` are taken modulo the cell's internal pixel coordinates,
    /// i.e. [`PIXEL_WIDTH`] and [`PIXEL_HEIGHT`].
    ///
    /// Returns an [`OffsetCell`] representing the new pixel data, in all the cells that it occupies space in.
    ///
    /// All offsets are taken as nonnegative.
    pub const fn with_offset(self, x_offset: u8, y_offset: u8) -> OffsetCell {
        let x_offset = x_offset % PIXEL_WIDTH;
        let y_offset = y_offset % PIXEL_HEIGHT;
        match (x_offset, y_offset) {
            (0, 0) => OffsetCell::Aligned { cell: self },
            (1, 0) => {
                let (left, right) = self.compute_x_offset(x_offset);
                OffsetCell::Horizontal { left, right }
            }
            (0, _) => {
                let (up, down) = self.compute_y_offset(y_offset);
                OffsetCell::Vertical { up, down }
            }
            (1, _) => {
                let (top, bottom) = self.compute_y_offset(y_offset);
                let (ul, ur) = top.compute_x_offset(x_offset);
                let (dl, dr) = bottom.compute_x_offset(x_offset);
                OffsetCell::Corner { ul, ur, dl, dr }
            }
            _ => unreachable!(),
        }
    }
}

impl std::ops::BitOr for Cell {
    type Output = Cell;

    /// Creates a new cell with pixels set in either `self` or `rhs`.
    fn bitor(self, rhs: Self) -> Self::Output {
        Cell::new(self.bits | rhs.bits)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn unique_offset() {
        let map: HashSet<_> = (0u8..=255).map(|n| Cell::new(n).braille_offset()).collect();
        assert_eq!(map.len(), 256);
    }

    #[test]
    fn braille_round_trip() {
        for i in 0..=255u8 {
            let c = char::from_u32(BRAILLE_BASE_CODEPOINT + i as u32).unwrap();
            assert_eq!(i, Cell::from_braille(c).unwrap().braille_offset());
            assert_eq!(
                c,
                std::str::from_utf8(&Cell::from_braille(c).unwrap().to_braille_utf8())
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap()
            );
        }
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
