//! Internal crate for keeping track of units.
//! Using pixels when cells are expected is a no no.
//!
//! Units used in this crate:
//!
//! Pixel x/y position/length: u16
//! Cell x/y position/length: u16
//! Sprite/screen cell index/length: usize
//! Subcell pixel x/y position/length: u8
//! Subcell pixel index/offset: u8

use crate::cell::{PIXEL_HEIGHT, PIXEL_WIDTH};

/// Computes an array length from its (x, y) dimensions
pub(crate) const fn cell_length(width: u16, height: u16) -> usize {
    (width as u32 * height as u32) as usize
}

/// Converts from a (x, y) subcell (pixel) position to a subcell (pixel) offset.
pub(crate) const fn px_offset(x: u8, y: u8) -> u8 {
    PIXEL_WIDTH * y + x
}
/// Converts from a subcell (pixel) index to a (x, y) subcell (pixel) position.
pub(crate) const fn offset_px(offset: u8) -> (u8, u8) {
    (offset % PIXEL_WIDTH, offset / PIXEL_WIDTH)
}

/// Converts from a (x, y) pixel position within a sprite / screen to its constituent
/// position components.
///
/// Returns a pair of pairs:
/// `((x cell coordinate, x subcell coordinate), (y cell coordinate, y subcell coordinate))`
pub(crate) const fn pos_components(x: u16, y: u16) -> ((u16, u8), (u16, u8)) {
    (
        (x / PIXEL_WIDTH as u16, (x % PIXEL_WIDTH as u16) as u8),
        (y / PIXEL_HEIGHT as u16, (y % PIXEL_HEIGHT as u16) as u8),
    )
}

/// Converts from a (x, y) position to an array index.
pub(crate) const fn index(x: u16, y: u16, width: u16) -> usize {
    (y as u32 * width as u32 + x as u32) as usize
}

/// Converts from an array index to a (x, y) position.
pub const fn from_index(i: usize, width: u16) -> (u16, u16) {
    (
        (i as u32 % width as u32) as u16,
        (i as u32 / width as u32) as u16,
    )
}
