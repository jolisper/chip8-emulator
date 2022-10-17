use crate::{errors::VMError, config::{CHIP8_TOTAL_STACK_DEPTH, CHIP8_MEM_SIZE}};

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
    pub dt: u8,
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

    pub(crate) fn dec_dt(&mut self) {
        self.dt -= 1;
    }

    pub(crate) fn dec_st(&mut self) {
        self.st -= 1;
    }

    fn validate_sp_in_bounds(&self) -> Result<(), VMError> {
        if (self.sp as usize) < CHIP8_TOTAL_STACK_DEPTH && self.sp >= 0 {
            return Ok(())
        } 
        Err(VMError::StackOverflow)
    }

    pub(crate) fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub(crate) fn get_pc(&self) -> usize {
        self.pc as usize
    }

    pub(crate) fn inc_pc(&mut self)  -> Result<(), VMError> {
        if !(self.pc < CHIP8_MEM_SIZE as u16) {
            return Err(VMError::ProgramCounterOverflow)
        }
        self.pc += 1;
        Ok(())
    }

}


