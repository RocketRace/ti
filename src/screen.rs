//! Module for writing output to the screen.
//! Contains the [`Screen`] type and its public interface.

use crate::{
    cell::{Cell, BRAILLE_UTF8_BYTES, PIXEL_HEIGHT, PIXEL_WIDTH},
    sprite::Sprite,
};

/// Type used to write to the screen. Contains public methods
/// to write pixels and sprites to the screen, as well as colors.
///
/// The point (0, 0) represents the top left pixel of the screen.
///
/// The [`Screen::rasterize`] method can be used to generate
/// bytes that can be written to a terminal.
pub struct Screen {
    cells: Vec<Cell>,
    #[allow(unused)]
    updates: Vec<Cell>,
    width: usize,
    height: usize,
}

/// A blit type used to select the type of operation
/// when writing to the screen. In the case of single pixels,
/// this is used to determine whether the output pixel is
/// set to 1, set to 0 or flipped.
#[derive(Clone, Copy)]
pub enum Blit {
    /// Sets the output to 0 where the input is set, and 1 elsewhere.
    Unset,
    /// Sets the output to 1 where the input is set, and 0 elsewhere.
    Set,
    /// Sets the output to 1 where the input is set, and ignore elsewhere.
    Add,
    /// Sets the output to 0 where the input is set, and ignore elsewhere.
    Subtract,
    /// Flip the output bits where the input is set.
    Toggle,
}

// https://github.com/rust-lang/rust/issues/88581
fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

