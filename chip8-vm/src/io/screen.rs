use crate::{config::{CHIP8_SCREEN_HEIGHT, CHIP8_SCREEN_WIDTH}, errors::VMError};

pub struct Screen {
    pixels: [[bool; CHIP8_SCREEN_HEIGHT]; CHIP8_SCREEN_WIDTH]
}

impl Default for Screen {
    fn default() -> Self {
        Self { pixels: [[false; CHIP8_SCREEN_HEIGHT]; CHIP8_SCREEN_WIDTH] }
    }
}

impl Screen {

    pub(crate) fn set_pixel(&mut self, x: usize, y: usize) -> Result<(), VMError> {
        if !self.check_bounds(x, y) {
            return Err(VMError::ScreenOutOfBounds(x, y))
        }
        self.pixels[x][y] = true;
        Ok(())
    }

    pub(crate) fn unset_pixel(&mut self, x: usize, y: usize) -> Result<(), VMError> {
        if !self.check_bounds(x, y) {
            return Err(VMError::ScreenOutOfBounds(x, y))
        }
        self.pixels[x][y] = false;
        Ok(())
    }

    pub(crate) fn is_pixel_set(&self, x: usize, y: usize) -> Result<bool, VMError> {
        if !self.check_bounds(x, y) {
            return Err(VMError::ScreenOutOfBounds(x, y))
        }
        Ok(self.pixels[x][y])
    }

    fn check_bounds(&self, x: usize, y: usize) -> bool {
        x < CHIP8_SCREEN_WIDTH && y < CHIP8_SCREEN_HEIGHT 
    }

}