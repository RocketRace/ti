//! Methods for adding color when drawing types such as [`crate::sprite::Sprite`] and [`crate::cell::Cell`] to the screen.
//!
//! This uses [`crossterm::style::Color`] to represent ANSI terminal colors.

use std::cmp::Ordering;

use crossterm::style;

use crate::cell::Cell;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color(pub u8);

// RGB, GREYSCALE: These are the values most terminals seem to use
// RGB must begin with 0 and end with 255
const RGB: [u8; 6] = [0, 95, 135, 175, 215, 255];
const GREYSCALE: [u8; 24] = {
    let mut x = [0u8; 24];
    let mut i = 0u8;
    while i < 24 {
        x[i as usize] = i * 10 + 8;
        i += 1;
    }
    x
};

/// Picks the ANSI RGB component that's closest to the input
fn interpolate_component(scale: &[u8], target: u8) -> u8 {
    let next_ansi = scale
        .iter()
        .position(|&next| next >= target)
        .unwrap_or(scale.len() - 1) as u8;
    let next = scale[next_ansi as usize];

    match target.cmp(&next) {
        // either that the target was matched right on, or the default value at the end of the
        // scale was used (which is as close as it can get)
        Ordering::Greater | Ordering::Equal => next_ansi,
        Ordering::Less => {
            // implies the first value of the scale was nonzero, and the target was below it
            if next_ansi == 0 {
                next_ansi
            } else {
                let prev_ansi = next_ansi - 1;
                let prev = scale[prev_ansi as usize];
                // simple linear distance
                if target - prev > next - target {
                    next_ansi
                } else {
                    prev_ansi
                }
            }
        }
    }
}

/// diagonal distance from a to b
fn dist(a: (u8, u8, u8), b: (u8, u8, u8)) -> f32 {
    let (a_r, a_g, a_b) = a;
    let (b_r, b_g, b_b) = b;
    ((a_r as f32 - b_r as f32).abs().powi(3)
        + (a_g as f32 - b_g as f32).abs().powi(3)
        + (a_b as f32 - b_b as f32).abs().powi(3))
    .cbrt()
}

