#![deny(clippy::all)]
#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use beryllium::{
    event::Event,
    init::{InitFlags, Sdl},
    window::WindowFlags,
};
use fermium::keycode;
use pixels::{Pixels, SurfaceTexture};
use zstring::zstr;
use rand::Rng;

const WORLD_WIDTH: u32 = 720;
const WORLD_HEIGHT: u32 = 480;

const SCALE_FACTOR: u32 = 2;

const SCREEN_WIDTH: u32 = WORLD_WIDTH * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = WORLD_HEIGHT * SCALE_FACTOR;

struct InputState {
    mouse_screen_pos: ScreenPos,
    mouse_world_pos: GridPos,
    space_pressed: bool,
    left_click_down: bool,
    middle_click_down: bool,
    right_click_down: bool,
    brush_radius: u32,
}

/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    particles: [Particle; WORLD_WIDTH as usize * WORLD_HEIGHT as usize],
    input: InputState,
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct GridVec {
    x: i32,
    y: i32,
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct GridPos {
    x: u32,
    y: u32,
}

#[derive(PartialEq, Debug, Copy, Clone)]
struct ScreenPos {
    x: u32,
    y: u32,
}

impl GridPos {
    fn moved_by(&self, mut vec: GridVec) -> GridPos {
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

#[derive(PartialEq, Debug, Copy, Clone)]
enum ParticleType {
    Boundary,
    Air,
    Sand,
    Water,
    Stone,
}

#[derive(Debug, Copy, Clone)]
struct Particle {
    particle_type: ParticleType
}

impl Default for Particle {
    fn default() -> Self { Particle{particle_type: ParticleType::Air} }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let sdl = Sdl::init(InitFlags::EVERYTHING)?;
    let window = sdl.create_vk_window(
        zstr!("Hello Pixels"),
        None,
        (SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32),
        WindowFlags::ALLOW_HIGHDPI,
    )?;

    let mut pixels = {
        // TODO: Beryllium does not expose the SDL2 `GetDrawableSize` APIs, so choosing the correct
        // surface texture size is not possible.
        let surface_texture = SurfaceTexture::new(SCREEN_WIDTH, SCREEN_HEIGHT, &*window);
        Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)?
    };
    let mut world = World::new();

    'game_loop: loop {
        while let Some(event) = sdl.poll_event() {
            match event {
                // Close events
                Event::Quit { .. } => break 'game_loop,
                Event::Keyboard { keycode: key, is_pressed: pressed, .. } => match key {
                    keycode::SDLK_ESCAPE => {
                        break 'game_loop
                    },
                    keycode::SDLK_SPACE => {
                        world.input.space_pressed = pressed;
                    },
                    _ => (),
                } 
                Event::MouseButton { button: mouse_button, is_pressed: pressed, ..} => {
                    if mouse_button == 1 {
                        world.input.left_click_down = pressed;
                    }
                    if mouse_button == 2 {
                        world.input.middle_click_down = pressed;
                    }
                    else if mouse_button == 3 {
                        world.input.right_click_down = pressed;
                    }
                }
                Event::MouseMotion { win_x: mouse_x, win_y: mouse_y, ..} => {
                    world.input.mouse_screen_pos = ScreenPos{x: mouse_x as u32, y: SCREEN_HEIGHT - mouse_y as u32 - 1};
                    world.input.mouse_world_pos = World::screen_to_world(world.input.mouse_screen_pos);
                }
                // Resize the window
                Event::WindowResized { width, height, .. } => pixels.resize_surface(width, height),

                _ => (),
            }
        }

        // Update internal state
        world.update();

        // Draw the current frame
        world.draw(pixels.get_frame());
        pixels.render()?;
    }

    Ok(())
}

impl World {
    fn new() -> Self {
        let created: World = World {
            particles: [Particle::default(); WORLD_WIDTH as usize * WORLD_HEIGHT as usize],
            input: InputState { 
                mouse_screen_pos: ScreenPos{x: 0, y: 0}, 
                mouse_world_pos: GridPos{x: 0, y: 0}, 
                space_pressed: false,
                left_click_down: false, 
                middle_click_down: false, 
                right_click_down: false, 
                brush_radius: 10 },
        };

        return created;
    }

    fn screen_to_world(screen: ScreenPos) -> GridPos {
        GridPos { x: screen.x / SCALE_FACTOR, y: screen.y / SCALE_FACTOR }
    }

    fn world_to_screen(world: GridPos) -> ScreenPos {
        ScreenPos { x: world.x * SCALE_FACTOR, y: world.y * SCALE_FACTOR }
    }

    fn get_index(pos: GridPos) -> usize {
        return pos.y as usize * WORLD_WIDTH as usize + pos.x as usize;
    }

    fn get_particle(&self, pos: GridPos) -> Particle {
        if pos.x <= 0 || pos.x >= WORLD_WIDTH - 2 || pos.y <= 0 || pos.y >= WORLD_HEIGHT - 2 {
            return Particle { particle_type: ParticleType::Boundary };
        }
        return self.particles[World::get_index(pos)];
    }

    fn replace_particle(&mut self, pos: GridPos, new_val: Particle) {
        if pos.x <= 0 || pos.x >= WORLD_WIDTH - 2 || pos.y <= 0 || pos.y >= WORLD_HEIGHT - 2 {
            return;
        }
        self.particles[World::get_index(pos)] = new_val;
    }

    fn add_particle(&mut self, pos: GridPos, new_val: Particle) {
        if pos.x <= 0 || pos.x >= WORLD_WIDTH - 2 || pos.y <= 0 || pos.y >= WORLD_HEIGHT - 2 {
            return;
        }
        if self.get_particle(pos).particle_type == ParticleType::Air {
            self.particles[World::get_index(pos)] = new_val;
        }
    }

    fn clear_circle(&mut self, pos: GridPos, radius: u32) {
        self.place_circle(pos, radius, Particle{particle_type:ParticleType::Air}, true);
    }

    fn place_circle(&mut self, pos: GridPos, radius: u32, new_val: Particle, replace: bool) {
        let left = if pos.x <= radius { 0 } else { pos.x - radius };
        let right = if pos.x >= WORLD_WIDTH - radius { WORLD_WIDTH - 1 } else { pos.x + radius };
        let bottom = if pos.y <= radius { 0 } else { pos.y - radius };
        let top = if pos.y >= WORLD_HEIGHT - radius { WORLD_HEIGHT - 1 } else { pos.y + radius };

        for y in bottom..top {
            for x in left..right {
                if replace { self.replace_particle(GridPos{x, y}, new_val.clone()); }
                else { self.add_particle(GridPos{x, y}, new_val.clone()); }
            }
        }
    }

    fn test_vec(&self, base_pos: GridPos, test_vec: GridVec, replace_water: bool) -> bool {
        let test_pos = base_pos.moved_by(test_vec);
        let material_at_test = self.get_particle(test_pos).particle_type;
        if material_at_test == ParticleType::Air { return true; }
        else if replace_water && material_at_test == ParticleType::Water { return true; }
        return false;
    }

    fn update(&mut self) {
        let mut rng = rand::thread_rng();
        for y in 0..WORLD_HEIGHT {
            // flip processing order for a random half of rows
            let flip = rng.gen_bool(0.5);
            for mut x in 0..WORLD_WIDTH {
                if flip { x = WORLD_WIDTH - x - 1; }

                let base_pos = GridPos{x, y};
                let cur_part = self.get_particle(base_pos);
                if cur_part.particle_type == ParticleType::Sand {
                    if y >= 1 {
                        let available_moves = vec![GridVec{x: 1, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 0, y: -1}];
                        let mut possible_moves = Vec::<GridVec>::new();
                        
                        for vec in available_moves {
                            if self.test_vec(base_pos, vec, true) {
                                possible_moves.push(vec.clone());
                            }
                        }

                        if possible_moves.len() > 0 {
                            let chosen_vec = possible_moves[rng.gen_range(0..possible_moves.len())];
                            let chosen_pos = base_pos.moved_by(chosen_vec);
                            self.replace_particle(base_pos, self.get_particle(chosen_pos));
                            self.replace_particle(chosen_pos, cur_part);
                        }
                    }
                }
                else if cur_part.particle_type == ParticleType::Water {
                    let available_moves = vec![GridVec{x: 1, y: -1}, GridVec{x: 0, y: -1}, GridVec{x: -1, y: -1}, GridVec{x: 0, y: -2} ];
                    let mut possible_moves = Vec::<GridVec>::new();
                        
                    for vec in available_moves {
                        if self.test_vec(base_pos, vec, false) {
                            possible_moves.push(vec.clone());
                        }
                    }

                    if possible_moves.len() <= 1 {
                        let available_moves_2 = vec![ GridVec{x: 1, y: 0}, GridVec{x: -1, y: 0}, GridVec{x: 2, y: 0}, GridVec{x: -2, y: 0}, GridVec{x: 3, y: 0}, GridVec{x: -3, y: 0} ];
                        for vec in available_moves_2 {
                            if self.test_vec(base_pos, vec, false) {
                                possible_moves.push(vec.clone());
                            }
                        }
                    }

                    if possible_moves.len() > 0 {
                        let chosen_vec = possible_moves[rng.gen_range(0..possible_moves.len())];
                        let chosen_pos = base_pos.moved_by(chosen_vec);
                        self.replace_particle(base_pos, self.get_particle(chosen_pos));
                        self.replace_particle(chosen_pos, cur_part);
                    }
                }
            }
        }

        if self.input.left_click_down {
            self.place_circle(self.input.mouse_world_pos, self.input.brush_radius, Particle{particle_type:ParticleType::Sand}, false);
        }
        else if self.input.middle_click_down {
            self.place_circle(self.input.mouse_world_pos, self.input.brush_radius, Particle{particle_type:ParticleType::Water}, false);
        }
        else if self.input.right_click_down {
            self.clear_circle(self.input.mouse_world_pos, self.input.brush_radius);
        }
        else if self.input.space_pressed {
            self.place_circle(self.input.mouse_world_pos, self.input.brush_radius, Particle{particle_type:ParticleType::Stone}, false);
        }

    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % SCREEN_WIDTH as usize) as u32;
            let y = SCREEN_HEIGHT - (i / SCREEN_WIDTH as usize) as u32 - 1;

            let screen_pos = ScreenPos{x, y};
            let world_pos = World::screen_to_world(screen_pos);

            let rgba = match self.get_particle(world_pos).particle_type {
                ParticleType::Sand => [0xdc, 0xcd, 0x79, 0xff],
                ParticleType::Water => [0x56, 0x9c, 0xd6, 0xff],
                ParticleType::Stone => [0xd4, 0xd4, 0xd4, 0xff],
                _ => [0x1e, 0x1e, 0x1e, 0xff],
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}