use ti::screen::Screen;

fn main() {
    let mut screen = Screen::new(8, 8);
    for i in 0..8 {
        screen.set_pixel_at(i, 0, true);
        screen.set_pixel_at(i, 7, true);
        screen.set_pixel_at(0, i, true);
        screen.set_pixel_at(7, i, true);
    }
    println!("{}", std::str::from_utf8(&screen.rasterize()).unwrap());
}