macro_rules! define_standard_colors {
    ($($num:literal $name:ident $str:literal $($note:literal)?),+) => {
        $(
            #[doc = "The ANSI standard"]
            #[doc = $str]
            #[doc = "color. Its appearance varies across terminals and themes."]
            $(#[doc = $note])?
            pub const $name: Color = Color::new($num);
        )+
    };
}

/// This module contains the 16 ANSI standard colors, supported by almost all terminals. If you want your program to be
/// maximally visible on all terminals, and don't mind the colors looking slightly different, you can use these.
pub mod standard {
    use super::Color;
    define_standard_colors! {
        0 BLACK "black",
        1 RED "red",
        2 GREEN "green",
        3 YELLOW "yellow",
        4 BLUE "blue",
        5 MAGENTA "magenta",
        6 CYAN "cyan",
        7 WHITE "white" "Note: This color is not equivalent to RGB white on most terminals.",
        8 BRIGHT_BLACK "bright black",
        9 BRIGHT_RED "bright red",
        10 BRIGHT_GREEN "bright green",
        11 BRIGHT_YELLOW "bright yellow",
        12 BRIGHT_BLUE "bright blue",
        13 BRIGHT_MAGENTA "bright magenta",
        14 BRIGHT_CYAN "bright cyan",
        15 BRIGHT_WHITE "bright white"
    }
}

use standard::*;

impl Color {
    /// Creates a new color from an 8-bit ANSI color value.
    pub const fn new(color: u8) -> Self {
        Self(color)
    }
    /// Returns an ANSI color that is visually similar to the specified
    /// RGB value. This will not always be accurate, because there are only
    /// 256 ANSI colors compared to 256^3 RGB values.
    ///
    /// The process used to approximate a color is as follows:
    /// * Find the ANSI color that is componentwise closest to the RGB triplet
    ///   using linear distance for each component
    /// * Find the ANSI greyscale value that is closest to the RGB triplet when
    ///   converted to greyscale, using a simple sum of components
    /// * Pick the option out of these two that minimizes the distance to the input
    ///   color, using cartesian distance as a metric. (Prefer the componentwise
    ///   option on a tie.)
    ///
    /// This is a very rudimentary method but computationally very simple.
    pub fn from_rgb_approximate(r: u8, g: u8, b: u8) -> Self {
        let components = Self::from_ansi_components(
            interpolate_component(&RGB, r),
            interpolate_component(&RGB, g),
            interpolate_component(&RGB, b),
        );
        let greyscale = Self::from_ansi_greyscale(interpolate_component(
            &GREYSCALE,
            ((r as u16 + g as u16 + b as u16) / 3) as u8,
        ));

        let components_rgb = components.to_rgb_approximate();
        let greyscale_rgb = greyscale.to_rgb_approximate();

        if dist(components_rgb, (r, g, b)) > dist(greyscale_rgb, (r, g, b)) {
            greyscale
        } else {
            components
        }
    }
    /// This is a simple algorithm that returns the closest ANSI standard color to the given RGB triplet.
    /// It picks the color that is closest in cartesian distance to the input value, in the RGB cube.
    ///
    /// It is similar to [`Color::from_rgb_approximate()`] but with lower resolution. Like
    /// [`Color::to_rgb_approximate()`], this models standard colors as RGB triplets with component values
    /// in {0, 128, 255}. (The one exception to this is [`standard::WHITE`], defined as RGB (192, 192, 192)
    /// that is similar to other terminals.)
    pub fn standard_color_approximate(r: u8, g: u8, b: u8) -> Self {
        let colors = [
            BLACK,
            RED,
            GREEN,
            GREEN,
            YELLOW,
            BLUE,
            MAGENTA,
            CYAN,
            WHITE,
            BRIGHT_BLACK,
            BRIGHT_RED,
            BRIGHT_GREEN,
            BRIGHT_GREEN,
            BRIGHT_YELLOW,
            BRIGHT_BLUE,
            BRIGHT_MAGENTA,
            BRIGHT_CYAN,
            BRIGHT_WHITE,
        ];

        colors
            .into_iter()
            .min_by(|&x, &y| {
                let rgb_x = x.to_rgb_approximate();
                let rgb_y = y.to_rgb_approximate();
                dist(rgb_x, (r, g, b)).total_cmp(&dist(rgb_y, (r, g, b)))
            })
            .unwrap()
    }
    /// Returns a new color with the specified from red, green and blue components.
    /// Each component may span from 0 to 5 (inclusive). If any values are higher, they
    /// are clipped to the maximum value (5).
    pub fn from_ansi_components(r: u8, g: u8, b: u8) -> Self {
        Self(r.min(5) * 36 + g.min(5) * 6 + b.min(5) + 16)
    }
    /// Returns a new color with the specified greyscale value. The value may be
    /// between 0 and 23 (inclusive), and represents a scale from black to white.
    /// If any values are higher, they are clipped to the maximum value (23).
    ///
    /// Note that most terminals will not represent 0 with black and 23 with white;
    /// consider using `from_ansi_rgb(0, 0, 0)` and `from_ansi_rgb(5, 5, 5)` instead.
    pub fn from_ansi_greyscale(step: u8) -> Self {
        Self(232 + step.min(23))
    }
    /// Returns the approximate RGB color associated with this ANSI color.
    ///
    /// This is not always accurate; terminals may always choose to theme
    /// ANSI colors differently. In particular, the standard and high-intensity
    /// ANSI colors (color values from 0 to 15) are often altered by custom themes.
    pub fn to_rgb_approximate(self) -> (u8, u8, u8) {
        match self.0 {
            // The standard colors are simple approximations, because every terminal does it differently.
            // This is a particularly simple choice of colors, following the windows XP console.
            0 => (0, 0, 0),
            1 => (128, 0, 0),
            2 => (0, 128, 0),
            3 => (128, 128, 0),
            4 => (0, 0, 128),
            5 => (128, 0, 128),
            6 => (0, 128, 128),
            7 => (192, 192, 192),
            8 => (128, 128, 128),
            9 => (255, 0, 0),
            10 => (0, 255, 0),
            11 => (255, 255, 0),
            12 => (0, 0, 255),
            13 => (255, 0, 255),
            14 => (0, 255, 255),
            15 => (255, 255, 255),
            // 3-component (RGB) colors
            16..=231 => {
                let offset = self.0 - 16;
                let r = (offset / 36) % 6;
                let g = (offset / 6) % 6;
                let b = offset % 6;
                (RGB[r as usize], RGB[g as usize], RGB[b as usize])
            }
            // Greyscale colors
            232..=255 => {
                let step = self.0 - 232;
                (
                    GREYSCALE[step as usize],
                    GREYSCALE[step as usize],
                    GREYSCALE[step as usize],
                )
            }
        }
    }

    /// Returns the equivalent crossterm color, for the purposes of integration
    pub fn to_crossterm_color(self) -> style::Color {
        style::Color::AnsiValue(self.0)
    }
}

pub struct ColorFlags {
    /// When `true`, color is applied when the cell is drawn, even if the cell is empty.
    ///
    /// Otherwise, color is only applied to nonempty cells.
    pub apply_on_empty: bool,
}

/// A [`Cell`] with associated [`Color`] data.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ColoredCell {
    pub cell: Cell,
    pub color: Option<Color>,
}

