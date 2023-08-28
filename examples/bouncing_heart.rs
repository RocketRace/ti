use ti::{
    screen::{Blit, Screen},
    sprite::Sprite,
};

fn main() {
    let width = 64;
    let height = 35;
    let mut screen = Screen::new_pixels(width, height);

    let use_alpha_channel = true;
    let sprite = Sprite::rgb_from_image_path("examples/heart.png", use_alpha_channel)
        .expect("png reading failure");

    let mut x = 0;
    let mut right = true;
    let mut y = 0;
    let mut down = true;
    screen
        .start_loop(60, |s| {
            s.clear();
            s.draw_sprite(&sprite, x, y, Blit::Set);
            if x == 0 {
                right = true;
            }
            if x == width - 15 {
                right = false;
            }
            if y == 0 {
                down = true;
            }
            if y == height - 15 {
                down = false;
            }
            if right {
                x += 1;
            } else {
                x -= 1;
            }
            if down {
                y += 1;
            } else {
                y -= 1;
            }
            Ok(())
        })
        .unwrap();
}
