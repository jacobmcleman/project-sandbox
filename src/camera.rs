use crate::gridmath::*;

pub struct Camera {
    offset: GridVec,
    screen_size: GridVec,
    scale_factor: u32,
}

impl Camera {
    pub fn new(width: u32, height: u32, scale_factor: u32) -> Self {
        Camera { 
            offset: GridVec::new(0, 0),
            screen_size: GridVec::new(width as i32, height as i32),
            scale_factor,
        }
    }

    pub fn screen_to_world(&self, screen: ScreenPos) -> GridVec {
        GridVec { x: (screen.x / self.scale_factor) as i32 + self.offset.x, 
            y: (screen.y / self.scale_factor) as i32 + self.offset.y }
    }

    pub fn world_to_screen(&self, world: GridVec) -> ScreenPos {
        ScreenPos { x: ((world.x * self.scale_factor as i32) - self.offset.x) as u32, 
            y: ((world.y * self.scale_factor as i32) - self.offset.y) as u32 }
    }

    pub fn move_by(&mut self, move_by: GridVec) {
        self.offset = self.offset + move_by;
    }
}