impl ColoredCell {
    /// Creates a new [`ColoredCell`] from parameters
    pub fn new(cell: Cell, color: Option<Color>) -> Self {
        Self { cell, color }
    }

    /// Combines this cell's pixel data with the argument [`Cell`] with a bitwise OR.
    pub fn merge_cell(&mut self, cell: Cell) {
        self.cell = self.cell | cell;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ansi_components() {
        assert_eq!(Color::from_ansi_components(1, 2, 3), Color::new(67));
        assert_eq!(Color::from_ansi_components(0, 0, 0), Color::new(16));
        assert_eq!(Color::from_ansi_components(5, 5, 5), Color::new(231));
        assert_eq!(Color::from_ansi_components(6, 7, 8), Color::new(231));
    }
    #[test]
    fn test_from_ansi_greyscale() {
        assert_eq!(Color::from_ansi_greyscale(0), Color::new(232));
        assert_eq!(Color::from_ansi_greyscale(1), Color::new(233));
        assert_eq!(Color::from_ansi_greyscale(10), Color::new(242));
        assert_eq!(Color::from_ansi_greyscale(23), Color::new(255));
        assert_eq!(Color::from_ansi_greyscale(100), Color::new(255));
    }

    #[test]
    fn test_component_approximation_exact() {
        assert_eq!(
            Color::from_rgb_approximate(0, 0, 0),
            Color::from_ansi_components(0, 0, 0)
        );
        assert_eq!(
            Color::from_rgb_approximate(255, 255, 255),
            Color::from_ansi_components(5, 5, 5)
        );
        assert_eq!(
            Color::from_rgb_approximate(95, 135, 215),
            Color::from_ansi_components(1, 2, 4)
        )
    }
    #[test]
    fn test_greyscale_approximation_exact() {
        assert_eq!(
            Color::from_rgb_approximate(8, 8, 8),
            Color::from_ansi_greyscale(0)
        );
        assert_eq!(
            Color::from_rgb_approximate(58, 58, 58),
            Color::from_ansi_greyscale(5)
        );
        assert_eq!(
            Color::from_rgb_approximate(238, 238, 238),
            Color::from_ansi_greyscale(23)
        )
    }

    #[test]
    fn test_component_approximation() {
        assert_eq!(
            Color::from_rgb_approximate(1, 0, 0),
            Color::from_ansi_components(0, 0, 0)
        );
        assert_eq!(
            Color::from_rgb_approximate(129, 251, 2),
            Color::from_ansi_components(2, 5, 0)
        );
    }
    #[test]
    fn test_greyscale_approximation() {
        assert_eq!(
            Color::from_rgb_approximate(64, 59, 62),
            Color::from_ansi_greyscale(5)
        );
        assert_eq!(
            Color::from_rgb_approximate(240, 241, 242),
            Color::from_ansi_greyscale(23)
        );
    }
    #[test]
    fn test_greyscale_incrementing() {
        let colors: Vec<_> = (0..24).map(Color::from_ansi_greyscale).collect();
        let mut sorted = colors.clone();
        sorted.sort_by(|a, b| a.to_rgb_approximate().0.cmp(&b.to_rgb_approximate().0));
        assert_eq!(colors, sorted)
    }

    #[test]
    fn test_standard_color_approx() {
        assert_eq!(Color::standard_color_approximate(12, 8, 3), standard::BLACK);
        assert_eq!(
            Color::standard_color_approximate(124, 45, 55),
            standard::RED
        );
        assert_eq!(
            Color::standard_color_approximate(184, 125, 200),
            standard::WHITE
        );
        assert_eq!(
            Color::standard_color_approximate(244, 225, 240),
            standard::BRIGHT_WHITE
        );
        assert_eq!(
            Color::standard_color_approximate(254, 1, 128),
            standard::MAGENTA
        );
        assert_eq!(
            Color::standard_color_approximate(254, 1, 126),
            standard::BRIGHT_RED
        );
    }
}
