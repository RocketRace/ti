//! Module for manipulating [`Sprite`]s, i.e. rectangular collections of [`Cell`]s with associated color information.
use smallvec::{smallvec, SmallVec};
use std::array;

#[cfg(feature = "images")]
pub use image::ImageResult;
#[cfg(feature = "images")]
use image::{imageops::FilterType, DynamicImage, GenericImageView, Rgba};

use crate::{
    cell::{Cell, OffsetCell, BRAILLE_UTF8_BYTES, PIXEL_HEIGHT, PIXEL_OFFSETS, PIXEL_WIDTH},
    color::{Color, ColoredCell},
    units::{cell_length, from_index, index, offset_px, px_offset},
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
}

type SpriteData = SmallVec<[ColoredCell; SPRITE_STACK_SIZE]>;

impl Sprite {
    /// Create a new empty [`Sprite`] with the given dimensions.
    /// The width and height parameters are in terms of cells.
    pub fn empty(width_cells: u16, height_cells: u16) -> Self {
        Self {
            offsets: array::from_fn(
                |_| smallvec![ColoredCell::default(); cell_length(width_cells, height_cells)],
            ),
            width: width_cells,
            height: height_cells,
        }
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
    pub fn new(data: SpriteData, width_cells: u16, height_cells: u16) -> Self {
        let mut this = Self::empty(width_cells, height_cells);
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
                                buf[i_ul] = ColoredCell::new(cell, color);
                            }
                            OffsetCell::Horizontal { left, right } => {
                                buf[i_ul].merge_cell(left);
                                buf[i_ur] = ColoredCell::new(right, color);
                            }
                            OffsetCell::Vertical { up, down } => {
                                buf[i_ul].merge_cell(up);
                                buf[i_dl] = ColoredCell::new(down, color);
                            }
                            OffsetCell::Corner { ul, ur, dl, dr } => {
                                buf[i_ul].merge_cell(ul);
                                buf[i_ur].merge_cell(ur);
                                buf[i_dl].merge_cell(dl);
                                buf[i_dr] = ColoredCell::new(dr, color);
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
    pub fn from_braille_string(s: &[&str], color: Option<Color>) -> Option<Self> {
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
                            data.push(ColoredCell { cell, color });
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
                    match (u16::try_from(width_cells), u16::try_from(height_cells)) {
                        (Ok(w), Ok(h)) => Some(Self::new(data, w, h)),
                        _ => None,
                    }
                }
            }
        }
    }

    /// Reads and parses an image sprite from the specified file path using RGB colors.
    ///
    /// The file can be in any image format supported by [`image::open()`], decided by the file extension given.
    ///
    /// The resulting image will be rescaled to a width and height of `width_px` and `height_px` pixels, without
    /// preserving aspect ratio. This rescaling is done with nearest neighbor sampling.
    ///
    /// The pixels in the output image are all "on" (in terms of their [`Cell`] representation). The colors in the
    /// input image are reflected in the *cell colors* of the output sprite.
    ///
    #[cfg(feature = "images")]
    pub fn rgb_from_image_path<P: AsRef<std::path::Path>>(
        path: P,
        width_px: u16,
        height_px: u16,
        use_alpha_channel: bool,
    ) -> image::ImageResult<Self> {
        Ok(Self::from_image_data(
            image::open(path)?,
            width_px,
            height_px,
            FilterType::Nearest,
            FilterType::Nearest,
            ColorMode::Rgb,
            use_alpha_channel,
        ))
    }

    /// Reads and parses an image sprite from the specified file path using standard ANSI colors.
    ///
    /// This is a version of [`Sprite::rgb_from_image_path()`] that parses colors as standard colors only.
    #[cfg(feature = "images")]
    pub fn standard_from_image_path<P: AsRef<std::path::Path>>(
        path: P,
        width_px: u16,
        height_px: u16,
        use_alpha_channel: bool,
    ) -> image::ImageResult<Self> {
        Ok(Self::from_image_data(
            image::open(path)?,
            width_px,
            height_px,
            FilterType::Nearest,
            FilterType::Nearest,
            ColorMode::Standard,
            use_alpha_channel,
        ))
    }

    /// Reads and parses an image sprite from the specified file path using standard ANSI colors.
    ///
    /// This is a version of [`Sprite::rgb_from_image_path()`] that parses colors as standard colors only.
    #[cfg(feature = "images")]
    pub fn mono_from_image_path<P: AsRef<std::path::Path>>(
        path: P,
        width_px: u16,
        height_px: u16,
    ) -> image::ImageResult<Self> {
        Ok(Self::from_image_data(
            image::open(path)?,
            width_px,
            height_px,
            FilterType::Nearest,
            FilterType::Nearest,
            ColorMode::Monochrome,
            true,
        ))
    }

    /// Parses a sprite from dynamic image data.
    ///
    /// The `rescale_filter` declares the method used to resize to a specified resolution, and `downscale_filter` declares
    /// the method used to thumbnail each cell into a single color.
    /// `color_mode` specifies the color resolution used in the output, and `use_alpha_channel` dictates whether the image's alpha channel
    /// will be used to infer sprite shape.
    #[cfg(feature = "images")]
    fn from_image_data(
        img: DynamicImage,
        width_px: u16,
        height_px: u16,
        rescale_filter: FilterType,
        downscale_filter: FilterType,
        color_mode: ColorMode,
        use_alpha_channel: bool,
    ) -> Self {
        use crate::units::pos_components;

        let width_cells = width_px / PIXEL_WIDTH as u16;
        let height_cells = height_px / PIXEL_HEIGHT as u16;
        let resized = img.resize_exact(width_px as u32, height_px as u32, rescale_filter);

        let mut data: SpriteData =
            smallvec![ColoredCell::default(); cell_length(width_cells, height_cells)];

        // Initialize pixel contents first
        if use_alpha_channel {
            for (x, y, Rgba([_, _, _, a])) in resized.pixels() {
                let ((cell_x, px_x), (cell_y, px_y)) = pos_components(x as u16, y as u16);
                let idx = index(cell_x, cell_y, width_cells);
                let bit = Cell::from_bit_position(px_x, px_y).unwrap();
                if a < 128 {
                    data[idx].cell = data[idx].cell | bit;
                }
            }
        } else {
            data.fill(ColoredCell::new(Cell::full(), None))
        }

        let colors =
            resized.resize_exact(width_cells as u32, height_cells as u32, downscale_filter);

        // Then, pixel colors
        if matches!(color_mode, ColorMode::Rgb | ColorMode::Standard) {
            for (x, y, Rgba([r, g, b, _])) in colors.pixels() {
                let index = index(x as u16, y as u16, width_cells);
                let color = if matches!(color_mode, ColorMode::Standard) {
                    Color::standard_color_approximate(r, g, b)
                } else {
                    Color::from_rgb_approximate(r, g, b)
                };
                data[index] = ColoredCell::new(Cell::new(0xff), Some(color));
            }
        }

        Sprite::new(data, width_cells, height_cells)
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
}

#[allow(unused)] // false positive from clippy
enum ColorMode {
    Monochrome,
    Standard,
    Rgb,
}

#[cfg(all(test, feature = "images"))]
mod image_tests {
    use crate::screen::Screen;

    use super::*;

    #[test]
    fn sprite_image_from_path() {
        let sprite =
            Sprite::rgb_from_image_path("examples/sprite.png", 24, 24, true).expect("png failure");
        assert_eq!(sprite.height, 6);
        assert_eq!(sprite.width, 12);
        let mut screen = Screen::new_cells(12, 6);
        screen.draw_sprite(&sprite, 0, 0, crate::screen::Blit::Set);
        screen.rasterize();
    }
}
