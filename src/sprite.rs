//! Module for manipulating [`Sprite`]s, i.e. rectangular collections of [`Cell`]s with associated color information.
use std::array;

use smallvec::{smallvec, SmallVec};

use crate::{
    cell::{Cell, Offset, BRAILLE_UTF8_BYTES, PIXEL_HEIGHT, PIXEL_WIDTH},
    color::{Color, ColoredCell},
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
    ///
    /// Note that the color data doesn't get
    pub offsets: [SpriteData; PIXEL_HEIGHT * PIXEL_WIDTH],
    width: usize,
    height: usize,
}

type SpriteData = SmallVec<[ColoredCell; SPRITE_STACK_SIZE]>;

impl Sprite {
    /// Create a new empty [`Sprite`] with the given dimensions.
    /// The width and height parameters are in terms of cells.
    pub fn empty(width_cells: usize, height_cells: usize) -> Self {
        Self {
            offsets: array::from_fn(
                |_| smallvec![ColoredCell::default(); width_cells * height_cells],
            ),
            width: width_cells,
            height: height_cells,
        }
    }
    /// Creates a sprite from raw data.
    pub fn new(data: SpriteData, width_cells: usize, height_cells: usize) -> Self {
        let mut this = Self::empty(width_cells, height_cells);
        for dy in 0..PIXEL_HEIGHT {
            for dx in 0..PIXEL_WIDTH {
                let offset = dy * PIXEL_WIDTH + dx;
                let (new_width, new_height) = this.offset_size(offset);
                let new_size = new_width * new_height;
                this.offsets[offset].resize(new_size, ColoredCell::default());

                let buf = &mut this.offsets[offset];
                for y in 0..height_cells {
                    for x in 0..width_cells {
                        // note: original has width `width_cells`, final has width `expanded_width`
                        let ColoredCell { cell, color } = data[y * width_cells + x];
                        let i = y * new_width + x;
                        match cell.with_offset(dx, dy) {
                            Offset::Aligned { cell } => {
                                buf[i] = ColoredCell::new(cell, color);
                            }
                            Offset::Horizontal { left, right } => {
                                buf[i].merge_cell(left);
                                buf[i + 1] = ColoredCell::new(right, color);
                            }
                            Offset::Vertical { up, down } => {
                                buf[i].merge_cell(up);
                                buf[i + new_width] = ColoredCell::new(down, color);
                            }
                            Offset::Corner { ul, ur, dl, dr } => {
                                buf[i].merge_cell(ul);
                                buf[i + 1].merge_cell(ur);
                                buf[i + new_width].merge_cell(dl);
                                buf[i + new_width + 1] = ColoredCell::new(dr, color);
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
    pub fn from_braille_string(s: &[&str]) -> Option<Self> {
        if s.is_empty() {
            Some(Sprite::empty(0, 0))
        } else {
            let width_bytes = s[0].len();
            if s.iter().any(|&r| r.len() != width_bytes) {
                None
            } else {
                let mut data = smallvec![];
                for &row in s {
                    for c in row.chars() {
                        if let Some(cell) = Cell::from_braille(c) {
                            data.push(ColoredCell {
                                cell,
                                color: Color::default(),
                            });
                        } else {
                            return None;
                        }
                    }
                }
                if width_bytes == 0 {
                    Some(Self::empty(0, 0))
                } else {
                    let width_cells = width_bytes / BRAILLE_UTF8_BYTES;
                    let height_cells = data.len() / width_cells;

                    Some(Self::new(data, width_cells, height_cells))
                }
            }
        }
    }

    /// Computes the size of a sprite's bounding box after being offset a specified amount.
    /// Returns a `(width, height)` pair, measured in cells.
    pub fn offset_size(&self, offset: usize) -> (usize, usize) {
        let x = offset % PIXEL_WIDTH != 0;
        let y = offset / PIXEL_WIDTH != 0;
        (usize::from(x) + self.width, usize::from(y) + self.height)
    }
}
