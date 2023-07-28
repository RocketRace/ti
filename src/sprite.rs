//! Module for manipulating [`Cell`]s, i.e. collections of cells with associated color information.
use std::array;

use smallvec::{smallvec, SmallVec};

use crate::{
    cell::{Cell, OffsetCell, BRAILLE_UTF8_BYTES, PIXEL_HEIGHT, PIXEL_WIDTH},
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
    pub width_cells: [usize; PIXEL_HEIGHT * PIXEL_WIDTH],
    pub height_cells: [usize; PIXEL_HEIGHT * PIXEL_WIDTH],
}

type SpriteData = SmallVec<[ColoredCell; SPRITE_STACK_SIZE]>;

fn offset_width(width: usize, offset: usize) -> usize {
    if offset % PIXEL_WIDTH != 0 {
        width + 1
    } else {
        width
    }
}

fn offset_height(height: usize, offset: usize) -> usize {
    if offset / PIXEL_WIDTH != 0 {
        height + 1
    } else {
        height
    }
}

impl Sprite {
    /// Create a new empty [`Sprite`] with the given dimensions.
    /// The width and height parameters are in terms of cells.
    pub fn empty(width_cells: usize, height_cells: usize) -> Self {
        Self {
            offsets: array::from_fn(
                |_| smallvec![ColoredCell::default(); width_cells * height_cells],
            ),
            width_cells: array::from_fn(|i| offset_width(width_cells, i)),
            height_cells: array::from_fn(|i| offset_height(height_cells, i)),
        }
    }
    /// Creates a sprite from raw data.
    pub fn new(data: SpriteData, width_cells: usize, height_cells: usize) -> Self {
        let mut offsets = vec![];
        for y_offset in 0..PIXEL_HEIGHT {
            for x_offset in 0..PIXEL_WIDTH {
                let expanded_width = offset_width(width_cells, y_offset * PIXEL_WIDTH + x_offset);
                let expanded_height =
                    offset_height(height_cells, y_offset * PIXEL_WIDTH + x_offset);

                let expanded_size = expanded_width * expanded_height;
                let mut offset_buf: SpriteData = smallvec![ColoredCell::default(); expanded_size];
                for y_cell in 0..height_cells {
                    for x_cell in 0..width_cells {
                        // note: original has width `width_cells`, final has width `expanded_width`
                        let ColoredCell { cell, color } = data[y_cell * width_cells + x_cell];
                        let i = y_cell * expanded_width + x_cell;
                        match cell.with_offset(x_offset, y_offset) {
                            OffsetCell::Aligned { cell } => {
                                offset_buf[i] = ColoredCell::new(cell, color);
                            }
                            OffsetCell::Horizontal { left, right } => {
                                offset_buf[i].merge_cell(left);
                                offset_buf[i + 1] = ColoredCell::new(right, color);
                            }
                            OffsetCell::Vertical { up, down } => {
                                offset_buf[i].merge_cell(up);
                                offset_buf[i + expanded_width] = ColoredCell::new(down, color);
                            }
                            OffsetCell::Corner { ul, ur, dl, dr } => {
                                offset_buf[i].merge_cell(ul);
                                offset_buf[i + 1].merge_cell(ur);
                                offset_buf[i + expanded_width].merge_cell(dl);
                                offset_buf[i + expanded_width + 1] = ColoredCell::new(dr, color);
                            }
                        }
                    }
                }
                offsets.push(offset_buf);
            }
        }
        Self {
            offsets: offsets
                .try_into()
                .expect("Precomputed offsets should contain all 8 values"),
            width_cells: array::from_fn(|i| offset_width(width_cells, i)),
            height_cells: array::from_fn(|i| offset_height(height_cells, i)),
        }
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
}
