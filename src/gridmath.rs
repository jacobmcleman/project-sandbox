pub const WORLD_WIDTH: u32 = 720;
pub const WORLD_HEIGHT: u32 = 480;

pub const SCALE_FACTOR: u32 = 2;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct GridVec {
    pub x: i32,
    pub y: i32,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct GridPos {
    pub x: u32,
    pub y: u32,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct ScreenPos {
    pub x: u32,
    pub y: u32,
}

impl GridPos {
    pub fn new(x:u32, y:u32) -> Self {
        GridPos {x, y}
    }

    pub fn moved_by(&self, mut vec: GridVec) -> GridPos {
        let mut result = self.clone();
        if self.x as i32 + vec.x <= 0 { 
            vec.x = 0;
            vec.y = 0;
        }
        else if (self.x as i32 + vec.x) as u32 >= WORLD_WIDTH - 1 { 
            vec.x = 0; 
            vec.y = 0;
        }
        if self.y as i32 + vec.y <= 0 { 
            vec.x = 0; 
            vec.y = 0;
        }
        if (self.y as i32 + vec.y) as u32 >= WORLD_HEIGHT - 1 { 
            vec.x = 0; 
            vec.y = 0;
        }

        result.x = (self.x as i32 + vec.x) as u32;
        result.y = (self.y as i32 + vec.y) as u32;

        return result;
    }
}