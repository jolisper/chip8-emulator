use crate::{memory::ROM::KEYMAP, errors::VMError, config::CHIP8_TOTAL_KEYS};

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

    pub(crate) fn key_down(&mut self, key: i32) {
        self.map_to_vkey(key)
            .map(|key| self.keyboard[key as usize] = true)
            .ok();
    }

    pub(crate) fn key_up(&mut self, key: i32) {
        self.map_to_vkey(key)
            .map(|key| self.keyboard[key as usize] = false)
            .ok();
    }

    pub(crate) fn is_key_down(&self, vkey: u32) -> bool {
        self.keyboard[vkey as usize]
    }

    pub(crate) fn map_to_vkey(&self, key: i32) -> Result<usize, VMError> {
        for (i, k) in KEYMAP.into_iter().enumerate() {
            if key == *k {
                return Ok(i);
            }
        }
        Err(VMError::KeyMapNotFound)
    }
}
