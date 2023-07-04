//! Module implementing atomic sprites.

use crate::{cell::Cell, color::Color};

/// A single cell's worth of pixel data. Outside from raw pixels to the screen,
/// a sprite is the atomic unit of graphical information.
///
/// A sprite contains information about its pixels, their color,
///
/// Multiple sprites can be composed together into a [`crate::graphic::Graphic`],
/// which is a convenient  wrapper around pixel information comprised of more than one sprite.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Sprite {
    /// The raw pixel data of this sprite
    pub cell: Cell,
    /// The color information of this sprite, if any. If this is None, the sprite will not be
    /// drawn with any color.
    pub color: Option<Color>,
    /// The drawing priority of the sprite. This dictates the order in which the sprite is drawn.
    /// Higher priority sprites are drawn earlier than lower priority sprites.
    ///
    /// If this sprite also defines a color, then this priority will be used to determine the color
    /// drawn to the screen in case multiple sprites with color are drawn in the same position.
    pub priority: usize,
}

impl Sprite {
    /// Create a new sprite with the parameters
    pub fn new(cell: Cell, color: Option<Color>, priority: usize) -> Self {
        Self {
            cell,
            color,
            priority,
        }
    }
}
