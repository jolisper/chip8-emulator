use std::io::Write;

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

const COLUMNS_PER_LINE: u32 = 8;

macro_rules! memaddr_pattern {() => ("{:#06X} | ")}
macro_rules! byte_pattern {() => ("{:#04X} ")}

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

    pub(crate) fn load_program(&mut self, buffer: &[u8]) -> Result<(), VMError> {
        if !(buffer.len() + CHIP8_PROGRAM_LOAD_ADDRESS < CHIP8_MEM_SIZE) {
            return Err(VMError::ProgramSizeOverflow)
        }
        self.memory[CHIP8_PROGRAM_LOAD_ADDRESS..CHIP8_PROGRAM_LOAD_ADDRESS+buffer.len()].copy_from_slice(buffer);
        Ok(())
    }

    pub(crate) fn get_opcode(&self, index: usize) -> Result<u16, VMError> {
        if !(index + 1 < CHIP8_MEM_SIZE) {
            return Err(VMError::MemoryOutOfBounds(index));
        }
        let instruction = ((self.memory[index] as u16) << 8) | self.memory[index + 1] as u16;
        Ok(instruction)
    }

    pub(crate) fn dump(&self) {
        let mut colums_count = 1;
        print!(memaddr_pattern!(), 0); 
        for (memaddr, byte) in self.memory.iter().enumerate() {
            if colums_count > COLUMNS_PER_LINE {
                colums_count = 1;
                println!();
                print!(memaddr_pattern!(), memaddr); 
            }
            print!(byte_pattern!(), byte);
            colums_count += 1;
        }
        std::io::stdout().flush().unwrap();
    }
}
