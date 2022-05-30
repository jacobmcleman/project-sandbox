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

mod sandworld;
mod input;
mod gridmath;

use crate::gridmath::*;
use crate::sandworld::*;
use crate::input::*;

const SCREEN_WIDTH: u32 = WORLD_WIDTH * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = WORLD_HEIGHT * SCALE_FACTOR;

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
    let mut input = InputState::new();

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
                    input.mouse_screen_pos = ScreenPos{x: mouse_x as u32, y: SCREEN_HEIGHT - mouse_y as u32 - 1};
                    input.mouse_world_pos = World::screen_to_world(input.mouse_screen_pos);
                }
                // Resize the window
                Event::WindowResized { width, height, .. } => pixels.resize_surface(width, height),

                _ => (),
            }
        }

        // Update internal state
        update(&mut world, &input);
        world.update();

        // Draw the current frame
        draw(&world, pixels.get_frame());
        pixels.render()?;
    }

    Ok(())
}

fn update(world: &mut World, input: &InputState) {
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
}

fn draw(world: &World, frame: &mut [u8]) {
    for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
        let x = (i % SCREEN_WIDTH as usize) as u32;
        let y = SCREEN_HEIGHT - (i / SCREEN_WIDTH as usize) as u32 - 1;

        let screen_pos = ScreenPos{x, y};
        let world_pos = World::screen_to_world(screen_pos);

        let rgba = match world.get_particle(world_pos).particle_type {
            sandworld::ParticleType::Sand => [0xdc, 0xcd, 0x79, 0xff],
            sandworld::ParticleType::Water => [0x56, 0x9c, 0xd6, 0xff],
            sandworld::ParticleType::Stone => [0xd4, 0xd4, 0xd4, 0xff],
            _ => [0x1e, 0x1e, 0x1e, 0xff],
        };

        pixel.copy_from_slice(&rgba);
    }
}