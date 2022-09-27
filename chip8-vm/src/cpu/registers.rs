use crate::{errors::VMError, config::CHIP8_TOTAL_STACK_DEPTH};

#[derive(Default)]
pub struct Registers {
    /// V general purpose registers, usually referred to as Vx, where x is a hexadecimal digit (0 through F).
    /// The VF register should not be used by any program, as it is  used as a flag by some instructions.
    pub v_0: u8,
    pub v_1: u8,
    pub v_2: u8,
    pub v_3: u8,
    pub v_4: u8,
    pub v_5: u8,
    pub v_6: u8,
    pub v_7: u8,
    pub v_8: u8,
    pub v_9: u8,
    pub v_a: u8,
    pub v_b: u8,
    pub v_c: u8,
    pub v_d: u8,
    pub v_e: u8,
    pub v_f: u8,

    /// Index register to store memory addresses.
    pub i: u16,

    /// Sound delay and timer registers.
    pub sd: u8,
    pub st: u8,

    /// Program counter.
    pub pc: u16,

    /// Stack pointer.
    pub sp: i8,
}

impl Registers {

    pub(crate) fn inc_sp(&mut self) -> Result<(), VMError> {
        self.validate_sp_in_bounds()?;
        self.sp += 1;
        Ok(())
    }

    pub(crate) fn dec_sp(&mut self) -> Result<(), VMError>  {
        self.sp -= 1;
        self.validate_sp_in_bounds()?;
        Ok(())
    }

    pub(crate) fn validate_sp_in_bounds(&self) -> Result<(), VMError>{
        if (self.sp as usize) < CHIP8_TOTAL_STACK_DEPTH && self.sp >= 0 {
            return Ok(())
        } 
        Err(VMError::StackOverflow)
    }
}


