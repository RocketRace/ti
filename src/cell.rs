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

/// Represents the kind of alignment that a cell may be in. Because the pixel grid is higher resolution
/// than the cell grid, i.e. a cell contains more than 1 pixel, trying to draw cells at precise pixel
/// coordinates will sometimes cause the cell to be offset from the grid.
#[derive(Debug, Clone, Copy)]
pub enum OffsetCell {
    /// The cell is aligned to the grid.
    Aligned(Cell),
    /// The cell is horizontally misaligned, i.e. occupies space in two horizontally adjacent cells.
    Horizontal(Cell, Cell),
    /// The cell is vertically misaligned, i.e. occupies space in two vetically adjacent cells.
    Vertical(Cell, Cell),
    /// The cell is both horizontally and vertically misaligned, i.e. occupies space in four adjacent cells,
    /// around a corner.
    Corner(Cell, Cell, Cell, Cell),
}

impl Cell {
    /// A cell is exactly 2 pixels wide, since it consists of one braille character.
    pub const PIXEL_WIDTH: usize = 2;
    /// A cell is exactly 4 pixels tall, since it consists of one braille character.
    pub const PIXEL_HEIGHT: usize = 4;

    /// Create a new cell with the specified internal bits.
    pub fn new(bits: u8) -> Self {
        Self { bits }
    }

    /// Computes the Unicode codepoint offset format of the braille character.
    pub fn braille_offset(self) -> u8 {
        (self.bits & 0b11100001)
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
            let bits = (offset & 0b11100001)
                | ((offset & 0b10) << 1)
                | ((offset & 0b100) << 2)
                | ((offset & 0b1000) >> 2)
                | ((offset & 0b10000) >> 1);
            Some(Self::new(bits))
        } else {
            None
        }
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

    fn compute_x_offset(self, x_offset: usize) -> (Cell, Cell) {
        let mask = 0b01010101;
        let first = (self.bits & mask) << (Cell::PIXEL_WIDTH - x_offset);
        let second = (self.bits & !mask) >> x_offset;
        (Cell::new(first), Cell::new(second))
    }

    fn compute_y_offset(self, y_offset: usize) -> (Cell, Cell) {
        let y_offset = Cell::PIXEL_HEIGHT - y_offset;
        let stride = Cell::PIXEL_WIDTH;
        let mask = (1 << (stride * y_offset)) - 1;
        let first = (self.bits & mask) << (stride * (Cell::PIXEL_HEIGHT - y_offset));
        let second = (self.bits & !mask) >> (stride * y_offset);
        (Cell::new(first), Cell::new(second))
    }

    /// Computes the alignment that this cell will end up in as a result of the given pixel offsets.
    /// The parameters `x_offset` and `y_offset` are taken modulo the cell's internal pixel coordinates,
    /// i.e. [`Cell::PIXEL_WIDTH`] and [`Cell::PIXEL_HEIGHT`].
    ///
    /// Returns an [`Offset`] representing the new pixel data, in all the cells that it occupies space in.
    ///
    /// All offsets are taken as nonnegative.
    pub fn compute_offset(self, x_offset: usize, y_offset: usize) -> OffsetCell {
        let x_offset = x_offset % Cell::PIXEL_WIDTH;
        let y_offset = y_offset % Cell::PIXEL_HEIGHT;
        match (x_offset, y_offset) {
            (0, 0) => OffsetCell::Aligned(self),
            (1, 0) => {
                let (left, right) = self.compute_x_offset(x_offset);
                OffsetCell::Horizontal(left, right)
            }
            (0, _) => {
                let (top, bottom) = self.compute_y_offset(y_offset);
                OffsetCell::Vertical(top, bottom)
            }
            (1, _) => {
                let (top, bottom) = self.compute_y_offset(y_offset);
                let (top_left, top_right) = top.compute_x_offset(x_offset);
                let (bottom_left, bottom_right) = bottom.compute_x_offset(x_offset);
                OffsetCell::Corner(top_left, top_right, bottom_left, bottom_right)
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
        assert_eq!(map.len(), 256)
    }

    #[test]
    fn braille_round_trip() {
        for i in 0..=255 {
            let c = char::from_u32(BRAILLE_BASE_CODEPOINT + i).unwrap();
            assert_eq!(i as u8, Cell::from_braille(c).unwrap().braille_offset());
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
