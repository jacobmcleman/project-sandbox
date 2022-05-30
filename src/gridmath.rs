use std::ops;

pub const WORLD_WIDTH: i32 = 720;
pub const WORLD_HEIGHT: i32 = 480;

pub const SCALE_FACTOR: i32 = 2;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct GridVec {
    pub x: i32,
    pub y: i32,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct ScreenPos {
    pub x: u32,
    pub y: u32,
}

impl GridVec {
    pub fn new(x:i32, y:i32) -> Self {
        GridVec {x, y}
    }
}

impl ops::Add<GridVec> for GridVec {
    type Output = GridVec;

    fn add(self, rhs: GridVec) -> GridVec {
        GridVec{x: self.x + rhs.x, y: self.y + rhs.y}
    }
}