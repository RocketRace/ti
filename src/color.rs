//! Methods for adding color when drawing types such as [`crate::sprite::Sprite`] and [`crate::cell::Cell`] to the screen.
//!
//! This uses [`crossterm::style::Color`] to represent ANSI terminal colors.

use crate::cell::Cell;

pub use crossterm::style::Color;

pub struct ColorFlags {
    /// When `true`, color is applied when the cell is drawn, even if the cell is empty.
    ///
    /// Otherwise, color is only applied to nonempty cells.
    pub apply_on_empty: bool,
}

/// A [`Cell`] with associated [`Color`] data.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ColoredCell {
    pub cell: Cell,
    pub color: Option<Color>,
}

impl ColoredCell {
    /// Creates a new [`ColoredCell`] from parameters
    pub fn new(cell: Cell, color: Option<Color>) -> Self {
        Self { cell, color }
    }

    /// Combines this cell's pixel data with the argument [`Cell`] with a bitwise OR.
    pub fn merge_cell(&mut self, cell: Cell) {
        self.cell = self.cell | cell;
    }
}
