use crate::cell::Cell;

/// A sprite color may be specified in three different ways: as [`Color::None`],
/// [`Color::Relaxed`], or [`Color::Forced`]. These have different behaviors when applied
/// to a sprite.
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ColoredCell {
    pub cell: Cell,
    pub color: Color,
}

impl ColoredCell {
    pub fn new(cell: Cell) -> Self {
        Self {
            cell,
            ..Default::default()
        }
    }

    pub fn with_color(self, color: Color) -> Self {
        Self { color, ..self }
    }

    pub fn with_cell(self, cell: Cell) -> Self {
        Self { cell, ..self }
    }

    pub fn merge_cell(self, cell: Cell, color: Color) -> Self {
        Self {
            cell: self.cell | cell,
            color,
        }
    }
}
