use std::io::Write;

use crate::{
    config::{CHIP8_SCREEN_HEIGHT, CHIP8_SCREEN_WIDTH},
    errors::VMError,
    memory::RAM,
};

pub struct Screen {
    pixels: [[bool; CHIP8_SCREEN_HEIGHT]; CHIP8_SCREEN_WIDTH],
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            pixels: [[false; CHIP8_SCREEN_HEIGHT]; CHIP8_SCREEN_WIDTH],
        }
    }
}

impl Screen {
    pub(crate) fn is_pixel_set(&self, x: usize, y: usize) -> Result<bool, VMError> {
        if !self.check_bounds(x, y) {
            return Err(VMError::ScreenOutOfBounds(x, y));
        }
        Ok(self.pixels[x][y])
    }

    pub(crate) fn draw_sprite(
        &mut self,
        x: usize,
        y: usize,
        offset: usize,
        ram: &RAM,
        tbytes: usize,
    ) -> Result<bool, VMError> {
        let mut pixel_collision = false;

        for ly in 0..tbytes {
            let sprite_byte = ram.get(offset + ly)?;
            for lx in 0..8 {
                // if pixel byte is zero, nothing to draw
                if sprite_byte & (0b1000_0000 >> lx) == 0b0000_0000 {
                    continue;
                }

                // Collision detection.
                if self.pixels[(lx + x) % CHIP8_SCREEN_WIDTH][(ly + y) % CHIP8_SCREEN_HEIGHT] {
                    pixel_collision = true;
                }

                // XOR pixels, if pixel is already "on" then go "off".
                self.pixels[(lx + x) % CHIP8_SCREEN_WIDTH][(ly + y) % CHIP8_SCREEN_HEIGHT] ^= true;
            }
        }
        Ok(pixel_collision)
    }

    fn check_bounds(&self, x: usize, y: usize) -> bool {
        x < CHIP8_SCREEN_WIDTH && y < CHIP8_SCREEN_HEIGHT
    }

    pub fn clear(&mut self) -> Result<(), VMError> {
        self.pixels = [[false; CHIP8_SCREEN_HEIGHT]; CHIP8_SCREEN_WIDTH];
        Ok(())
    }

    pub(crate) fn dump(&self) {
        for x in 0..CHIP8_SCREEN_HEIGHT {
            for y in 0..CHIP8_SCREEN_WIDTH {
                if self.pixels[y][x] {
                    print!("{},", "X");
                } else {
                    print!("{},", ".");
                }
            }
            println!();
        }
        std::io::stdout().flush().unwrap();
    }
}
