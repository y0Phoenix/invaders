use std::time::Duration;

use rusty_time::Timer;

use crate::{frame::Drawable, Direction, INVADER_MULTIPLIER, NUM_COLS, NUM_ROWS};

pub struct Invader {
    pub x: usize,
    pub y: usize,
}

impl Invader {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

pub struct Invaders {
    pub army: Vec<Invader>,
    move_timer: Timer,
    direction: Direction,
    pub speed: f64,
}

impl Invaders {
    pub fn new(speed: u128) -> Self {
        let mut army = Vec::new();
        for x in 0..NUM_COLS {
            for y in 0..NUM_ROWS {
                if (x % 2 == 0)
                    && (y % 2 == 0)
                    && (x > 1)
                    && (x < NUM_COLS - 2)
                    && (y > 0)
                    && (y < INVADER_MULTIPLIER as usize)
                {
                    army.push(Invader::new(x, y));
                }
            }
        }
        let multiplier = speed as f64 * 1.05;
        let move_timer = if speed == 1 {
            Timer::new(Duration::from_millis(2000))
        } else {
            Timer::new(Duration::from_millis((2000.0 / multiplier) as u64))
        };
        Self {
            army,
            move_timer,
            direction: Direction::Left,
            speed: multiplier,
        }
    }
    pub fn update(&mut self, delta: Duration) -> bool {
        self.move_timer.tick(delta);
        if !self.move_timer.finished() {
            return false;
        }
        self.move_timer.reset();
        let mut downwards = false;
        if self.direction == Direction::Left {
            let min_x = self.army.iter().map(|invader| invader.x).min().unwrap_or(0);
            if min_x == 0 {
                self.direction = Direction::Right;
                downwards = true;
            }
        }
        if self.direction == Direction::Right {
            let max_x = self.army.iter().map(|invader| invader.x).max().unwrap_or(0);
            if max_x == NUM_COLS - 1 {
                self.direction = Direction::Left;
                downwards = true;
            }
        }
        if downwards {
            for invader in self.army.iter_mut() {
                invader.y = match invader.y + 1 {
                    y if y >= NUM_ROWS => y - 1,
                    y => y,
                };
            }
        } else {
            for invader in self.army.iter_mut() {
                invader.x = match ((invader.x as i32) + self.direction.clone() as i32) as usize {
                    x if x >= NUM_COLS => (x as i32 - self.direction.clone() as i32) as usize,
                    x => x,
                }
            }
        }
        true
    }
    /**
     * desc: win condition
     */
    pub fn all_dead(&self) -> bool {
        self.army.is_empty()
    }
    /**
     * desc: lose condition
     */
    pub fn reached_bottom(&self) -> bool {
        self.army.iter().map(|invader| invader.y).max().unwrap_or(0) >= NUM_ROWS - 1
    }
    pub fn kill_at(&mut self, x: usize, y: usize) -> bool {
        if let Some(i) = self.army.iter().position(|inv| inv.x == x && inv.y == y) {
            self.army.remove(i);
            return true;
        }
        false
    }
}

impl Drawable for Invaders {
    fn draw(&self, frame: &mut crate::frame::Frame) {
        for invader in self.army.iter() {
            frame[invader.x][invader.y] = if self.move_timer.percent_left() > 0.5 {
                "x"
            } else {
                "+"
            }
        }
    }
}
