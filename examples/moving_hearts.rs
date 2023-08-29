use std::time::Duration;

use ti::{
    screen::{Blit, Screen},
    sprite::Sprite,
};

fn main() {
    let max = 16;
    let mut screen = Screen::new_pixels(16 + max * 2, 16 + max * 2);
    screen.enter_screen().unwrap();

    let sprite =
        Sprite::rgb_from_image_path("examples/heart.png", 1, true, 0).expect("png reading failure");

    for position in 0..=max {
        screen.clear();
        screen.draw_sprite(&sprite, 0, 0, Blit::Toggle);
        screen.draw_sprite(&sprite, position, position, Blit::Toggle);
        screen.draw_sprite(&sprite, position * 2, position * 2, Blit::Toggle);
        screen.render_screen().unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_secs(5));

    screen.exit_screen().unwrap();
}
