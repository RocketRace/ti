use ti::{
    screen::{Blit, Screen},
    sprite::Sprite,
};

fn main() {
    let scale = 4;
    let sprite = Sprite::from_rgb_image_path("examples/sprite.png", 24 * scale, 24 * scale)
        .expect("png reading failure");
    let mut screen = Screen::new_cells(12 * scale, 6 * scale);
    screen.draw_sprite(&sprite, 0, 0, Blit::Set);
    screen.render_screen().expect("rendering failure");
}
