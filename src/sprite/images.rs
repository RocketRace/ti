//! Module for reading sprites out of images.

use super::*;

use std::collections::BTreeMap;
use std::path::Path;

pub use image::ImageResult;

use image::imageops::FilterType::Nearest;
use image::{DynamicImage, GenericImage, GenericImageView, Rgba};

use crate::units::pos_components;

/// The different ways that raw pixel data can be interpreted as a sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColorMode {
    Monochrome,
    Standard,
    Rgb,
}

/// A sprite atlas opened from a file.
pub struct Atlas {
    image: DynamicImage,
    /// A setting to determine how sprites are read from this atlas
    pub color_mode: ColorMode,
    /// A setting to determine how sprites are read from this atlas
    pub use_alpha_channel: bool,
}

impl Atlas {
    /// Opens a sprite atlas from a file path.
    pub fn open<P: AsRef<Path>>(
        path: P,
        color_mode: ColorMode,
        use_alpha_channel: bool,
    ) -> ImageResult<Self> {
        image::open(path).map(|image| Atlas {
            image,
            color_mode,
            use_alpha_channel,
        })
    }
    /// Fetches the sprite at the given coordinates in this atlas.
    pub fn sprite(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        scale: u16,
        priority: u16,
    ) -> Sprite {
        Sprite::from_image_data(
            DynamicImage::ImageRgba8(self.image.view(x, y, width, height).to_image()),
            self.color_mode,
            scale,
            self.use_alpha_channel,
            priority,
        )
    }
}

impl Sprite {
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
    pub fn rgb_from_image_path<P: AsRef<std::path::Path>>(
        path: P,
        scale: u16,
        use_alpha_channel: bool,
        priority: u16,
    ) -> image::ImageResult<Self> {
        Ok(Self::from_image_data(
            image::open(path)?,
            ColorMode::Rgb,
            scale,
            use_alpha_channel,
            priority,
        ))
    }

    /// Reads and parses an image sprite from the specified file path using standard ANSI colors.
    ///
    /// This is a version of [`Sprite::rgb_from_image_path()`] that parses colors as standard colors only.
    pub fn standard_from_image_path<P: AsRef<std::path::Path>>(
        path: P,
        scale: u16,
        use_alpha_channel: bool,
        priority: u16,
    ) -> image::ImageResult<Self> {
        Ok(Self::from_image_data(
            image::open(path)?,
            ColorMode::Standard,
            scale,
            use_alpha_channel,
            priority,
        ))
    }

    /// Reads and parses an image sprite from the specified file path using standard ANSI colors.
    ///
    /// This is a version of [`Sprite::rgb_from_image_path()`] that parses colors as standard colors only.
    pub fn mono_from_image_path<P: AsRef<std::path::Path>>(
        path: P,
        scale: u16,
        priority: u16,
    ) -> image::ImageResult<Self> {
        Ok(Self::from_image_data(
            image::open(path)?,
            ColorMode::Monochrome,
            scale,
            true,
            priority,
        ))
    }

    /// Parses a sprite from dynamic image data.
    ///
    /// The `rescale_filter` declares the method used to resize to a specified resolution, and `downscale_filter` declares
    /// the method used to thumbnail each cell into a single color.
    /// `color_mode` specifies the color resolution used in the output, and `use_alpha_channel` dictates whether the image's alpha channel
    /// will be used to infer sprite shape.
    fn from_image_data(
        mut img: DynamicImage,
        color_mode: ColorMode,
        scale: u16,
        use_alpha_channel: bool,
        priority: u16,
    ) -> Self {
        img = img.resize_exact(
            img.width() * scale as u32,
            img.height() * scale as u32,
            Nearest,
        );
        let width_px = img.width() as u16;
        let height_px = img.height() as u16;

        let width_cells = width_px / PIXEL_WIDTH as u16;
        let height_cells = height_px / PIXEL_HEIGHT as u16;

        let mut data: SpriteData =
            smallvec![ColoredCell::default(); cell_length(width_cells, height_cells)];

        // Initialize pixel contents first
        if use_alpha_channel {
            for (x, y, Rgba([_, _, _, a])) in img.pixels() {
                let ((cell_x, px_x), (cell_y, px_y)) = pos_components(x as u16, y as u16);
                let idx = index(cell_x, cell_y, width_cells);
                let bit = Cell::from_bit_position(px_x, px_y).unwrap();
                if a > 128 {
                    data[idx].cell = data[idx].cell | bit;
                }
            }
        } else {
            data.fill(ColoredCell::new(Cell::full(), None))
        }

        // Then, pixel colors
        if matches!(color_mode, ColorMode::Rgb | ColorMode::Standard) {
            for y_cell in 0..height_cells {
                for x_cell in 0..width_cells {
                    let x_px = x_cell * PIXEL_WIDTH as u16;
                    let y_px = y_cell * PIXEL_HEIGHT as u16;

                    let index = index(x_cell, y_cell, width_cells);

                    let view = img.sub_image(
                        x_px as u32,
                        y_px as u32,
                        PIXEL_WIDTH as u32,
                        PIXEL_HEIGHT as u32,
                    );

                    // hmm
                    let mut pxs = BTreeMap::new();
                    for (_, _, Rgba([r, g, b, a])) in view.pixels() {
                        if a > 128 || !use_alpha_channel {
                            let color = if matches!(color_mode, ColorMode::Rgb) {
                                Color::from_rgb_approximate(r, g, b)
                            } else {
                                Color::standard_color_approximate(r, g, b)
                            };
                            pxs.entry(color).and_modify(|n| *n += 1).or_insert(1);
                        }
                    }
                    let max = pxs.into_iter().max_by_key(|p| p.1).map(|p| p.0);

                    data[index].color = max;
                }
            }
        }

        Sprite::new(data, width_cells, height_cells, priority)
    }
}
