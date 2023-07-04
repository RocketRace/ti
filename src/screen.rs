//! Module for writing output to the screen.
//! Contains the [Screen] type and its public interface.

use crate::cell::{Cell, BRAILLE_UTF8_BYTES};

/// Type used to write to the screen. Contains public methods
/// to write pixels and sprites to the screen, as well as colors.
///
/// The point (0, 0) represents the top left pixel of the screen.
///
/// The [`Screen::rasterize`] method can be used to generate
/// bytes that can be written to a terminal.
pub struct Screen {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

/// A blit type used to select the type of operation
/// when writing to the screen. In the case of single pixels,
/// this is used to determine whether the output pixel is
/// set to 1, set to 0 or flipped.
#[derive(Clone, Copy)]
pub enum Blit {
    /// Sets the output bits to 0
    Unset,
    /// Set the output bits to 1
    Set,
    /// Flips the output bits, i.e. sets 1 to 0 and 0 to 1
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

    fn pixel_index(&self, x: usize, y: usize) -> (usize, u8) {
        let index = y / Cell::PIXEL_HEIGHT * self.cell_width() + x / Cell::PIXEL_WIDTH;
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
    /// Returns `true` if a value was changed, and `false` if the given coordinate was out of bounds.
    pub fn transform_pixel(&mut self, x: usize, y: usize, blit: Blit) -> bool {
        if x < self.width && y < self.height {
            let (index, pixel) = self.pixel_index(x, y);
            let orig = self.cells[index].bits;
            self.cells[index].bits = match blit {
                Blit::Set => orig | pixel,
                Blit::Unset => orig & !pixel,
                Blit::Toggle => orig ^ pixel,
            };
            true
        } else {
            false
        }
    }

    /// Sets the pixel value at the given coordinates to be the given value. If `value` is
    /// `true`, sets the pixel value to be 1. Otherwise, sets it to 0.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing graphics that can partially clip off screen.
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.transform_pixel(x, y, if value { Blit::Set } else { Blit::Unset });
    }

    /// Flips the pixel value at the given coordinates to be 1.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing graphics that can partially clip off screen.
    pub fn toggle(&mut self, x: usize, y: usize) {
        self.transform_pixel(x, y, Blit::Toggle);
    }

    /// Converts the screen to a utf-8 sequence of bytes that can be rendered in a terminal.
    /// Includes newlines in its output.
    pub fn rasterize(&self) -> Vec<u8> {
        // additional + height given for newline chars
        let mut buf = vec![0; self.cells.len() * BRAILLE_UTF8_BYTES + self.cell_height()];
        for y in 0..self.cell_height() {
            for x in 0..self.cell_width() {
                let i = y * self.cell_width() + x;
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
            screen.set(i, 0, true);
            screen.set(i, 7, true);
            screen.set(0, i, true);
            screen.set(7, i, true);
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
        screen.set(0, 0, true);
        assert!(screen.pixel_at(0, 0));
        screen.set(0, 0, true);
        assert!(screen.pixel_at(0, 0));
        screen.set(0, 0, false);
        assert!(!screen.pixel_at(0, 0));
        screen.set(0, 0, false);
        assert!(!screen.pixel_at(0, 0));
        screen.toggle(0, 0);
        assert!(screen.pixel_at(0, 0));
        screen.toggle(0, 0);
        assert!(!screen.pixel_at(0, 0));
    }
}
