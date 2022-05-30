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

    /*
        Concatenate the bits of the 2 coordinates into a single 64 bit value
        Used for hashing and storage
    */
    pub fn combined(&self) -> u64 {
        self.x as u64 | (self.y as u64) << 32
    }

    /*
        Extract a grid coordinate from the bits of 2 coordinates combined into a 
        single 64 bit value
        Used for hashing and storage
    */
    pub fn decombined(combo: u64) -> GridVec {
        GridVec::new(
            (combo & 0x00000000FFFFFFFF) as i32, 
            ((combo & 0xFFFFFFFF00000000) >> 32) as i32)
    }
}

impl ops::Add<GridVec> for GridVec {
    type Output = GridVec;

    fn add(self, rhs: GridVec) -> GridVec {
        GridVec{x: self.x + rhs.x, y: self.y + rhs.y}
    }
}

impl ops::Sub<GridVec> for GridVec {
    type Output = GridVec;

    fn sub(self, rhs: GridVec) -> GridVec {
        GridVec{x: self.x - rhs.x, y: self.y - rhs.y}
    }
}

#[cfg(test)]
mod tests {
    use crate::gridmath::*;

    #[test]
    fn basic_addition() {
        let a = GridVec::new(1, 0);
        let b = GridVec::new(0, 2);
        let result = a + b;
        let expected = GridVec::new(1, 2);
        assert_eq!(result, expected);
    }

    #[test]
    fn basic_subtraction() {
        let a = GridVec::new(1, 0);
        let b = GridVec::new(0, 2);
        let result = a - b;
        let expected = GridVec::new(1, -2);
        assert_eq!(result, expected);
    }

    #[test]
    fn combination() {
        let result = GridVec::new(4, 10).combined();
        let expected = 0x0000000A00000004;
        assert_eq!(result, expected);
    }

    #[test]
    fn decombination() {
        let result = GridVec::decombined(0x0000000A00000004);
        let expected = GridVec::new(4, 10);
        assert_eq!(result, expected);
    }

    #[test]
    fn combination_decombination() {
        let result = GridVec::decombined(GridVec::new(4, 10).combined());
        let expected = GridVec::new(4, 10);
        assert_eq!(result, expected);
    }
}