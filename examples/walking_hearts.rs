use ti::{
    event::Direction,
    screen::{Blit, Screen},
    sprite::Sprite,
};

fn main() {
    let width = 128;
    let height = 64;
    let mut screen = Screen::new_pixels(width, height);

    let sprite =
        Sprite::rgb_from_image_path("examples/heart.png", 2, true, 0).expect("png reading failure");

    let mut x = 5;
    let mut y = 4;
    screen
        .start_loop(60, |s, event| {
            s.clear();
            for y in 3..height - 3 {
                s.draw_pixel_colored(1, y, Blit::Set, None);
                s.draw_pixel_colored(width - 2, y, Blit::Set, None);
            }
            for x in 1..width - 1 {
                s.draw_pixel_colored(x, 3, Blit::Set, None);
                s.draw_pixel_colored(x, height - 4, Blit::Set, None);
            }
            s.draw_sprite(&sprite, x, y, Blit::Set);
            match event.and_then(|e| e.direction_wasd()) {
                // magic numbers based on sprite shape
                Some(Direction::Right) => x = x.saturating_add(1).clamp(2, width - 34),
                Some(Direction::Left) => x = x.saturating_sub(1).clamp(2, width - 34),
                Some(Direction::Down) => y = y.saturating_add(1).clamp(2, height - 34),
                Some(Direction::Up) => y = y.saturating_sub(1).clamp(2, height - 34),
                None => (),
            }
            Ok(())
        })
        .unwrap();
}
