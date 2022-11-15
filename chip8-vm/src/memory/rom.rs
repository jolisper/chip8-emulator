use sdl2::{self, sys::SDL_KeyCode};

pub static KEYMAP: &'static [(i32, usize)] = &[
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
