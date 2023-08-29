//! Quick and dirty example of a simple tick loop
use ti::{
    color::standard,
    screen::{Blit, Screen},
    sprite::Sprite,
};

#[derive(Clone)]
struct Heart {
    pub sprite: Sprite,
    pub max_x: u16,
    pub max_y: u16,
    pub x: u16,
    pub right: bool,
    pub y: u16,
    pub down: bool,
    pub slowness: u64,
}

impl Heart {
    fn tick(&mut self, ticks: u64) {
        if ticks % self.slowness == 0 {
            if self.x == 0 {
                self.right = true;
            }
            if self.x == self.max_x - 1 {
                self.right = false;
            }
            if self.y == 0 {
                self.down = true;
            }
            if self.y == self.max_y - 1 {
                self.down = false;
            }
            if self.right {
                self.x += 1;
            } else {
                self.x -= 1;
            }
            if self.down {
                self.y += 1;
            } else {
                self.y -= 1;
            }
        }
    }
}

fn main() {
    let width = 64;
    let height = 35;
    let mut screen = Screen::new_pixels(width, height);

    let sprite =
        Sprite::rgb_from_image_path("examples/heart.png", 1, true, 2).expect("png reading failure");

    let heart = Heart {
        sprite,
        max_x: width - 16,
        max_y: height - 16,
        x: 0,
        right: true,
        y: 0,
        down: true,
        slowness: 3,
    };

    let mut hearts = vec![heart; 3];
    hearts[1].slowness = 2;
    hearts[1].x = 25;
    hearts[1].right = false;
    hearts[1].y = 12;
    hearts[1].sprite = hearts[1].sprite.recolor(|_| Some(standard::GREEN));
    hearts[1].sprite.priority = 1;
    hearts[2].slowness = 1;
    hearts[2].x = 40;
    hearts[2].y = 3;
    hearts[2].down = false;
    hearts[2].sprite = hearts[1].sprite.recolor(|_| Some(standard::BRIGHT_YELLOW));
    hearts[2].sprite.priority = 0;

    let mut ticks = 0;
    screen
        .start_loop(60, |s, _| {
            s.clear();
            for heart in &mut hearts {
                heart.tick(ticks);
                s.draw_sprite(&heart.sprite, heart.x, heart.y, Blit::Toggle);
            }
            ticks += 1;
            Ok(())
        })
        .unwrap();
}
