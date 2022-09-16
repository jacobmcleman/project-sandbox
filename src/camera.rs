use crate::gridmath::*;

pub struct Camera {
    bounds: GridBounds
}

impl Camera {
    pub fn new(width: u32, height: u32) -> Self {
        Camera { 
            bounds: GridBounds::new_from_corner(GridVec::new(0, 0), GridVec::new(width as i32, height as i32)),
        }
    }

    pub fn move_by(&mut self, move_by: GridVec) {
        self.bounds.move_by(move_by);
    }

    pub fn bounds(&self) -> GridBounds {
        self.bounds
    }

    pub fn screen_to_world(&self, screen_pos: ScreenPos) -> GridVec {
        let shifted = GridVec::new(screen_pos.x as i32, screen_pos.y as i32) + self.bounds.bottom_left();
        return shifted;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.bounds.resize(GridVec::new(width as i32, height as i32));
    }

    pub fn screen_height(&self) -> u32{
        self.bounds.height()
    }

    pub fn screen_width(&self) -> u32{
        self.bounds.width()
    }
}

