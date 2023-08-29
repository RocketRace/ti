use crossterm::event::{self, KeyCode};

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

impl Event {
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
