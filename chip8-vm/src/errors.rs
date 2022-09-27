use core::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum VMError {
    MemoryOutOfBounds(usize),
    StackOutOfBounds(usize),
    StackOverflow,
    KeyMapNotFound,
    ReservedMemoryWriteAttempt,
}

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
                write!(f, "Reserved memory write attempt (0x000 to 0x1FF)")
            },
        }
    }
}

impl Error for VMError {}
