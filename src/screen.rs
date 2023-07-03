use crate::cell::{Cell, BRAILLE_UTF8_BYTES};

pub struct Screen {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

#[derive(Clone, Copy)]
pub enum Blit {
    Set,
    Unset,
    Toggle,
}

// https://github.com/rust-lang/rust/issues/88581
fn div_ceil(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

impl Screen {
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

    pub fn cell_height(&self) -> usize {
        div_ceil(self.height, Cell::PIXEL_HEIGHT)
    }

    pub fn cell_width(&self) -> usize {
        div_ceil(self.width, Cell::PIXEL_WIDTH)
    }

    fn pixel_index(&self, x: usize, y: usize) -> (usize, u8) {
        let index = y / Cell::PIXEL_HEIGHT * self.cell_width() + x / Cell::PIXEL_WIDTH;
        let x_pixel = x % Cell::PIXEL_WIDTH;
        let y_pixel = y % Cell::PIXEL_HEIGHT;
        let pixel = 1 << ((y_pixel * Cell::PIXEL_WIDTH) + x_pixel);
        (index, pixel)
    }

    fn transform_pixel(&mut self, x: usize, y: usize, blit: Blit) {
        if x < self.width && y < self.height {
            let (index, pixel) = self.pixel_index(x, y);
            let orig = self.cells[index].0;
            self.cells[index].0 = match blit {
                Blit::Set => orig | pixel,
                Blit::Unset => orig & !pixel,
                Blit::Toggle => orig ^ pixel,
            };
        }
    }

    pub fn pixel_at(&self, x: usize, y: usize) -> bool {
        let (index, pixel) = self.pixel_index(x, y);
        self.cells[index].0 & pixel != 0
    }

    /// Ignores out-of-bounds inputs
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.transform_pixel(x, y, if value { Blit::Set } else { Blit::Unset });
    }

    /// Ignores out-of-bounds inputs
    pub fn toggle(&mut self, x: usize, y: usize) {
        self.transform_pixel(x, y, Blit::Toggle);
    }

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
