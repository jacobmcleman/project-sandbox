use crate::gridmath::*;

pub struct Camera {
    center: GridVec,
    screen_size: GridVec,
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Camera { 
            center: GridVec::new(width as i32 / 2, height as i32 / 2),
            screen_size: GridVec::new(width as i32, height as i32),
        }
    }

    pub fn move_by(&mut self, move_by: GridVec) {
        self.center = self.center + move_by;
    }

    pub fn bounds(&self) -> GridBounds {
        GridBounds::new(self.center, self.screen_size / 2)
    }

    pub fn screen_to_world(&self, screen_pos: ScreenPos) -> GridVec {
        let shifted = GridVec::new(screen_pos.x as i32, screen_pos.y as i32) - (self.screen_size / 2) + self.center;
        return shifted;
    }
}

