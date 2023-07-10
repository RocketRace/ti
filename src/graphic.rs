//! Module for manipulating [`Graphic`]s, i.e. collections of cells with associated color information.
use smallvec::{smallvec, SmallVec};

use crate::{cell::Cell, color::Color};

/// Stack allocation size for graphics cell data
const GRAPHIC_STACK_SIZE: usize = 64;

/// A visual graphic.
pub struct Graphic {
    data: SmallVec<[(Cell, Color); GRAPHIC_STACK_SIZE]>,
    cell_width: usize,
    cell_height: usize,
}

impl Graphic {
    /// Create a new empty [`Graphic`] with the given dimensions.
    /// The width and height parameters are in terms of cells.
    pub fn empty(cell_width: usize, cell_height: usize) -> Self {
        Self {
            data: smallvec![(Cell::default(), Color::None); cell_width * cell_height],
            cell_width,
            cell_height,
        }
    }

    /// Creates a [`Graphic`] from the given sequence of braille strings.
    /// Each element of the parameter slice is interpreted as a row of the graphic.
    ///
    /// Returns None if any characters in the string are non-braille, or if the rows
    /// are different lengths.
    pub fn from_braille_string(s: &[&str]) -> Option<Self> {
        if s.is_empty() {
            Some(Graphic::empty(0, 0))
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
                    Some(Graphic::empty(0, 0))
                } else {
                    let cell_height = data.len() / cell_width;

                    Some(Graphic {
                        data,
                        cell_width,
                        cell_height,
                    })
                }
            }
        }
    }
}
