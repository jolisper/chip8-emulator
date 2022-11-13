mod config;

extern crate sdl2;

use clap::Parser;
use sdl2::audio::{AudioCallback, AudioSpec, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::AudioSubsystem;
use std::io::Read;
use std::time::Duration;

use crate::config::*;
use chip8_vm::VM;

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
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

#[derive(Parser)]
struct Args {
    rom_file: String,
    debug: bool,
}

pub fn main() -> Result<(), String> {
    let args = Args::parse();
    let rom_file_name = args.rom_file;
    let debug_mode = args.debug;

    // Load ROM file
    let mut file = std::fs::File::open(rom_file_name).unwrap();
    let mut buf = Vec::new();
    file.read_to_end(&mut buf).expect("read all ROM file");

    let mut chip8: VM = VM::new();
    chip8.load_program(&buf)?;

    let sdl_context = sdl2::init().unwrap();
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
        canvas.set_draw_color(Color::RGB(153, 102, 0));
        canvas.clear();
        canvas.set_draw_color(Color::RGB(255, 204, 0));

        draw_pixels(&mut chip8, &mut canvas)?;

        canvas.present();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(kc), ..
                } => {
                    chip8.keyboard_key_down(kc as i32);
                }
                Event::KeyUp {
                    keycode: Some(kc), ..
                } => {
                    chip8.keyboard_key_up(kc as i32);
                }
                _ => {}
            }
        }

        if chip8.registers_dt() > 0 {
            std::thread::sleep(Duration::from_millis(10));
            chip8.registers_dec_dt();
        }

        if chip8.registers_st() > 0 {
            device.resume(); // Start playback
            chip8.registers_dec_st();
        } else {
            device.pause();
        }

        chip8.exec_next_opcode(debug_mode)?;

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 1000));
    }

    Ok(())
}

fn build_audio_device(audio_subsystem: &AudioSubsystem) -> sdl2::audio::AudioDevice<SquareWave> {
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };
    let audio_spec = |spec: AudioSpec| SquareWave {
        phase_inc: 440.0 / spec.freq as f32,
        phase: 0.0,
        volume: 0.25,
    };
    audio_subsystem
        .open_playback(None, &desired_spec, audio_spec)
        .unwrap()
}

fn draw_pixels(chip8: &mut VM, canvas: &mut Canvas<Window>) -> Result<(), String> {
    for x in 0..CHIP8_WIDTH {
        for y in 0..CHIP8_HEIGHT {
            if chip8.screen_is_pixel_set(x as usize, y as usize)? {
                canvas.fill_rect(Rect::new(
                    (x * CHIP8_WINDOW_MULTIPLIER) as i32,
                    (y * CHIP8_WINDOW_MULTIPLIER) as i32,
                    CHIP8_WINDOW_MULTIPLIER,
                    CHIP8_WINDOW_MULTIPLIER,
                ))?;
            }
        }
    }
    Ok(())
}
