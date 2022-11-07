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
    pub(crate) fn set_at(&mut self, sp: u8, value: u16) -> Result<(), VMError> {
        self.stack[sp as usize] = value;
        Ok(())
    }

    pub(crate) fn get_at(&self, sp: u8) -> Result<u16, VMError> {
        Ok(self.stack[sp as usize])
    }

    pub(crate) fn dump(&self) {
        print!("[");
        for addr in self.stack {
            print!("{:#06X} ", addr);
        }
        println!("]");
    }
}
