use crate::gridmath::*;

pub struct Camera {
    screen_bounds: GridBounds,
    world_bounds: GridBounds,
    scale_factor: u32,
}

impl Camera {
    pub fn new(width: u32, height: u32, scale: u32) -> Self {
        Camera { 
            screen_bounds: GridBounds::new_from_corner(GridVec::new(0, 0), GridVec::new(width as i32, height as i32)),
            world_bounds: GridBounds::new_from_corner(GridVec::new(0, 0), GridVec::new((width / scale) as i32, (height / scale) as i32)),
            scale_factor: scale,
        } 
    }

    pub fn move_by(&mut self, move_by: GridVec) {
        self.world_bounds.move_by(move_by);
    }

    pub fn _screen_bounds(&self) -> GridBounds {
        self.screen_bounds
    }

    pub fn scale_factor(&self) -> u32 {
        self.scale_factor
    }

    pub fn world_bounds(&self) -> GridBounds {
        self.world_bounds
    }

    pub fn screen_to_world(&self, screen_pos: ScreenPos) -> GridVec {
        let shifted = (GridVec::new(screen_pos.x as i32, screen_pos.y as i32) / self.scale_factor as i32) + self.world_bounds.bottom_left();
        return shifted;
    }

    pub fn change_scale(&mut self, scale: u32) {
        self.scale_factor = scale;
        self.resize(self.screen_bounds.width(), self.screen_bounds.height());

    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.screen_bounds.resize(GridVec::new(width as i32, height as i32));
        // If its not a clean division, add one more to sample the fraction of a scaled pixel from
        let scaled_width = (width / self.scale_factor) as i32 + if width % self.scale_factor != 0 {1} else {0};
        let scaled_height = (height / self.scale_factor) as i32  + if height % self.scale_factor != 0 {1} else {0};
        self.world_bounds.resize(GridVec::new(scaled_width, scaled_height));
    }

    pub fn screen_height(&self) -> u32{
        self.screen_bounds.height()
    }

    pub fn screen_width(&self) -> u32{
        self.screen_bounds.width()
    }
}

