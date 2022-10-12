use std::ops;
use std::fmt;

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

    pub fn manhattan_distance(&self, other: GridVec) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    pub fn manhattan_length(&self) -> i32 {
        self.x.abs() + self.y.abs()
    }

    pub fn is_adjacent(&self, other: GridVec) -> bool {
        match self.manhattan_distance(other) {
            1 => true,
            2 => (self.x - other.x).abs() == 1,
            _ => false
        }
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

impl ops::Mul<i32> for GridVec {
    type Output = GridVec;

    fn mul(self, rhs: i32) -> GridVec {
        GridVec{x: self.x * rhs, y: self.y * rhs}
    }
}

impl ops::Div<i32> for GridVec {
    type Output = GridVec;

    fn div(self, rhs: i32) -> GridVec {
        GridVec{x: self.x / rhs, y: self.y / rhs}
    }
}

impl ops::Rem<i32> for GridVec {
    type Output = GridVec;

    fn rem(self, rhs: i32) -> Self::Output {
        GridVec{x: self.x % rhs, y: self.y / rhs }
    }
}

impl fmt::Display for GridVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

#[cfg(test)]
mod tests {
    use crate::gridvec::*;

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
    fn basic_multiplication() {
        let a = GridVec::new(1, 2);
        let result = a * 2;
        let expected = GridVec::new(2, 4);
        assert_eq!(result, expected);
    }

    #[test]
    fn basic_division() {
        let a = GridVec::new(1, 2);
        let result = a/ 2;
        let expected = GridVec::new(0, 1);
        assert_eq!(result, expected);
    }

    #[test]
    fn manhattan_distance() {
        let a = GridVec::new(1, 1);
        let b = GridVec::new(-1, 0);
        let result = a.manhattan_distance(b);
        let expected = 3;
        assert_eq!(result, expected);
    }

    #[test]
    fn manhattan_distance_zero() {
        let a = GridVec::new(1, 1);
        let b = GridVec::new(1, 1);
        let result = a.manhattan_distance(b);
        let expected = 0;
        assert_eq!(result, expected);
    }

    #[test]
    fn adjacency_orthogonal() {
        let a = GridVec::new(0, 0);
        let b = GridVec::new(0, -1);
        let result = a.is_adjacent(b);
        let expected = true;
        assert_eq!(result, expected);
    }

    #[test]
    fn adjacency_diagonal() {
        let a = GridVec::new(0, 0);
        let b = GridVec::new(1, 1);
        let result = a.is_adjacent(b);
        let expected = true;
        assert_eq!(result, expected);
    }

    #[test]
    fn adjacency_miss() {
        let a = GridVec::new(0, 0);
        let b = GridVec::new(0, 2);
        let result = a.is_adjacent(b);
        let expected = false;
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