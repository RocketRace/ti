//! Interactions with the terminal screen.
//!
//! Contains the [`Screen`] type and its public interface.

use std::{
    io::{self, stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor::{Hide, MoveTo, MoveToColumn, MoveToRow, Show},
    style::SetForegroundColor,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand, QueueableCommand,
};

use crate::{
    cell::{Cell, BRAILLE_UTF8_BYTES, PIXEL_HEIGHT, PIXEL_WIDTH},
    color::Color,
    sprite::Sprite,
    units::{cell_length, from_index, index, pos_components, px_offset},
};

/// A blit type used to select the type of operation
/// when writing to the screen. In the case of single pixels,
/// this is used to determine whether the output pixel is
/// set to 1, set to 0 or flipped.
///
/// # Examples
///
/// ```
/// use ti::screen::{Screen, Blit};
///
/// let mut screen = Screen::new_cells(1, 1);
///
/// // Override regardless of previous
/// screen.draw_pixel(0, 0, Blit::Set);
/// assert_eq!(screen.get_pixel(0, 0), Some(true));
///
/// // Override (negatively) regardless of previous
/// screen.draw_pixel(0, 0, Blit::Unset);
/// assert_eq!(screen.get_pixel(0, 0), Some(false));
///
/// // Set only true values in input to true
/// screen.draw_pixel(0, 0, Blit::Add);
/// assert_eq!(screen.get_pixel(0, 0), Some(true));
///
/// // Set only true values in input to false
/// screen.draw_pixel(0, 0, Blit::Subtract);
/// assert_eq!(screen.get_pixel(0, 0), Some(false));
///
/// // Flip true values in input
/// screen.draw_pixel(0, 0, Blit::Toggle);
/// assert_eq!(screen.get_pixel(0, 0), Some(true));
/// ```
#[derive(Clone, Copy)]
pub enum Blit {
    /// Sets the output to 1 where the input is set, and 0 elsewhere.
    Set,
    /// Sets the output to 0 where the input is set, and 1 elsewhere.
    Unset,
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
///
/// # Examples
///
/// ```
/// use ti::screen::Screen;
///
/// let screen = Screen::new_cells(2, 2);
/// ```
pub struct Screen {
    cells: Vec<Cell>,
    deltas: Vec<Option<Cell>>,
    colors: Vec<Option<Color>>,
    width: u16,
    height: u16,
}

impl Screen {
    /// Create a new empty screen with the given dimensions in cells.
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    ///
    /// let screen = Screen::new_cells(2, 3);
    /// assert_eq!(screen.width(), 2);
    /// assert_eq!(screen.height(), 3);
    /// ```
    pub fn new_cells(width: u16, height: u16) -> Self {
        Self {
            cells: vec![Cell::empty(); cell_length(width, height)],
            deltas: vec![None; cell_length(width, height)],
            colors: vec![None; cell_length(width, height)],
            width,
            height,
        }
    }
    /// Create a new empty screen with the given dimensions in pixels.
    /// The resulting width and height are rounded up to the nearest multiple of
    /// [`PIXEL_WIDTH`] and [`PIXEL_HEIGHT`].
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    ///
    /// let screen = Screen::new_pixels(3, 10);
    /// assert_eq!(screen.width(), 2);
    /// assert_eq!(screen.height(), 3);
    /// ```
    pub fn new_pixels(width: u16, height: u16) -> Self {
        Self::new_cells(
            (width + PIXEL_WIDTH as u16 - 1) / PIXEL_WIDTH as u16,
            (height + PIXEL_HEIGHT as u16 - 1) / PIXEL_HEIGHT as u16,
        )
    }

    /// Get the width of the screen, in number of cells.
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    ///
    /// let screen = Screen::new_cells(2, 3);
    /// assert_eq!(screen.width(), 2);
    /// ```
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// Get the height of the screen, in number of cells.
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    ///
    /// let screen = Screen::new_cells(2, 3);
    /// assert_eq!(screen.height(), 3);
    /// ```
    pub const fn height(&self) -> u16 {
        self.height
    }

    /// Compute the array index of a cell at position (x, y).
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    ///
    /// let screen = Screen::new_cells(4, 2);
    /// let index = screen.index(0, 1);
    /// assert_eq!(index, 4);
    /// ```
    pub const fn index(&self, x: u16, y: u16) -> usize {
        index(x, y, self.width())
    }

    /// Compute the position (x, y) of a cell at the given array index.
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    ///
    /// let screen = Screen::new_cells(4, 2);
    /// assert_eq!(screen.from_index(6), (2, 1));
    /// ```
    pub const fn from_index(&self, i: usize) -> (u16, u16) {
        from_index(i, self.width())
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
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::{Screen, Blit};
    /// use ti::cell::Cell;
    ///
    /// let mut screen = Screen::new_cells(4, 2);
    /// let cell = Cell::from_braille('⢌').unwrap();
    ///
    /// assert!(screen.draw_cell(cell, 0, 0, Blit::Set));
    /// assert_eq!(screen.get_cell(0, 0), Some(cell));
    ///
    /// assert!(!screen.draw_cell(cell, 99, 99, Blit::Set));
    /// assert_eq!(screen.get_cell(0, 0), Some(cell));
    ///
    /// assert!(screen.draw_cell(cell, 0, 0, Blit::Toggle));
    /// assert_eq!(screen.get_cell(0, 0), Some(Cell::empty()));
    /// ```
    pub fn draw_cell(&mut self, cell: Cell, x: u16, y: u16, blit: Blit) -> bool {
        if x < self.width() && y < self.height() {
            let index = self.index(x, y);
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

    /// Sets the color of the cell at the specified position.
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    /// use ti::color::Color;
    ///
    /// let mut screen = Screen::new_cells(2, 1);
    /// let color = Color::new(23);
    /// assert!(screen.draw_cell_color(color, 1, 0));
    /// assert_eq!(screen.get_color(1, 0), Some(color));
    /// ```
    pub fn draw_cell_color(&mut self, color: Color, x: u16, y: u16) -> bool {
        if x < self.width() && y < self.height() {
            let i = self.index(x, y);
            self.colors[i] = Some(color);
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
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::{Screen, Blit};
    ///
    /// let mut screen = Screen::new_pixels(1, 1);
    /// assert!(screen.set_pixel(0, 0, true));
    /// assert_eq!(screen.get_pixel(0, 0), Some(true));
    /// ```
    pub fn draw_pixel(&mut self, x: u16, y: u16, blit: Blit) -> bool {
        let ((x_cell, x_pixel), (y_cell, y_pixel)) = pos_components(x, y);
        // We don't want to influence the other bits
        let blit = match blit {
            Blit::Unset => Blit::Subtract,
            Blit::Set => Blit::Add,
            blit => blit,
        };
        let Some(cell) = Cell::from_bit_position(x_pixel, y_pixel) else { unreachable!() };
        self.draw_cell(cell, x_cell, y_cell, blit)
    }

    /// Returns the cell value at the specified (cell) coordinates. Returns None if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    /// use ti::cell::Cell;
    ///
    /// let screen = Screen::new_cells(2, 2);
    /// assert_eq!(screen.get_cell(4, 6), None);
    /// assert_eq!(screen.get_cell(0, 1), Some(Cell::empty()));
    /// ```
    pub fn get_cell(&self, x: u16, y: u16) -> Option<Cell> {
        if x < self.width() && y < self.height() {
            let index = self.index(x, y);
            Some(self.cells[index])
        } else {
            None
        }
    }

    /// Returns the color at of the cell at the specified coordinates. Returns None if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    /// use ti::color::Color;
    ///
    /// let mut screen = Screen::new_cells(2, 2);
    /// let color = Color::new(123);
    /// assert_eq!(screen.get_color(999, 999), None);
    /// screen.draw_cell_color(color, 0, 0);
    /// assert_eq!(screen.get_color(0, 0), Some(color));
    /// ```
    pub fn get_color(&self, x: u16, y: u16) -> Option<Color> {
        if x < self.width() && y < self.height() {
            let index = self.index(x, y);
            self.colors[index]
        } else {
            None
        }
    }

    /// Returns the pixel value at the specified (pixel) coordinates. Returns None if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use ti::screen::Screen;
    ///
    /// let mut screen = Screen::new_cells(1, 1);
    /// screen.set_pixel(0, 0, true);
    /// assert_eq!(screen.get_pixel(0, 0), Some(true));
    /// assert_eq!(screen.get_pixel(1, 0), Some(false));
    /// assert_eq!(screen.get_pixel(99, 0), None);
    /// ```
    pub fn get_pixel(&self, x: u16, y: u16) -> Option<bool> {
        let ((x_cell, x_pixel), (y_cell, y_pixel)) = pos_components(x, y);
        let Some(mask) = Cell::from_bit_position(x_pixel, y_pixel) else { unreachable!() };
        self.get_cell(x_cell, y_cell)
            .map(|cell| cell.bits & mask.bits != 0)
    }

    /// Draws a single sprite to the screen. The x and y coordinates are specified in pixels,
    /// and refer to the top left corner of the sprite.
    ///
    /// Returns `false` if any part of the sprite was clipped by the screen boundaries, `true` otherwise.
    pub fn draw_sprite(&mut self, sprite: &Sprite, x_pixel: u16, y_pixel: u16, blit: Blit) -> bool {
        let ((dx_cell, x_px), (dy_cell, y_px)) = pos_components(x_pixel, y_pixel);
        let offset = px_offset(x_px, y_px);
        let data = &sprite.offsets[offset as usize];
        data.iter().enumerate().fold(true, |acc, (i, cell)| {
            let (x_cell, y_cell) = sprite.from_index(i, offset);
            let x = x_cell + dx_cell;
            let y = y_cell + dy_cell;
            if !cell.cell.is_empty() {
                let drawn = self.draw_cell(cell.cell, x, y, blit);
                if let Some(color) = cell.color {
                    let colored = self.draw_cell_color(color, x, y);
                    acc & drawn & colored
                } else {
                    acc & drawn
                }
            } else {
                acc
            }
        })
    }

    /// Sets the pixel value at the given coordinates to be the given value. If `value` is
    /// `true`, sets the pixel value to be 1. Otherwise, sets it to 0.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing sprites that can partially clip off screen.
    pub fn set_pixel(&mut self, x: u16, y: u16, value: bool) -> bool {
        self.draw_pixel(x, y, if value { Blit::Add } else { Blit::Subtract })
    }

    /// Flips the pixel value at the given coordinates to be 1.
    ///
    /// **Ignores** out-of-bounds input.
    /// This may be preferred when drawing sprites that can partially clip off screen.
    pub fn toggle_pixel(&mut self, x: u16, y: u16) -> bool {
        self.draw_pixel(x, y, Blit::Toggle)
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
        let mut buf = vec![0; self.cells.len() * BRAILLE_UTF8_BYTES + self.height() as usize];
        for y in 0..self.height() {
            for x in 0..self.width() {
                let i = self.index(x, y);
                let y = y as usize;
                // extra newlines also counted here
                buf[i * 3 + y..(i + 1) * 3 + y].copy_from_slice(&self.cells[i].to_braille_utf8());
            }
            let y = y as usize;
            buf[(y + 1) * (self.width() as usize * 3 + 1) - 1] = b'\n';
        }
        let Ok(s) = String::from_utf8(buf) else { unreachable!() };
        s
    }

    /// Enters the terminal's alternate screen.
    pub fn enter_screen(&self) -> io::Result<()> {
        stdout().execute(EnterAlternateScreen)?.execute(Hide)?;
        Ok(())
    }

    /// Exit's the terminal's alternate screen.
    pub fn exit_screen(&self) -> io::Result<()> {
        stdout().execute(LeaveAlternateScreen)?.execute(Show)?;
        Ok(())
    }

    /// Renders the current state of the screen buffer to the terminal.
    pub fn render_screen(&mut self) -> io::Result<()> {
        let mut stdout = stdout();
        self.write_screen_to(&mut stdout)
    }

    /// Renders the current state of the screen to some writable buffer.
    pub fn write_screen_to<B: Write>(&mut self, buf: &mut B) -> io::Result<()> {
        buf.queue(MoveTo(0, 0))?;
        let mut cur_x = 0;
        let mut cur_y = 0;
        let mut cur_color = None;
        for (i, (&delta, &color)) in self.deltas.iter().zip(self.colors.iter()).enumerate() {
            if let Some(cell) = delta {
                let (x, y) = self.from_index(i);
                match (x == cur_x, y == cur_y) {
                    (true, true) => (),
                    (true, false) => {
                        buf.queue(MoveToRow(y))?;
                    }
                    (false, true) => {
                        buf.queue(MoveToColumn(x))?;
                    }
                    (false, false) => {
                        buf.queue(MoveTo(x, y))?;
                    }
                }
                if color != cur_color {
                    if let Some(color) = color {
                        buf.queue(SetForegroundColor(color.to_crossterm_color()))?;
                    }
                    cur_color = color;
                }
                buf.write_all(&cell.to_braille_utf8())?;
                cur_x = x + 1;
                cur_y = y;
            }
        }
        buf.flush()?;
        Ok(())
    }

    /// Resets the working state of the screen.
    pub fn reset_deltas(&mut self) {
        self.deltas.fill(None);
        self.colors.fill(None);
    }

    /// Enters the rendering loop. Renders 60 times a second.
    pub fn start_loop<F: FnMut(&mut Self) -> io::Result<()>>(
        &mut self,
        frame_rate: u8,
        mut tick: F,
    ) -> io::Result<()> {
        self.enter_screen()?;
        while let Ok(()) = tick(self) {
            self.render_screen()?;
            self.reset_deltas();
            std::thread::sleep(Duration::from_secs_f64(1. / frame_rate as f64))
        }
        self.exit_screen()?;
        Ok(())
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
        assert_eq!(screen.get_pixel(0, 0), Some(false));
        screen.set_pixel(0, 0, true);
        assert_eq!(screen.get_pixel(0, 0), Some(true));
        screen.set_pixel(0, 0, true);
        assert_eq!(screen.get_pixel(0, 0), Some(true));
        screen.set_pixel(0, 0, false);
        assert_eq!(screen.get_pixel(0, 0), Some(false));
        screen.set_pixel(0, 0, false);
        assert_eq!(screen.get_pixel(0, 0), Some(false));
        screen.toggle_pixel(0, 0);
        assert_eq!(screen.get_pixel(0, 0), Some(true));
        screen.toggle_pixel(0, 0);
        assert_eq!(screen.get_pixel(0, 0), Some(false));
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
