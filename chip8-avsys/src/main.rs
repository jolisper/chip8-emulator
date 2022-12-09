mod config;

extern crate sdl2;

use clap::Parser;
use sdl2::audio::{AudioCallback, AudioSpec, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::sys::SDL_KeyCode;
use sdl2::video::Window;
use sdl2::AudioSubsystem;
use std::io::Read;
use std::time::Duration;

use crate::config::*;
use chip8_vm::{Signal, VM};

static KEYMAP: &'static [(i32, usize)] = &[
    (SDL_KeyCode::SDLK_1 as i32, 0x1),
    (SDL_KeyCode::SDLK_2 as i32, 0x2),
    (SDL_KeyCode::SDLK_3 as i32, 0x3),
    (SDL_KeyCode::SDLK_4 as i32, 0xC),
    (SDL_KeyCode::SDLK_q as i32, 0x4),
    (SDL_KeyCode::SDLK_w as i32, 0x5),
    (SDL_KeyCode::SDLK_e as i32, 0x6),
    (SDL_KeyCode::SDLK_r as i32, 0xD),
    (SDL_KeyCode::SDLK_a as i32, 0x7),
    (SDL_KeyCode::SDLK_s as i32, 0x8),
    (SDL_KeyCode::SDLK_d as i32, 0x9),
    (SDL_KeyCode::SDLK_f as i32, 0xE),
    (SDL_KeyCode::SDLK_z as i32, 0xA),
    (SDL_KeyCode::SDLK_x as i32, 0x0),
    (SDL_KeyCode::SDLK_c as i32, 0xB),
    (SDL_KeyCode::SDLK_v as i32, 0xF),
];

static INVERSE_KEYMAP: &'static [(usize, i32)] = &[
    (0x1, SDL_KeyCode::SDLK_1 as i32),
    (0x2, SDL_KeyCode::SDLK_2 as i32),
    (0x3, SDL_KeyCode::SDLK_3 as i32),
    (0xC, SDL_KeyCode::SDLK_4 as i32),
    (0x4, SDL_KeyCode::SDLK_q as i32),
    (0x5, SDL_KeyCode::SDLK_w as i32),
    (0x6, SDL_KeyCode::SDLK_e as i32),
    (0xD, SDL_KeyCode::SDLK_r as i32),
    (0x7, SDL_KeyCode::SDLK_a as i32),
    (0x8, SDL_KeyCode::SDLK_s as i32),
    (0x9, SDL_KeyCode::SDLK_d as i32),
    (0xE, SDL_KeyCode::SDLK_f as i32),
    (0xA, SDL_KeyCode::SDLK_z as i32),
    (0x0, SDL_KeyCode::SDLK_x as i32),
    (0xB, SDL_KeyCode::SDLK_c as i32),
    (0xF, SDL_KeyCode::SDLK_v as i32),
];

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
    let mut calls_count = 0;

    'running: loop {
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
                    chip8.keyboard_key_down(kc as i32, KEYMAP);
                }
                Event::KeyUp {
                    keycode: Some(kc), ..
                } => {
                    chip8.keyboard_key_up(kc as i32, KEYMAP);
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

        match chip8.exec_next_opcode(debug_mode, &mut calls_count)? {
            Signal::DrawScreen => {
                draw_screen(&mut chip8, &mut canvas)?;
            }
            Signal::WaitKeyUp(key) => 'wait_keyup: loop {
                for event in event_pump.poll_iter() {
                    match event {
                        Event::KeyUp {
                            keycode: Some(kc), ..
                        } => {
                            if let Some(sdl_key) = ch8_key2sdl_key(key as usize) {
                                if sdl_key == kc as i32 {
                                    chip8.keyboard_key_up(sdl_key, KEYMAP);
                                    break 'wait_keyup;
                                }
                            }
                        }
                        Event::Quit { .. }
                        | Event::KeyDown {
                            keycode: Some(Keycode::Escape),
                            ..
                        } => break 'running,
                        _ => {}
                    }
                }
            },
            _ => {}
        }
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

fn draw_screen(chip8: &mut VM, canvas: &mut Canvas<Window>) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(153, 102, 0));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(255, 204, 0));
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
    canvas.present();
    Ok(())
}

fn ch8_key2sdl_key(key: usize) -> Option<i32> {
    for (ch8_k, sdl_k) in INVERSE_KEYMAP.into_iter() {
        if key == *ch8_k {
            return Some(*sdl_k);
        }
    }
    None
}
