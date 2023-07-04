use ti::{
    cell::Cell,
    screen::{Blit, Screen},
    sprite::Sprite,
};

fn main() {
    let mut screen = Screen::new(40, 20);

    // draws a HI! message using smileys :)
    draw_smiley(&mut screen, 0, 0);
    draw_smiley(&mut screen, 0, 1);
    draw_smiley(&mut screen, 0, 2);
    draw_smiley(&mut screen, 0, 3);
    draw_smiley(&mut screen, 0, 4);
    draw_smiley(&mut screen, 3, 2);
    draw_smiley(&mut screen, 6, 0);
    draw_smiley(&mut screen, 6, 1);
    draw_smiley(&mut screen, 6, 2);
    draw_smiley(&mut screen, 6, 3);
    draw_smiley(&mut screen, 6, 4);
    draw_smiley(&mut screen, 11, 0);
    draw_smiley(&mut screen, 11, 1);
    draw_smiley(&mut screen, 11, 2);
    draw_smiley(&mut screen, 11, 3);
    draw_smiley(&mut screen, 11, 4);
    draw_smiley(&mut screen, 16, 0);
    draw_smiley(&mut screen, 16, 1);
    draw_smiley(&mut screen, 16, 2);
    draw_smiley(&mut screen, 16, 4);

    println!("{}", std::str::from_utf8(&screen.rasterize()).unwrap());
}

fn draw_smiley(screen: &mut Screen, x: usize, y: usize) {
    let left = Sprite::new(Cell::from_braille('⢌').unwrap(), None, 0);
    let middle = Sprite::new(Cell::from_braille('⣈').unwrap(), None, 0);
    let right = Sprite::new(Cell::from_braille('⠄').unwrap(), None, 0);
    screen.draw_sprite_aligned(left, x, y, Blit::Set);
    screen.draw_sprite_aligned(middle, x + 1, y, Blit::Set);
    screen.draw_sprite_aligned(right, x + 2, y, Blit::Set);
}
