use std::{time::Duration, io::{self, Write}};

use crossterm::{QueueableCommand, cursor::MoveTo, event::{self, Event, KeyCode}};

use crate::{NUM_COLS, NUM_ROWS, frame::Drawable, shot::{Shot}, SHOT_COUNT, invaders::Invaders};

pub struct Player {
    x: usize,
    y: usize,
    shots: Vec<Shot>,
    pub name: String,
    pub score: u32,
    pub level: u128
}

impl Player {
    pub fn new() -> Self {
        Self { x: NUM_COLS / 2, y: NUM_ROWS - 1, shots: Vec::new(), name: String::new(), score: 0, level: 1 }
    }
    pub fn move_left(&mut self) {
        if self.x > 0 {
            self.x -= 1;
        }
    }
    pub fn move_right(&mut self) {
        if self.x < NUM_COLS - 1 {
            self.x += 1;
        }
    }
    pub fn shoot(&mut self) -> bool {
        if self.shots.len() < SHOT_COUNT {
            self.shots.push(Shot::new(self.x, self.y - 1));
            return true;
        }
        false 
    }
    pub fn update(&mut self, delta: Duration) {
        for shot in self.shots.iter_mut() {
            shot.update(delta);
        }
        self.shots.retain(|shot| !shot.dead());
    }
    pub fn detect_hits(&mut self, invaders: &mut Invaders) -> bool {
        for shot in self.shots.iter_mut() {
            if invaders.kill_at(shot.x, shot.y) && !shot.exploding {
                shot.explode();
                self.score += 1;
                return true;
            }
        }
        false
    }
    pub fn get_name(&mut self) -> bool {
        let mut stdout = io::stdout();
        stdout.queue(MoveTo(5, 3)).unwrap();
        print!("Please Enter Your Name");
        stdout.flush().unwrap();
        let mut input = String::new();
        let mut characters = 1;
        let bool = loop {
            if event::poll(Duration::default()).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Esc => {
                            break false;
                        },
                        KeyCode::Backspace => {
                            if characters > 1 {
                                input.pop();
                                characters -= 1;
                                stdout.queue(MoveTo(27 + characters, 3)).unwrap();
                                print!(" ");
                                stdout.flush().unwrap();
                            }
                        },
                        KeyCode::Enter => {
                            break true;
                        }
                        _ => {
                            for i in 33..176 as u8 {
                                if key.code == KeyCode::Char(i as char) {
                                    input.push(i as char);
                                    stdout.queue(MoveTo(27 + characters, 3)).unwrap();
                                    characters += 1;
                                    print!("{}", i as char);
                                    stdout.flush().unwrap();
                                }
                            }
                        }
                    }
                }
            }
        };
        input.trim().to_string();
        self.name = input;
        bool
    }

    pub fn clear_shots(&mut self) {
        self.shots.clear();
    }
}

impl Drawable for Player {
    fn draw(&self, frame: &mut crate::frame::Frame) {
        // draw player
        frame[self.x][self.y] = "A".to_string();
        // draw shots
        for shot in self.shots.iter() {
            shot.draw(frame);
        }
    }
}