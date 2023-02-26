pub mod frame;
pub mod render;
pub mod player;
pub mod shot;
pub mod invaders;
pub mod system;
pub mod menu;
pub mod request;

pub const NUM_ROWS: usize = 20;
pub const NUM_COLS: usize = 40;
pub const INVADER_MULTIPLIER: u32 = 9;
pub const SHOT_COUNT: usize = 6;

#[derive(PartialEq, Clone)]
pub enum Direction {
    Left = -1,
    Right = 1
}