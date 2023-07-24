use ti::{
    cell::{Cell, OffsetCell},
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

    let block = Cell::new(0xff);
    let offset = block.compute_offset(1, 3);
    screen.draw_cell(block, 0, 0, Blit::Set);
    screen.draw_cell(block, 0, 1, Blit::Set);
    screen.draw_cell(block, 1, 0, Blit::Set);
    screen.draw_cell_unaligned(offset, 1, 1, Blit::Set);

    println!("{}", std::str::from_utf8(&screen.rasterize()).unwrap());
}

fn draw_smiley(screen: &mut Screen, x: usize, y: usize) {
    let smiley = Sprite::from_braille_string(&["⢌⣈⠄"]);
}
