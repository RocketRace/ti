//! Module for manipulating [`Cell`]s, i.e. collections of cells with associated color information.
use smallvec::{smallvec, SmallVec};

use crate::{cell::Cell, color::Color};

/// Stack allocation size for each sprite's cell data
const SPRITE_STACK_SIZE: usize = 64;

/// A sprite made up of a contiguous rectangular region of cells.
/// The cells may be colored.
pub struct Sprite {
    data: SmallVec<[(Cell, Color); SPRITE_STACK_SIZE]>,
    width_cells: usize,
    height_cells: usize,
}

impl Sprite {
    /// Create a new empty [`Sprite`] with the given dimensions.
    /// The width and height parameters are in terms of cells.
    pub fn empty(width_cells: usize, height_cells: usize) -> Self {
        Self {
            data: smallvec![(Cell::default(), Color::None); width_cells * height_cells],
            width_cells,
            height_cells,
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
            let cell_width = s[0].len();
            if s.iter().any(|&r| r.len() != cell_width) {
                None
            } else {
                let mut data = smallvec![];
                for &row in s {
                    for c in row.chars() {
                        if let Some(cell) = Cell::from_braille(c) {
                            data.push((cell, Color::None));
                        } else {
                            return None;
                        }
                    }
                }
                if cell_width == 0 {
                    Some(Self::empty(0, 0))
                } else {
                    let cell_height = data.len() / cell_width;

                    Some(Self {
                        data,
                        width_cells: cell_width,
                        height_cells: cell_height,
                    })
                }
            }
        }
    }
}
