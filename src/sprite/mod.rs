//! Module for manipulating [`Sprite`]s, i.e. rectangular collections of [`Cell`]s with associated color information.
#[cfg(feature = "images")]
mod images;
use std::array;

#[cfg(feature = "images")]
pub use images::*;

use smallvec::{smallvec, SmallVec};

use crate::{
    cell::{Cell, OffsetCell, BRAILLE_UTF8_BYTES, PIXEL_HEIGHT, PIXEL_OFFSETS, PIXEL_WIDTH},
    color::{Color, ColoredCell},
    units::{cell_length, from_index, index, offset_px, pos_components, px_offset},
};

/// Stack allocation size for each sprite's cell data
const SPRITE_STACK_SIZE: usize = 64;

/// A sprite made up of a contiguous rectangular region of cells.
/// The cells may be colored.
#[derive(Debug, Clone)]
pub struct Sprite {
    /// Contains precomputed pixel offsets for the sprite. Since sprites are typically drawn
    /// many times, we compute and store the offsets upfront.
    ///
    /// The heap pointers are separate because the size of a sprite (in cells) can vary
    /// depending on its offset. Still not ideal to have max 8 heap allocations per sprite.
    pub offsets: [SpriteData; PIXEL_OFFSETS as usize],
    width: u16,
    height: u16,
    /// The draw priority of the sprite. Higher priority sprites will be drawn on top of lower priority ones.
    pub priority: u16,
}

type SpriteData = SmallVec<[ColoredCell; SPRITE_STACK_SIZE]>;

impl Sprite {
    /// Create a new empty [`Sprite`] with the given dimensions.
    /// The width and height parameters are in terms of cells.
    /// This is because there is no way to distinguish between an empty pixel that's inside
    /// the sprite and an empty pixel that's outside the sprite.
    pub fn empty(width_cells: u16, height_cells: u16, priority: u16) -> Self {
        // note: don't use the new() constructor, it calls empty()
        Self {
            offsets: array::from_fn(
                |_| smallvec![ColoredCell::default(); cell_length(width_cells, height_cells)],
            ),
            width: width_cells,
            height: height_cells,
            priority,
        }
    }
    /// Create a new filled rectangular [`Sprite`] with the given color.
    ///
    /// The width and height parameters are in terms of pixels.
    pub fn rectangle(width: u16, height: u16, color: Option<Color>, priority: u16) -> Self {
        let ((cell_x, px_x), (cell_y, px_y)) = pos_components(width, height);
        let width_cells = cell_x + if px_x == 0 { 0 } else { 1 };
        let height_cells = cell_y + if px_y == 0 { 0 } else { 1 };

        // Create an oversized rectangle and then crop the bottom/right borders
        let mut data = smallvec![ColoredCell::new(Cell::full(), color); cell_length(width_cells, height_cells)];
        if width_cells > 0 && height_cells > 0 {
            if px_x != 0 {
                for i in 0..height_cells {
                    // Type annotations needed when indexing these for some reason
                    let colored: &mut ColoredCell =
                        &mut data[index(width_cells - 1, i, width_cells)];
                    colored.cell = Cell::from_braille('⡇').unwrap();
                }
            }
            if px_y != 0 {
                for i in 0..width_cells {
                    let colored: &mut ColoredCell =
                        &mut data[index(i, height_cells - 1, width_cells)];
                    let levels = [
                        Cell::from_braille('⠉').unwrap(),
                        Cell::from_braille('⠛').unwrap(),
                        Cell::from_braille('⠿').unwrap(),
                    ];
                    colored.cell = levels[px_y as usize - 1];
                }
                // oh and don't forget the corner
                if px_x != 0 {
                    let last = data.len() - 1;
                    let colored: &mut ColoredCell = &mut data[last];
                    let levels = [
                        Cell::from_braille('⠁').unwrap(),
                        Cell::from_braille('⠃').unwrap(),
                        Cell::from_braille('⠇').unwrap(),
                    ];
                    colored.cell = levels[px_y as usize - 1];
                }
            }
        }

        dbg!(&data, width_cells, height_cells, priority);
        Self::new(data, width_cells, height_cells, priority)
    }

    /// Computes the array index of the cell at position (x, y) with the given sprite offset.
    pub const fn index(&self, x: u16, y: u16, offset: u8) -> usize {
        let (width, _) = self.offset_size(offset);
        index(x, y, width)
    }

    /// Computes the position (x, y) of a cell at a specified array index with the given sprite offset.
    pub const fn from_index(&self, i: usize, offset: u8) -> (u16, u16) {
        let (width, _) = self.offset_size(offset);
        from_index(i, width)
    }

