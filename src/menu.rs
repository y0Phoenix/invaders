use crate::{ frame::{Frame, Drawable}};

#[derive(PartialEq)]
pub enum Menu {
    Main = 1,
    Game = 2,
    Help = 3,
    Leaderboard = 4
}

pub struct NewMenu {
    text: String,
    x: usize,
    y: usize
}

impl NewMenu {
    pub fn new(text: String, x: usize, y: usize) -> Self {
        Self { text, x, y }
    }
}

impl Drawable for NewMenu {
    fn draw(&self, frame: &mut Frame) {
        let mut new_line = false;
        let mut y_opp = self.y;
        let mut x_opp = self.x;
        for c in self.text.chars() {
            let y = if new_line {
                y_opp += 1;
                y_opp
            }
            else {
                y_opp
            };
            let x = if new_line {
                x_opp = self.x;
                x_opp
            }
            else {
                x_opp += 1;
                x_opp - 1
            };
            if c == '\n' {
                new_line = true;
            }
            else {
                new_line = false;
                frame[x][y] = c.to_string();
            }
        }
    }
}
