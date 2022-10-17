use core::fmt;
use std::{error::Error, fmt::Debug};

#[derive(Debug)]
pub enum VMError {
    MemoryOutOfBounds(usize),
    StackOutOfBounds(usize),
    StackOverflow,
    KeyMapNotFound,
    ReservedMemoryWriteAttempt,
    ScreenOutOfBounds(usize, usize),
    ProgramSizeOverflow,
    ProgramCounterOverflow,
}

impl Error for VMError {}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            VMError::MemoryOutOfBounds(index) => {
                write!(f, "invalid memory index: {}", index)
            },
            VMError::StackOutOfBounds(sp) => {
                write!(f, "invalid stack index: {}", sp)
            }
            VMError::StackOverflow => {
                write!(f, "impossible to increment SP register, stackoverflow")
            }
            VMError::KeyMapNotFound => {
                write!(f, "key map not found")
            }
            VMError::ReservedMemoryWriteAttempt => {
                write!(f, "reserved memory write attempt (0x000 to 0x1FF)")
            },
            VMError::ScreenOutOfBounds(x, y) => {
                write!(f, "screen pixel set/unset out of bounds: x={} y={}", x, y)
            },
            VMError::ProgramSizeOverflow => {
                write!(f, "the program size si to big to be loaded in memmory")
            },
            VMError::ProgramCounterOverflow => {
                write!(f, "the PC register overflow")
            },
        }
    }
}

impl From<VMError> for String {
    fn from(vmerr: VMError) -> Self {
       vmerr.to_string() 
    }
}