use crate::gridmath::*;

pub struct InputState {
    pub mouse_screen_pos: ScreenPos,
    pub mouse_world_pos: GridPos,
    pub space_pressed: bool,
    pub left_click_down: bool,
    pub middle_click_down: bool,
    pub right_click_down: bool,
    pub brush_radius: u32,
}

impl InputState {
    pub fn new() -> Self {
        InputState { 
            mouse_screen_pos: ScreenPos{x: 0, y: 0}, 
            mouse_world_pos: GridPos{x: 0, y: 0}, 
            space_pressed: false,
            left_click_down: false, 
            middle_click_down: false, 
            right_click_down: false, 
            brush_radius: 10 
        }
    }
}