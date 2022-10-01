use crate::errors::VMError;

use crate::{config::*};

const CHARSET: &'static [u8] = &[ 
    // 0 
    0xf0, 0x90, 0x90, 0x90, 0xf0,
    // 1
    0x20, 0x60, 0x20, 0x20, 0x70,
    // 2
    0xf0, 0x10, 0xf0, 0x80, 0xf0,
    // 3
    0xf0, 0x10, 0xf0, 0x10, 0xf0,
    // 4 
    0x90, 0x90, 0xf0, 0x10, 0x10,
    // 5 
    0xf0, 0x80, 0xf0, 0x10, 0xf0,
    // 6
    0xf0, 0x80, 0xf0, 0x90, 0xf0,
    // 7
    0xf0, 0x10, 0x20, 0x40, 0x40,
    // 8
    0xf0, 0x90, 0xf0, 0x90, 0xf0,
    // 9 
    0xf0, 0x90, 0xf0, 0x10, 0xf0,
    // A
    0xf0, 0x90, 0xf0, 0x90, 0x90,
    // B
    0xe0, 0x90, 0xe0, 0x90, 0xe0,
    // C
    0xf0, 0x80, 0x80, 0x80, 0xf0,
    // D
    0xe0, 0x90, 0x90, 0x90, 0xe0,
    // E
    0xf0, 0x80, 0xf0, 0x80, 0xf0,
    // F
    0xf0, 0x80, 0xf0, 0x80, 0x80,
];

pub struct RAM {
    memory: [u8; CHIP8_MEM_SIZE],
}

impl Default for RAM {
    fn default() -> Self {
        let mut ram = Self {
            memory: [0x00; CHIP8_MEM_SIZE],
        };
        // Set the default chatset at the beginning of reserved memory. 
        ram.memory[..CHARSET.len()].copy_from_slice(CHARSET);

        ram
    }
}

impl RAM {

    pub(crate) fn set(&mut self, index: usize, value: u8) -> Result<(), VMError> {
        if index > CHIP8_MEM_SIZE - 1 {
            return Err(VMError::MemoryOutOfBounds(index));
        }
        if index <= CHIP8_MEM_RESEVED_LIMIT {
            return Err(VMError::ReservedMemoryWriteAttempt)
        }
        self.memory[index] = value;
        Ok(())
    }

    pub(crate) fn get(&self, index: usize) -> Result<u8, VMError> {
        let value = self.memory.get(index);
        match value {
            Some(value) => Ok(*value),
            None => Err(VMError::MemoryOutOfBounds(index)),
        }
    }

    pub(crate) fn get_ref(&self, offset: usize) -> &[u8] {
        &self.memory[offset..]
    }
}
