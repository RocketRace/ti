//! Methods for adding color when drawing types such as [`crate::sprite::Sprite`] and [`crate::cell::Cell`] to the screen.

use crate::cell::Cell;

/// Color metadata for a cell or a sprite.
///
/// A sprite color may be specified in three different ways:
/// - [`Color::None`],
/// - [`Color::Relaxed`], or
/// - [`Color::Forced`].
/// These have different behaviors when applied to a sprite.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Color {
    /// No color is applied to the sprite. This sprite will not assert any color in the
    /// current cell.
    #[default]
    None,
    /// Color is applied to the sprite, but in a relaxed fashion. If the sprite is empty,
    /// consisting of only unset pixels, then its color will not be asserted. Otherwise,
    /// the color will be drawn to the screen.
    Relaxed(()),
    /// Color is always applied to the sprite, even if the sprite is empty.
    Forced(()),
}

/// A [`Cell`] with associated [`Color`] data.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ColoredCell {
    pub cell: Cell,
    pub color: Color,
}

impl ColoredCell {
    /// Creates a new [`ColoredCell`] from parameters
    pub fn new(cell: Cell, color: Color) -> Self {
        Self { cell, color }
    }

    /// Combines this cell's pixel data with the argument [`Cell`] with a bitwise OR.
    pub fn merge_cell(&mut self, cell: Cell) {
        self.cell = self.cell | cell;
    }
}
