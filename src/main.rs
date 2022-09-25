#![deny(clippy::all)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use beryllium::{
    event::Event,
    init::{InitFlags, Sdl},
    window::{WindowFlags},
};
use fermium::keycode;
use pixels::{Pixels, SurfaceTexture};
use zstring::zstr;

mod input;
mod camera;

use gridmath::*;
use sandworld::*;
use crate::input::*;
use crate::camera::*;
use std::time::{Instant};

const SCREEN_WIDTH: u32 = 720;
const SCREEN_HEIGHT: u32 = 480;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let sdl = Sdl::init(InitFlags::EVERYTHING)?;
    let window = sdl.create_vk_window(
        zstr!("Sandbox"),
        None,
        (SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32),
        WindowFlags::ALLOW_HIGHDPI | WindowFlags::RESIZABLE,
    )?;

    let mut pixels = {
        // TODO: Beryllium does not expose the SDL2 `GetDrawableSize` APIs, so choosing the correct
        // surface texture size is not possible.
        let surface_texture = SurfaceTexture::new(SCREEN_WIDTH, SCREEN_HEIGHT, &*window);
        Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)?
    };
    let mut world = World::new();
    let mut input = InputState::new();
    let mut camera = Camera::new(SCREEN_WIDTH, SCREEN_HEIGHT, 2);

    let mut debug_draw = false;
    let mut debug_perf = false;

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
                        input.space_pressed = pressed;
                    },
                    keycode::SDLK_1 => {
                        input.num1_pressed = pressed;
                    },
                    keycode::SDLK_LEFT => {
                        input.left_pressed = pressed;
                    },
                    keycode::SDLK_RIGHT => {
                        input.right_pressed = pressed;
                    },
                    keycode::SDLK_DOWN => {
                        input.down_pressed = pressed;
                    },
                    keycode::SDLK_UP => {
                        input.up_pressed = pressed;
                    },
                    keycode::SDLK_F2 => {
                        if pressed { debug_perf = !debug_perf; }
                    },
                    keycode::SDLK_F3 => {
                        if pressed { debug_draw = !debug_draw; }
                    },
                    keycode::SDLK_RIGHTBRACKET => {
                        if pressed { camera.change_scale( camera.scale_factor() + 1); }
                    },
                    keycode::SDLK_LEFTBRACKET => {
                        if pressed { if camera.scale_factor() > 1 {camera.change_scale( camera.scale_factor() - 1); }}
                    },
                    _ => (),
                } 
                Event::MouseButton { button: mouse_button, is_pressed: pressed, ..} => {
                    if mouse_button == 1 {
                        input.left_click_down = pressed;
                    }
                    if mouse_button == 2 {
                        input.middle_click_down = pressed;
                    }
                    else if mouse_button == 3 {
                        input.right_click_down = pressed;
                    }
                }
                Event::MouseMotion { win_x: mouse_x, win_y: mouse_y, ..} => {
                    input.mouse_screen_pos = ScreenPos{x: mouse_x as u32, y: camera.screen_height() - mouse_y as u32 - 1};
                    input.mouse_world_pos = camera.screen_to_world(input.mouse_screen_pos);
                }
                // Resize the window
                Event::WindowResized { width, height, .. } => {
                    pixels.resize_surface(width, height);
                    pixels.resize_buffer(width, height);
                    camera.resize(width, height);
                }

                _ => (),
            }
        }
        
        input.directional_input = GridVec::new(0, 0);
        if input.left_pressed { input.directional_input.x += -1; }
        if input.right_pressed { input.directional_input.x += 1; }
        if input.up_pressed { input.directional_input.y += 1; }
        if input.down_pressed { input.directional_input.y += -1; }

        let frame_start = Instant::now();

        // Process inputs
        update(&mut world, &mut camera, &input);

        // Update world
        let updated_chunks = world.update();

        // Draw the current frame
        draw(&world, &camera, pixels.get_frame(), debug_draw);

        pixels.render()?;
        
        if debug_perf {
            let frame_finished = Instant::now();
            println!("Frame processed in {}μs - Chunk updates: {}", frame_finished.duration_since(frame_start).as_micros(), updated_chunks);
        }
    }

    Ok(())
}

fn update(world: &mut World, cam: &mut Camera, input: &InputState) {
    if input.left_click_down {
        world.place_circle(input.mouse_world_pos, input.brush_radius, Particle::new(ParticleType::Sand), false);
    }
    else if input.middle_click_down {
        world.place_circle(input.mouse_world_pos, input.brush_radius, Particle::new(ParticleType::Water), false);
    }
    else if input.right_click_down {
        world.clear_circle(input.mouse_world_pos, input.brush_radius);
    }
    else if input.space_pressed {
        world.place_circle(input.mouse_world_pos, input.brush_radius, Particle::new(ParticleType::Stone), false);
    }
    else if input.num1_pressed {
        world.place_circle(input.mouse_world_pos, 1, Particle::new(ParticleType::Source), false)
    }

    cam.move_by(input.directional_input);
}

fn draw(world: &World, cam: &Camera, frame: &mut [u8], debug_draw: bool) {
    let visible_part_buffer = world.render(&cam.world_bounds(), debug_draw);

    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = (i % cam.screen_width() as usize) as u32;
        let y = cam.screen_height() - (i / cam.screen_width() as usize) as u32 - 1;

        let buffer_index = (
            x / cam.scale_factor() + 
            y / cam.scale_factor() * cam.world_bounds().width()
        ) as usize;

        let rgba = match visible_part_buffer[buffer_index].particle_type {
                sandworld::ParticleType::Sand => [0xdc, 0xcd, 0x79, 0xff],
                sandworld::ParticleType::Water => [0x56, 0x9c, 0xd6, 0xff],
                sandworld::ParticleType::Stone => [0xd4, 0xd4, 0xd4, 0xff],
                sandworld::ParticleType::Air => [0x1e, 0x1e, 0x1e, 0xff],
                sandworld::ParticleType::Source => [0xf7, 0xdf, 0x00, 0xff],
                sandworld::ParticleType::Dirty => [0xFF, 0x00, 0xFF, 0xff],
                _ => [0x00, 0x00, 0x00, 0xff],
        };

        pixel.copy_from_slice(&rgba);
    }
}