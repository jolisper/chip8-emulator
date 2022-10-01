mod config;

extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

use chip8_vm::VM;
use crate::config::*;

#[allow(unused)]
pub fn main() -> Result<(), String> {
    let mut chip8: VM = VM::new();

    chip8.screen_draw_sprite(00, 0, 0x00, 5);
    chip8.screen_draw_sprite(05, 0, 0x05, 5);
    chip8.screen_draw_sprite(10, 0, 0x0a, 5);
    chip8.screen_draw_sprite(15, 0, 0x0f, 5);
    chip8.screen_draw_sprite(20, 0, 0x14, 5);
    chip8.screen_draw_sprite(25, 0, 0x19, 5);
    chip8.screen_draw_sprite(30, 0, 0x1e, 5);
    chip8.screen_draw_sprite(35, 0, 0x23, 5);
    chip8.screen_draw_sprite(40, 0, 0x28, 5);
    chip8.screen_draw_sprite(45, 0, 0x2d, 5);
    chip8.screen_draw_sprite(50, 0, 0x32, 5);
    chip8.screen_draw_sprite(55, 0, 0x37, 5);
    chip8.screen_draw_sprite(60, 0, 0x3c, 5);
    chip8.screen_draw_sprite(00, 6, 0x41, 5);
    chip8.screen_draw_sprite(05, 6, 0x46, 5);
    chip8.screen_draw_sprite(10, 6, 0x4b, 5);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            EMULATOR_WINDOW_TITLE,
            CHIP8_WIDTH * CHIP8_WINDOW_MULTIPLIER,
            CHIP8_HEIGHT * CHIP8_WINDOW_MULTIPLIER,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 255, 255));

        for x in 0..CHIP8_WIDTH {
            for y in 0..CHIP8_HEIGHT {
                if chip8.screen_is_pixel_set(x as usize, y as usize).unwrap() {
                    canvas.fill_rect(Rect::new(
                        (x * CHIP8_WINDOW_MULTIPLIER) as i32, 
                        (y * CHIP8_WINDOW_MULTIPLIER) as i32, 
                        CHIP8_WINDOW_MULTIPLIER, 
                    CHIP8_WINDOW_MULTIPLIER, 
                    )).unwrap();
                }
            }
        }
        
        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown { keycode: Some(kc), .. } =>
                {
                    chip8.keyboard_key_down(kc as i32);
                }
                Event::KeyUp { keycode: Some(kc), .. } =>
                {
                    chip8.keyboard_key_up(kc as i32);
                }
                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}
