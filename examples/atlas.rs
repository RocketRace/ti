use std::{io, time::Duration};

use ti::{
    screen::Screen,
    sprite::{Atlas, ColorMode::Rgb},
};

fn main() -> io::Result<()> {
    let mut screen = Screen::new_pixels(18, 18);
    let atlas = Atlas::open("examples/heart.png", Rgb, true).expect("couldn't read atlas");

    let view = atlas.sprite(5, 0, 6, 6, 4, 0);
    screen.draw_sprite(&view, 0, 0, ti::screen::Blit::Set);

    screen.enter_screen()?;
    screen.render_screen()?;
    std::thread::sleep(Duration::from_secs(3));
    screen.exit_screen()?;
    Ok(())
}
