use std::time::Duration;

use crossterm::style::Color;
use ti::{
    screen::{Blit, Screen},
    sprite::Sprite,
};

fn main() {
    let mut screen = Screen::new_pixels(40, 20);
    // draws a HI! message using smileys :)
    for x in [0, 10, 20, 30] {
        for y in [0, 4, 8, 12, 16] {
            draw_smiley(&mut screen, x, y, Blit::Add);
        }
    }
    draw_smiley(&mut screen, 5, 8, Blit::Add);
    draw_smiley(&mut screen, 30, 12, Blit::Subtract);
    // print!("{}", screen.rasterize());
    match screen.render_screen() {
        Ok(()) => (),
        Err(_) => println!("aaaa"),
    }
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn draw_smiley(screen: &mut Screen, x: u16, y: u16, blit: Blit) {
    let smiley = Sprite::from_braille_string(&["⢌⣈⠄"], Some(Color::Green)).unwrap();
    screen.draw_sprite(&smiley, x, y, blit);
}
