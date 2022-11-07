use std::io::Write;

use crate::{
    config::{CHIP8_MEM_SIZE, CHIP8_TOTAL_STACK_DEPTH},
    errors::VMError,
};

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
    pc: u16,

    /// Stack pointer.
    sp: u8,
}

impl Registers {
    pub(crate) fn inc_sp(&mut self) -> Result<(), VMError> {
        self.validate_sp_in_bounds()?;
        self.sp += 1;
        Ok(())
    }

    pub(crate) fn dec_sp(&mut self) -> Result<(), VMError> {
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
        if (self.sp as usize) < CHIP8_TOTAL_STACK_DEPTH {
            return Ok(());
        }
        Err(VMError::StackOverflow)
    }

    pub(crate) fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    pub(crate) fn get_pc(&self) -> u16 {
        self.pc
    }

    pub(crate) fn inc_pc(&mut self) -> Result<(), VMError> {
        if !(self.pc < CHIP8_MEM_SIZE as u16) {
            return Err(VMError::ProgramCounterOverflow);
        }
        self.pc += 2; // The opcodes are 2 bytes long
        Ok(())
    }

    pub(crate) fn dec_pc(&mut self) -> Result<(), VMError> {
        if self.pc < 2 {
            return Err(VMError::ProgramCounterUnderflow);
        }
        self.pc -= 2;
        return Ok(());
    }

    pub(crate) fn set_v_register(&mut self, index: usize, value: u8) {
        (*self.v_registers_mut()[index]) = value;
    }

    pub(crate) fn get_v_register(&self, index: usize) -> u8 {
        *self.v_registers()[index]
    }

    /// Returns an inmutable array view of the V registers
    fn v_registers(&self) -> [&u8; 16] {
        [
            &self.v_0, &self.v_1, &self.v_2, &self.v_3, &self.v_4, &self.v_5, &self.v_6, &self.v_7,
            &self.v_8, &self.v_9, &self.v_a, &self.v_b, &self.v_c, &self.v_d, &self.v_e, &self.v_f,
        ]
    }

    /// Returns a mutable array view of the V registers
    fn v_registers_mut(&mut self) -> [&mut u8; 16] {
        [
            &mut self.v_0,
            &mut self.v_1,
            &mut self.v_2,
            &mut self.v_3,
            &mut self.v_4,
            &mut self.v_5,
            &mut self.v_6,
            &mut self.v_7,
            &mut self.v_8,
            &mut self.v_9,
            &mut self.v_a,
            &mut self.v_b,
            &mut self.v_c,
            &mut self.v_d,
            &mut self.v_e,
            &mut self.v_f,
        ]
    }

    pub(crate) fn set_vf(&mut self) {
        self.v_f = 1;
    }

    pub(crate) fn unset_vf(&mut self) {
        self.v_f = 0;
    }

    pub(crate) fn get_vf(&self) -> u8 {
        self.v_f
    }

    pub(crate) fn set_i(&mut self, value: u16) {
        self.i = value;
    }

    pub(crate) fn get_i(&self) -> u16 {
        self.i
    }

    pub(crate) fn get_dt(&self) -> u8 {
        self.dt
    }

    pub(crate) fn set_dt(&mut self, value: u8) {
        self.dt = value;
    }

    pub(crate) fn get_sp(&self) -> u8 {
        self.sp
    }

    pub(crate) fn set_st(&mut self, value: u8) {
        self.st = value;
    }

    pub(crate) fn get_st(&self) -> u8 {
        self.st
    }

    pub(crate) fn dump(&self) {
        for (i, register) in self.v_registers().iter().enumerate() {
            print!(
                "V{:X}: dec:   {:03}, hex:   {:02X}, bin:         {:08b}\n",
                i, register, register, register
            );
        }
        print!(
            "DT: dec:   {:03}, hex:   {:02X}, bin:         {:08b}\n",
            self.dt, self.dt, self.dt
        );
        print!(
            "ST: dec:   {:03}, hex:   {:02X}, bin:         {:08b}\n",
            self.st, self.st, self.st
        );
        print!(
            "SP: dec:   {:03}, hex:   {:02X}, bin:         {:08b}\n",
            self.sp, self.sp, self.sp
        );
        print!(
            "I : dec: {:05}, hex: {:04X}, bin: {:016b}\n",
            self.i, self.i, self.i
        );
        print!(
            "PC: dec: {:05}, hex: {:04X}, bin: {:016b}\n",
            self.pc, self.pc, self.pc
        );
        std::io::stdout().flush().unwrap();
    }
}
