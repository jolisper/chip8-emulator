mod config;

extern crate sdl2;

use sdl2::AudioSubsystem;
use sdl2::audio::{AudioSpecDesired, AudioCallback, AudioSpec};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

use chip8_vm::VM;
use crate::config::*;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

#[allow(unused)]
pub fn main() -> Result<(), String> {
    let mut chip8: VM = VM::new();

    chip8.load_program(b"Hello world!");
    chip8.memory_dump();

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

    // Wrapped print of "A" sprint
    chip8.screen_draw_sprite(62, 12, 0x32, 5);

    // Print same sprite in the same position twice, check collision:
    chip8.screen_draw_sprite(32, 16, 0x32, 5);
    if !chip8.screen_draw_sprite(32, 16, 0x32, 5).unwrap() {
        panic!("Collision must be detected")
    }

    // Check on/off pixel behavior when is set again
    if chip8.screen_is_pixel_set(60, 16).unwrap() {
        panic!("The pixel must be unset")
    }

    // Set delay timer
    chip8.registers_set_dt(10);

    // Set sound timer
    chip8.registers_set_st(20);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let audio_subsystem = sdl_context.audio()?;
    let device = build_audio_device(&audio_subsystem); 

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

        if chip8.registers_dt() > 0 {
            std::thread::sleep(Duration::from_millis(100));
            chip8.registers_dec_dt();
        }

        if chip8.registers_st() > 0 {
            // Start playback
            device.resume();
            chip8.registers_dec_st();
        } else {
            device.pause();
        } 

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}

fn build_audio_device(audio_subsystem: &AudioSubsystem) -> sdl2::audio::AudioDevice<SquareWave> {
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),  // mono
        samples: None       // default sample size
    };
    let audio_spec = |spec: AudioSpec| {
        SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25
        }
    };
    audio_subsystem.open_playback(None, &desired_spec,  audio_spec).unwrap()
}
