//! Interactions with the terminal screen.
//!
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
    // updates: Vec<Cell>,
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

impl Screen {
    /// Create a new empty screen with the given dimensions in cells.
    pub fn new_cells(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::default(); width * height],
            // updates: vec![Cell::default(); width * height],
            width,
            height,
        }
    }
    /// Create a new empty screen with the given dimensions in pixels.
    /// The resulting width and height are rounded up to the nearest multiple of
    /// [`PIXEL_WIDTH`] and [`PIXEL_HEIGHT`].
    pub fn new_pixels(width: usize, height: usize) -> Self {
        Self::new_cells(
            (width + PIXEL_WIDTH - 1) / PIXEL_WIDTH,
            (height + PIXEL_HEIGHT - 1) / PIXEL_HEIGHT,
        )
    }

    /// Get the height of the screen, in number of cells.
    pub fn height(&self) -> usize {
        self.height
    }
    /// Get the width of the screen, in number of cells.
    pub fn width(&self) -> usize {
        self.width
    }

    fn cell_index(&self, cell_x: usize, cell_y: usize) -> usize {
        cell_y * self.width() + cell_x
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
        if x / PIXEL_WIDTH < self.width() && y / PIXEL_HEIGHT < self.height() {
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
        if x < self.width() && y < self.height() {
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
        sprite: &Sprite,
        x_pixel: usize,
        y_pixel: usize,
        blit: Blit,
    ) -> bool {
        let offset = (y_pixel % PIXEL_HEIGHT) * PIXEL_WIDTH + (x_pixel % PIXEL_WIDTH);
        let choice = &sprite.offsets[offset];
        let (width, _) = sprite.offset_size(offset);
        choice.iter().enumerate().fold(true, |acc, (i, cell)| {
            let y_cell = y_pixel / PIXEL_HEIGHT + i / width;
            let x_cell = x_pixel / PIXEL_WIDTH + i % width;
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

    /// Clears the whole screen, setting it to white.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.bits = 0;
        }
    }

    /// Converts the screen to a utf-8 sequence of bytes that can be rendered in a terminal.
    /// Includes newlines in its output.
    pub fn rasterize(&self) -> String {
        // additional + height given for newline chars
        let mut buf = vec![0; self.cells.len() * BRAILLE_UTF8_BYTES + self.height()];
        for y in 0..self.height() {
            for x in 0..self.width() {
                let i = self.cell_index(x, y);
                // extra newlines also counted here
                buf[i * 3 + y..(i + 1) * 3 + y].copy_from_slice(&self.cells[i].to_braille_utf8());
            }
            buf[(y + 1) * (self.width() * 3 + 1) - 1] = b'\n';
        }
        String::from_utf8(buf).expect("Unreachable")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_screen_size_pixels() {
        let screen = Screen::new_pixels(16, 24);
        assert_eq!(screen.width(), 8);
        assert_eq!(screen.height(), 6);
    }

    #[test]
    fn odd_screen_size_pixels() {
        let screen = Screen::new_pixels(3, 3);
        assert_eq!(screen.width(), 2);
        assert_eq!(screen.height(), 1);
    }

    #[test]
    fn make_square() {
        let mut screen = Screen::new_pixels(8, 8);
        assert_eq!(screen.width(), 4);
        assert_eq!(screen.height(), 2);
        for i in 0..8 {
            screen.set_pixel(i, 0, true);
            screen.set_pixel(i, 7, true);
            screen.set_pixel(0, i, true);
            screen.set_pixel(7, i, true);
        }
        assert_eq!(&screen.rasterize(), "⡏⠉⠉⢹\n⣇⣀⣀⣸\n");
    }

    #[test]
    fn blit_types() {
        let mut screen = Screen::new_pixels(1, 1);
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
    fn draw_cell() {
        let mut screen = Screen::new_cells(2, 1);
        let cell = Cell::new(0b0011_1100);
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
    fn draw_unaliged_cell() {
        let mut screen = Screen::new_cells(2, 2);
        let sprite = Sprite::from_braille_string(&["⣿"]).unwrap();
        screen.draw_sprite(&sprite, 0, 0, Blit::Set);
        // unicode escapes used because many editors don't like blank characters
        assert_eq!(screen.rasterize(), "⣿\u{2800}\n\u{2800}\u{2800}\n");
        screen.clear();
        screen.draw_sprite(&sprite, 1, 1, Blit::Set);
        assert_eq!(screen.rasterize(), "⢰⡆\n⠈⠁\n");
        screen.clear();
        screen.draw_sprite(&sprite, 2, 2, Blit::Set);
        assert_eq!(screen.rasterize(), "\u{2800}⣤\n\u{2800}⠛\n");
    }

    #[test]
    fn toggle_unaligned_cell() {
        let mut screen = Screen::new_cells(2, 2);
        let sprite = Sprite::from_braille_string(&["⣿"]).unwrap();
        screen.draw_sprite(&sprite, 0, 0, Blit::Set);
        screen.draw_sprite(&sprite, 1, 1, Blit::Toggle);
        assert_eq!(screen.rasterize(), "⡏⡆\n⠈⠁\n");
    }

    #[test]
    fn draw_monochrome_sprite() {
        let mut screen = Screen::new_cells(3, 2);
        let s = &["⢰⣶⡆", "⠸⠿⠇"];
        let sprite = Sprite::from_braille_string(s).unwrap();
        screen.draw_sprite(&sprite, 0, 0, Blit::Set);
        eprintln!("{}", screen.rasterize());
        assert_eq!(screen.rasterize(), "⢰⣶⡆\n⠸⠿⠇\n");
        screen.draw_sprite(&sprite, 1, 1, Blit::Toggle);
        eprintln!("{}", screen.rasterize());
        assert_eq!(screen.rasterize(), "⢰⠒⢢\n⠸⣀⣸\n");
        screen.draw_sprite(&sprite, 2, 4, Blit::Unset);
    }
}
