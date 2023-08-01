//! Interactions with the terminal screen.
//!
//! Contains the [`Screen`] type and its public interface.

use std::io::{self, stdout, Write};

use crossterm::{
    cursor::{MoveTo, MoveToColumn, MoveToRow},
    style::SetForegroundColor,
    terminal::EnterAlternateScreen,
    ExecutableCommand, QueueableCommand,
};

use crate::{
    cell::{Cell, BRAILLE_UTF8_BYTES, PIXEL_HEIGHT, PIXEL_WIDTH},
    color::Color,
    sprite::Sprite,
};

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

/// Type used to write to the screen. Contains public methods
/// to write pixels and sprites to the screen, as well as colors.
///
/// The point (0, 0) represents the top left pixel of the screen.
///
/// The [`Screen::rasterize`] method can be used to generate
/// bytes that can be written to a terminal.
pub struct Screen {
    cells: Vec<Cell>,
    deltas: Vec<Option<Cell>>,
    colors: Vec<Option<Color>>,
    width: usize,
    height: usize,
}

impl Screen {
    /// Create a new empty screen with the given dimensions in cells.
    pub fn new_cells(width: usize, height: usize) -> Self {
        Self {
            cells: vec![Cell::empty(); width * height],
            deltas: vec![None; width * height],
            colors: vec![None; width * height],
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

    fn cell_position(&self, index: usize) -> (usize, usize) {
        (index % self.width(), index / self.width())
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
            let old = self.cells[index];
            let new = Cell::new(match blit {
                Blit::Set => cell.bits,
                Blit::Unset => !cell.bits,
                Blit::Add => old.bits | cell.bits,
                Blit::Subtract => old.bits & !cell.bits,
                Blit::Toggle => old.bits ^ cell.bits,
            });
            self.deltas[index] = Some(new);
            self.cells[index] = new;
            true
        } else {
            false
        }
    }

    pub fn draw_cell_color(&mut self, color: Color, x: usize, y: usize) -> bool {
        if x < self.width() && y < self.height() {
            let index = self.cell_index(x, y);
            self.colors[index] = Some(color);
            true
        } else {
            false
        }
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
        let x_cell = x / PIXEL_WIDTH;
        let y_cell = y / PIXEL_HEIGHT;
        // We don't want to influence the other bits
        let blit = match blit {
            Blit::Unset => Blit::Subtract,
            Blit::Set => Blit::Add,
            blit => blit,
        };
        let Some(cell) = Cell::from_bit_position(x % PIXEL_WIDTH, y % PIXEL_HEIGHT) else { unreachable!() };
        self.draw_cell(cell, x_cell, y_cell, blit)
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
                & if let Some(color) = cell.color {
                    self.draw_cell_color(color, x_cell, y_cell)
                } else {
                    true
                }
        })
    }

    /// Sets the pixel value at the given coordinates to be the given value. If `value` is
    /// `true`, sets the pixel value to be 1. Otherwise, sets it to 0.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing sprites that can partially clip off screen.
    pub fn set_pixel(&mut self, x: usize, y: usize, value: bool) {
        self.draw_pixel(x, y, if value { Blit::Add } else { Blit::Subtract });
    }

    /// Flips the pixel value at the given coordinates to be 1.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing sprites that can partially clip off screen.
    pub fn toggle_pixel(&mut self, x: usize, y: usize) {
        self.draw_pixel(x, y, Blit::Toggle);
    }

    /// Clears the whole screen, setting it to empty.
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.bits = 0;
        }
        for delta in &mut self.deltas {
            *delta = Some(Cell::empty())
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
        let Ok(s) = String::from_utf8(buf) else { unreachable!() };
        s
    }

    pub fn render_screen(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen)?;
        stdout.queue(MoveTo(0, 0))?;
        let mut cur_x = 0;
        let mut cur_y = 0;
        let mut cur_color = None;
        for (i, (&delta, &color)) in self.deltas.iter().zip(self.colors.iter()).enumerate() {
            if let Some(cell) = delta {
                let (x, y) = self.cell_position(i);
                match (x == cur_x, y == cur_y) {
                    (true, true) => (),
                    (true, false) => {
                        stdout.queue(MoveToRow(y as u16))?;
                    }
                    (false, true) => {
                        stdout.queue(MoveToColumn(x as u16))?;
                    }
                    (false, false) => {
                        stdout.queue(MoveTo(x as u16, y as u16))?;
                    }
                }
                if color != cur_color {
                    if let Some(color) = color {
                        stdout.queue(SetForegroundColor(color))?;
                    }
                    cur_color = color;
                }
                stdout.write_all(&cell.to_braille_utf8())?;
                cur_x = x + 1;
                cur_y = y;
            }
        }
        stdout.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel_at(screen: &Screen, x: usize, y: usize) -> bool {
        let index = screen.cell_index(x / PIXEL_WIDTH, y / PIXEL_HEIGHT);
        let x_pixel = x % PIXEL_WIDTH;
        let y_pixel = y % PIXEL_HEIGHT;
        let pixel = 1 << ((y_pixel * PIXEL_WIDTH) + x_pixel);
        screen.cells[index].bits & pixel != 0
    }

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
        assert!(!pixel_at(&screen, 0, 0));
        screen.set_pixel(0, 0, true);
        assert!(pixel_at(&screen, 0, 0));
        screen.set_pixel(0, 0, true);
        assert!(pixel_at(&screen, 0, 0));
        screen.set_pixel(0, 0, false);
        assert!(!pixel_at(&screen, 0, 0));
        screen.set_pixel(0, 0, false);
        assert!(!pixel_at(&screen, 0, 0));
        screen.toggle_pixel(0, 0);
        assert!(pixel_at(&screen, 0, 0));
        screen.toggle_pixel(0, 0);
        assert!(!pixel_at(&screen, 0, 0));
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
        let sprite = Sprite::from_braille_string(&["⣿"], None).unwrap();
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
        let sprite = Sprite::from_braille_string(&["⣿"], None).unwrap();
        screen.draw_sprite(&sprite, 0, 0, Blit::Set);
        screen.draw_sprite(&sprite, 1, 1, Blit::Toggle);
        assert_eq!(screen.rasterize(), "⡏⡆\n⠈⠁\n");
    }

    #[test]
    fn draw_monochrome_sprite() {
        let mut screen = Screen::new_cells(3, 2);
        let s = &["⢰⣶⡆", "⠸⠿⠇"];
        let sprite = Sprite::from_braille_string(s, None).unwrap();
        screen.draw_sprite(&sprite, 0, 0, Blit::Set);
        eprintln!("{}", screen.rasterize());
        assert_eq!(screen.rasterize(), "⢰⣶⡆\n⠸⠿⠇\n");
        screen.draw_sprite(&sprite, 1, 1, Blit::Toggle);
        eprintln!("{}", screen.rasterize());
        assert_eq!(screen.rasterize(), "⢰⠒⢢\n⠸⣀⣸\n");
        screen.draw_sprite(&sprite, 2, 4, Blit::Unset);
    }
}
