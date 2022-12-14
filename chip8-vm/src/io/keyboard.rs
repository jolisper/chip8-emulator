use crate::{config::CHIP8_TOTAL_KEYS, errors::VMError};

pub struct Keyboard {
    keyboard: [bool; CHIP8_TOTAL_KEYS],
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            keyboard: [false; CHIP8_TOTAL_KEYS],
        }
    }
}

impl Keyboard {
    pub(crate) fn key_down(&mut self, key: i32, keymap: &[(i32, usize)]) {
        self.map_to_vkey(key, keymap)
            .map(|key| self.keyboard[key as usize] = true)
            .ok();
    }

    pub(crate) fn key_up(&mut self, key: i32, keymap: &[(i32, usize)]) {
        self.map_to_vkey(key, keymap)
            .map(|key| self.keyboard[key as usize] = false)
            .ok();
    }

    pub(crate) fn is_key_down(&self, vkey: u8) -> bool {
        self.keyboard[vkey as usize]
    }

    pub(crate) fn is_key_up(&self, vkey: u8) -> bool {
        !self.keyboard[vkey as usize]
    }

    pub(crate) fn map_to_vkey(&self, key: i32, keymap: &[(i32, usize)]) -> Result<usize, VMError> {
        for (sdl_k, ch8_k) in keymap.into_iter() {
            if key == *sdl_k {
                return Ok(*ch8_k);
            }
        }
        Err(VMError::KeyMapNotFound)
    }
}
