//! Module for writing output to the screen.
//! Contains the [`Screen`] type and its public interface.

use crate::{
    cell::{Cell, BRAILLE_UTF8_BYTES},
    graphic::Graphic,
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
    /// [`Cell::PIXEL_WIDTH`] and [`Cell::PIXEL_HEIGHT`].
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            cells: vec![
                Cell::default();
                div_ceil(width, Cell::PIXEL_WIDTH) * div_ceil(height, Cell::PIXEL_HEIGHT)
            ],
            updates: vec![
                Cell::default();
                div_ceil(width, Cell::PIXEL_WIDTH) * div_ceil(height, Cell::PIXEL_HEIGHT)
            ],
            width,
            height,
        }
    }

    /// Compute the height of the screen, in number of cells. This is a divisor of its pixel height.
    pub fn cell_height(&self) -> usize {
        div_ceil(self.height, Cell::PIXEL_HEIGHT)
    }
    /// Compute the width of the screen, in number of cells. This is a divisor of its pixel width.
    pub fn cell_width(&self) -> usize {
        div_ceil(self.width, Cell::PIXEL_WIDTH)
    }
    /// Compute the height of the screen, in number of pixels. This is a multiple of its cell height.
    pub fn pixel_height(&self) -> usize {
        self.height
    }
    /// Compute the width of the screen, in number of pixels. This is a multiple of its cell width.
    pub fn pixel_width(&self) -> usize {
        self.width
    }

    fn cell_index(&self, cell_x: usize, cell_y: usize) -> usize {
        cell_y * self.cell_width() + cell_x
    }

    fn pixel_index(&self, x: usize, y: usize) -> (usize, u8) {
        let index = self.cell_index(x / Cell::PIXEL_WIDTH, y / Cell::PIXEL_HEIGHT);
        let x_pixel = x % Cell::PIXEL_WIDTH;
        let y_pixel = y % Cell::PIXEL_HEIGHT;
        let pixel = 1 << ((y_pixel * Cell::PIXEL_WIDTH) + x_pixel);
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
    pub fn draw_cell(&mut self, cell: Cell, cell_x: usize, cell_y: usize, blit: Blit) -> bool {
        if cell_x < self.cell_width() && cell_y < self.cell_height() {
            let index = self.cell_index(cell_x, cell_y);
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

    pub fn draw_graphic(&mut self, graphic: Graphic, pixel_x: usize, pixel_y: usize) {}

    /// Sets the pixel value at the given coordinates to be the given value. If `value` is
    /// `true`, sets the pixel value to be 1. Otherwise, sets it to 0.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing graphics that can partially clip off screen.
    pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
        self.draw_pixel(x, y, if value { Blit::Set } else { Blit::Unset });
    }

    /// Flips the pixel value at the given coordinates to be 1.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing graphics that can partially clip off screen.
    pub fn toggle_pixel(&mut self, x: usize, y: usize) {
        self.draw_pixel(x, y, Blit::Toggle);
    }

    /// Converts the screen to a utf-8 sequence of bytes that can be rendered in a terminal.
    /// Includes newlines in its output.
    pub fn rasterize(&self) -> Vec<u8> {
        // additional + height given for newline chars
        let mut buf = vec![0; self.cells.len() * BRAILLE_UTF8_BYTES + self.cell_height()];
        for y in 0..self.cell_height() {
            for x in 0..self.cell_width() {
                let i = self.cell_index(x, y);
                // extra newlines also counted here
                buf[i * 3 + y..(i + 1) * 3 + y].copy_from_slice(&self.cells[i].to_braille_utf8())
            }
            buf[(y + 1) * (self.cell_width() * 3 + 1) - 1] = b'\n';
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
        assert_eq!(screen.cell_width(), 8);
        assert_eq!(screen.cell_height(), 6);
    }

    #[test]
    fn odd_screen_size() {
        let screen = Screen::new(3, 3);
        assert_eq!(screen.cell_width(), 2);
        assert_eq!(screen.cell_height(), 1);
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
}
