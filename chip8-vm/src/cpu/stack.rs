use crate::{config::CHIP8_TOTAL_STACK_DEPTH, errors::VMError};

pub struct Stack {
    stack: [u16; CHIP8_TOTAL_STACK_DEPTH],
}

impl Default for Stack {
    fn default() -> Self {
        Stack {
            stack: [0x0000; CHIP8_TOTAL_STACK_DEPTH],
        }
    }
}

impl Stack {

    pub(crate) fn push(&mut self, sp: i8, value: u16) -> Result<(), VMError> {
        let spuz = sp as usize;
        self.stack[spuz] = value;
        Ok(())
    }

    pub(crate) fn pop(&self, sp: i8) -> Result<u16, VMError> {
        let spuz = sp as usize;
        let value = self.stack[spuz];
        Ok(value)
    }
}
