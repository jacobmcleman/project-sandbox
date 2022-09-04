use crate::gridmath::*;

pub struct InputState {
    pub mouse_screen_pos: ScreenPos,
    pub mouse_world_pos: GridVec,
    pub directional_input: GridVec,
    pub space_pressed: bool,
    pub num1_pressed: bool,
    pub left_click_down: bool,
    pub middle_click_down: bool,
    pub right_click_down: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub brush_radius: i32,
}

impl InputState {
    pub fn new() -> Self {
        InputState { 
            mouse_screen_pos: ScreenPos{x: 0, y: 0}, 
            mouse_world_pos: GridVec{x: 0, y: 0}, 
            directional_input: GridVec{x: 0, y: 0}, 
            space_pressed: false,
            num1_pressed: false,
            left_click_down: false, 
            middle_click_down: false, 
            right_click_down: false, 
            left_pressed: false, 
            right_pressed: false, 
            up_pressed: false, 
            down_pressed: false, 
            brush_radius: 10 
        }
    }
}