    /// Creates a sprite from raw data.
    pub fn new(data: SpriteData, width_cells: u16, height_cells: u16, priority: u16) -> Self {
        let mut this = Self::empty(width_cells, height_cells, priority);
        for dy in 0..PIXEL_HEIGHT {
            for dx in 0..PIXEL_WIDTH {
                let offset = px_offset(dx, dy);
                let (new_width, new_height) = this.offset_size(offset);
                let new_size = cell_length(new_width, new_height);
                this.offsets[offset as usize].resize(new_size, ColoredCell::default());

                for y in 0..height_cells {
                    for x in 0..width_cells {
                        let i_orig = this.index(x, y, 0);
                        let i_ul = this.index(x, y, offset);
                        let i_ur = this.index(x + 1, y, offset);
                        let i_dl = this.index(x, y + 1, offset);
                        let i_dr = this.index(x + 1, y + 1, offset);
                        let buf = &mut this.offsets[offset as usize];
                        let ColoredCell { cell, color } = data[i_orig];

                        match cell.with_offset(dx, dy) {
                            OffsetCell::Aligned { cell } => {
                                buf[i_ul].merge_cell(cell, color);
                            }
                            OffsetCell::Horizontal { left, right } => {
                                buf[i_ul].merge_cell(left, color);
                                buf[i_ur].merge_cell(right, color);
                            }
                            OffsetCell::Vertical { up, down } => {
                                buf[i_ul].merge_cell(up, color);
                                buf[i_dl].merge_cell(down, color);
                            }
                            OffsetCell::Corner { ul, ur, dl, dr } => {
                                buf[i_ul].merge_cell(ul, color);
                                buf[i_ur].merge_cell(ur, color);
                                buf[i_dl].merge_cell(dl, color);
                                buf[i_dr].merge_cell(dr, color);
                            }
                        }
                    }
                }
            }
        }
        this
    }

    /// Creates a [`Sprite`] from the given sequence of braille strings.
    /// Each element of the parameter slice is interpreted as a row of the sprite.
    ///
    /// Returns None if any characters in the string are non-braille, or if the rows
    /// are different lengths.
    pub fn from_braille_string(s: &[&str], color: Option<Color>, priority: u16) -> Option<Self> {
        if s.is_empty() {
            Some(Sprite::empty(0, 0, priority))
        } else {
            let width_bytes = s[0].len();
            if s.iter().any(|&r| r.len() != width_bytes) {
                None
            } else {
                let mut data = smallvec![];
                for &row in s {
                    for c in row.chars() {
                        if let Some(cell) = Cell::from_braille(c) {
                            data.push(ColoredCell { cell, color });
                        } else {
                            return None;
                        }
                    }
                }
                if width_bytes == 0 {
                    Some(Self::empty(0, 0, priority))
                } else {
                    let width_cells = width_bytes / BRAILLE_UTF8_BYTES;
                    let height_cells = data.len() / width_cells;
                    match (u16::try_from(width_cells), u16::try_from(height_cells)) {
                        (Ok(w), Ok(h)) => Some(Self::new(data, w, h, priority)),
                        _ => None,
                    }
                }
            }
        }
    }

    /// Computes the size of a sprite's bounding box after being offset a specified amount.
    /// Returns a `(width, height)` pair, measured in cells.
    ///
    /// Overflows / panics when the sprite's original bounding box has a dimension of size
    /// [`u16::MAX`] and the offset would increment its width.
    pub const fn offset_size(&self, offset: u8) -> (u16, u16) {
        let (x, y) = offset_px(offset);
        ((x != 0) as u16 + self.width, (y != 0) as u16 + self.height)
    }

    /// Returns the width of the sprite in cells, assuming it is at a zero pixel offset
    pub const fn default_width(&self) -> u16 {
        self.offset_size(0).0
    }
    /// Returns the height of the sprite in cells, assuming it is at a zero pixel offset
    pub const fn default_height(&self) -> u16 {
        self.offset_size(0).1
    }

    /// Creates a copy of the sprite but with all pixels set to the given color
    pub fn recolor<F: Fn(ColoredCell) -> Option<Color>>(&self, f: F) -> Self {
        let data: SpriteData = self.offsets[0]
            .iter()
            .copied()
            .map(|cell| ColoredCell::new(cell.cell, f(cell)))
            .collect();
        Self::new(
            data,
            self.default_width(),
            self.default_height(),
            self.priority,
        )
    }
}

#[cfg(all(test, feature = "images"))]
mod image_tests {
    use crate::screen::Screen;

    use super::*;

    #[test]
    fn sprite_image_from_path() {
        let sprite =
            Sprite::rgb_from_image_path("examples/heart.png", 1, true, 0).expect("png failure");
        assert_eq!(sprite.height, 4);
        assert_eq!(sprite.width, 8);
        let mut screen = Screen::new_pixels(16, 16);
        screen.draw_sprite(&sprite, 0, 0, crate::screen::Blit::Set);
        screen.rasterize();
    }
}