impl Screen {
    /// Create a new empty screen with the given dimensions in pixels.
    /// The resulting width and height are rounded up to the nearest multiple of
    /// [`PIXEL_WIDTH`] and [`PIXEL_HEIGHT`].
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![
                Cell::default();
                div_ceil(width, PIXEL_WIDTH) * div_ceil(height, PIXEL_HEIGHT)
            ],
            updates: vec![
                Cell::default();
                div_ceil(width, PIXEL_WIDTH) * div_ceil(height, PIXEL_HEIGHT)
            ],
            width,
            height,
        }
    }

    /// Compute the height of the screen, in number of cells. This is a divisor of its pixel height.
    pub fn height_cells(&self) -> usize {
        div_ceil(self.height, PIXEL_HEIGHT)
    }
    /// Compute the width of the screen, in number of cells. This is a divisor of its pixel width.
    pub fn width_cells(&self) -> usize {
        div_ceil(self.width, PIXEL_WIDTH)
    }
    /// Compute the height of the screen, in number of pixels. This is a multiple of its cell height.
    pub fn height_pixels(&self) -> usize {
        self.height
    }
    /// Compute the width of the screen, in number of pixels. This is a multiple of its cell width.
    pub fn width_pixels(&self) -> usize {
        self.width
    }

    fn cell_index(&self, cell_x: usize, cell_y: usize) -> usize {
        cell_y * self.width_cells() + cell_x
    }

    fn pixel_index(&self, x: usize, y: usize) -> (usize, u8) {
        let index = self.cell_index(x / PIXEL_WIDTH, y / PIXEL_HEIGHT);
        let x_pixel = x % PIXEL_WIDTH;
        let y_pixel = y % PIXEL_HEIGHT;
        let pixel = 1 << ((y_pixel * PIXEL_WIDTH) + x_pixel);
        (index, pixel)
    }

    #[allow(unused)]
    fn pixel_at(&self, x: usize, y: usize) -> bool {
        let (index, pixel) = self.pixel_index(x, y);
        self.cells[index].bits & pixel != 0
    }

    /// Transforms the pixel value at the given coordinates with a generic given blitting strategy.
    ///
    /// This accepts a `blit` parameter that determines how the pixel will be drawn:
    /// * [`Blit::Set`] and [`Blit::Add`] are synonymous and cause the pixel to be set.
    /// * [`Blit::Unset`] and [`Blit::Subtract`] are synonymous and cause the pixel to be unset.
    /// * [`Blit::Toggle`] causes the pixel to be flipped, i.e. turned from a 1 to a 0 and vice versa.
    ///
    /// Returns `true` if the coordinates were valid, and `false` if the given coordinate was out of bounds.
    pub fn draw_pixel(&mut self, x: usize, y: usize, blit: Blit) -> bool {
        if x < self.width && y < self.height {
            let (index, pixel) = self.pixel_index(x, y);
            let orig = self.cells[index].bits;
            self.cells[index].bits = match blit {
                Blit::Set | Blit::Add => orig | pixel,
                Blit::Unset | Blit::Subtract => orig & !pixel,
                Blit::Toggle => orig ^ pixel,
            };
            true
        } else {
            false
        }
    }

    /// Draws a [`Cell`] to the screen at a given cell position. The given x and y positions
    /// are in terms of cells.
    ///
    /// This accepts an additional `blit` parameter specifying how
    /// the sprite should be drawn:
    /// * [`Blit::Set`] => Draw the entire sprite normally to the screen, including
    ///   unset pixels.
    /// * [`Blit::Unset`] => Draw the entire sprite inverted to the screen, including
    ///   pixels that were originally unset in the sprite.
    /// * [`Blit::Add`] => Draw the sprite additively to the screen. Pixels that
    ///   are set in the sprite will be set, rest are unchanged.
    /// * [`Blit::Subtract`] => Draw the sprite subtractively to the screen. Pixels that
    ///   are set in the sprite will be unset, rest are unchanged.
    /// * [`Blit::Toggle`] => Flip the pixels on the screen where the sprite is set.
    ///
    /// Returns `true` if the coordinates were valid, and `false` if the given coordinate was out of bounds.
    pub fn draw_cell(&mut self, cell: Cell, x: usize, y: usize, blit: Blit) -> bool {
        if x < self.width_cells() && y < self.height_cells() {
            let index = self.cell_index(x, y);
            let orig = self.cells[index].bits;
            self.cells[index].bits = match blit {
                Blit::Unset => !cell.bits,
                Blit::Subtract => orig & !cell.bits,
                Blit::Set => cell.bits,
                Blit::Add => orig | cell.bits,
                Blit::Toggle => orig ^ cell.bits,
            };
            true
        } else {
            false
        }
    }

    /// Draws a single sprite to the screen. The x and y coordinates are specified in pixels,
    /// and refer to the top left corner of the sprite.
    ///
    /// Returns `false` if any part of the sprite was clipped by the screen boundaries, `true` otherwise.
    pub fn draw_sprite(
        &mut self,
        sprite: Sprite,
        x_pixel: usize,
        y_pixel: usize,
        blit: Blit,
    ) -> bool {
        let offset = (y_pixel % PIXEL_HEIGHT) * PIXEL_WIDTH + (x_pixel % PIXEL_WIDTH);
        let choice = &sprite.offsets[offset];
        let sprite_width = sprite.width_cells[offset];
        choice.iter().enumerate().fold(true, |acc, (i, cell)| {
            let y_cell = y_pixel / PIXEL_HEIGHT + i / sprite_width;
            let x_cell = x_pixel / PIXEL_WIDTH + i % sprite_width;
            acc & self.draw_cell(cell.cell, x_cell, y_cell, blit)
        })
    }

    /// Sets the pixel value at the given coordinates to be the given value. If `value` is
    /// `true`, sets the pixel value to be 1. Otherwise, sets it to 0.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing sprites that can partially clip off screen.
    pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
        self.draw_pixel(x, y, if value { Blit::Set } else { Blit::Unset });
    }

    /// Flips the pixel value at the given coordinates to be 1.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing sprites that can partially clip off screen.
    pub fn toggle_pixel(&mut self, x: usize, y: usize) {
        self.draw_pixel(x, y, Blit::Toggle);
    }

    /// Converts the screen to a utf-8 sequence of bytes that can be rendered in a terminal.
    /// Includes newlines in its output.
    pub fn rasterize(&self) -> Vec<u8> {
        // additional + height given for newline chars
        let mut buf = vec![0; self.cells.len() * BRAILLE_UTF8_BYTES + self.height_cells()];
        for y in 0..self.height_cells() {
            for x in 0..self.width_cells() {
                let i = self.cell_index(x, y);
                // extra newlines also counted here
                buf[i * 3 + y..(i + 1) * 3 + y].copy_from_slice(&self.cells[i].to_braille_utf8())
            }
            buf[(y + 1) * (self.width_cells() * 3 + 1) - 1] = b'\n';
        }
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_screen_size() {
        let screen = Screen::new(16, 24);
        assert_eq!(screen.width_cells(), 8);
        assert_eq!(screen.height_cells(), 6);
    }

    #[test]
    fn odd_screen_size() {
        let screen = Screen::new(3, 3);
        assert_eq!(screen.width_cells(), 2);
        assert_eq!(screen.height_cells(), 1);
    }

    #[test]
    fn make_square() {
        let mut screen = Screen::new(8, 8);
        for i in 0..8 {
            screen.set_pixel(i, 0, true);
            screen.set_pixel(i, 7, true);
            screen.set_pixel(0, i, true);
            screen.set_pixel(7, i, true);
        }
        assert_eq!(
            std::str::from_utf8(&screen.rasterize()).unwrap(),
            "⡏⠉⠉⢹\n⣇⣀⣀⣸\n"
        )
    }

    #[test]
    fn blit_types() {
        let mut screen = Screen::new(1, 1);
        assert!(!screen.pixel_at(0, 0));
        screen.set_pixel(0, 0, true);
        assert!(screen.pixel_at(0, 0));
        screen.set_pixel(0, 0, true);
        assert!(screen.pixel_at(0, 0));
        screen.set_pixel(0, 0, false);
        assert!(!screen.pixel_at(0, 0));
        screen.set_pixel(0, 0, false);
        assert!(!screen.pixel_at(0, 0));
        screen.toggle_pixel(0, 0);
        assert!(screen.pixel_at(0, 0));
        screen.toggle_pixel(0, 0);
        assert!(!screen.pixel_at(0, 0));
    }

    #[test]
    fn simple_aligned_sprites() {
        let mut screen = Screen::new(4, 4);
        let cell = Cell::new(0b00111100);
        screen.draw_cell(cell, 0, 0, Blit::Set);
        assert_eq!(screen.cells[0], cell);
        assert_eq!(screen.cells[1], Cell::new(0));
        screen.draw_cell(cell, 1, 0, Blit::Unset);
        assert_eq!(screen.cells[0], cell);
        assert_eq!(screen.cells[1], Cell::new(!cell.bits));
        screen.draw_cell(cell, 0, 0, Blit::Toggle);
        assert_eq!(screen.cells[0], Cell::new(0));
        assert_eq!(screen.cells[1], Cell::new(!cell.bits));
    }

    #[test]
    fn draw_cell() {}

    #[test]
    fn draw_sprite() {}
}
