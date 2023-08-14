use std::{io, time::Duration};

use ti::{
    color::Color,
    screen::{Blit, Screen},
    sprite::Sprite,
};

fn main() -> io::Result<()> {
    let mut screen = Screen::new_pixels(40, 20);
    // draws a HI! message using smileys :)
    for x in [0, 10, 20, 30] {
        for y in [0, 4, 8, 12, 16] {
            draw_smiley(&mut screen, x, y, Blit::Add);
        }
    }
    draw_smiley(&mut screen, 5, 8, Blit::Add);
    draw_smiley(&mut screen, 30, 12, Blit::Subtract);
    screen.enter_screen()?;
    screen.render_screen()?;
    std::thread::sleep(Duration::from_secs(3));
    screen.exit_screen()?;

    Ok(())
}

fn draw_smiley(screen: &mut Screen, x: u16, y: u16, blit: Blit) {
    let smiley =
        Sprite::from_braille_string(&["⢌⣈⠄"], Some(Color::from_rgb_approximate(0, 255, 0)))
            .unwrap();
    screen.draw_sprite(&smiley, x, y, blit);
}
