use ti::{
    screen::{Blit, Screen},
    sprite::Sprite,
};

fn main() {
    let mut screen = Screen::new(40, 20);
    // draws a HI! message using smileys :)
    for x in [0, 10, 20, 30] {
        for y in [0, 4, 8, 12, 16] {
            draw_smiley(&mut screen, x, y, Blit::Add);
        }
    }
    draw_smiley(&mut screen, 5, 8, Blit::Add);
    draw_smiley(&mut screen, 30, 12, Blit::Subtract);
    print!("{}", std::str::from_utf8(&screen.rasterize()).unwrap());
}

fn draw_smiley(screen: &mut Screen, x: usize, y: usize, blit: Blit) {
    let smiley = Sprite::from_braille_string(&["⢌⣈⠄"]).unwrap();
    screen.draw_sprite(smiley, x, y, blit);
}
