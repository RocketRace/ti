//! Key event handling.

use crossterm::event::{self, KeyCode};

/// A keyboard event. Includes most keys on most keyboards, but does not include all keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    Right,
    Left,
    Up,
    Down,
    Char(char),
    Enter,
    Esc,
    Backspace,
    Tab,
}

/// A direction. This is a convenience enum to abstract some of the directionality handling away.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    Up,
    Left,
    Down,
    Right,
}

impl Event {
    /// Returns the direction associated with this event, if any.
    ///
    /// Includes arrow keys, and a configurable keyset for Up, Left, Down, Right.
    ///
    /// A special case of this (using WASD for the directions) is common enough that it has
    /// a special method: [`Event::direction_wasd()`].
    pub fn direction(&self, up: char, left: char, down: char, right: char) -> Option<Direction> {
        match self {
            Event::Up => Some(Direction::Up),
            Event::Left => Some(Direction::Left),
            Event::Down => Some(Direction::Down),
            Event::Right => Some(Direction::Right),
            Event::Char(c) if *c == up => Some(Direction::Up),
            Event::Char(c) if *c == left => Some(Direction::Left),
            Event::Char(c) if *c == down => Some(Direction::Down),
            Event::Char(c) if *c == right => Some(Direction::Right),
            _ => None,
        }
    }
    /// Returns the direction associated with this event, if any.
    ///
    /// Maps arrow keys and WASD keys to their directions.
    pub fn direction_wasd(&self) -> Option<Direction> {
        self.direction('w', 'a', 's', 'd')
    }
    /// Create an event from a crossterm event, if possible.
    pub fn from_crossterm_event(event: event::Event) -> Option<Self> {
        match event {
            event::Event::Key(key) => match key.code {
                KeyCode::Backspace => Some(Event::Backspace),
                KeyCode::Enter => Some(Event::Enter),
                KeyCode::Left => Some(Event::Left),
                KeyCode::Right => Some(Event::Right),
                KeyCode::Up => Some(Event::Up),
                KeyCode::Down => Some(Event::Down),
                KeyCode::Tab => Some(Event::Tab),
                KeyCode::Char(c) => Some(Event::Char(c)),
                KeyCode::Esc => Some(Event::Esc),
                _ => None,
            },
            _ => None,
        }
    }
}
