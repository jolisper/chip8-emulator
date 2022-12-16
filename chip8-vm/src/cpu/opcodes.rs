use std::cell::Cell;

use crate::{
    config::CHIP8_TOTAL_STANDARD_OPCODES,
    errors::VMError,
    io::{Keyboard, Screen},
    memory::RAM,
};

use super::{Registers, Stack};

pub enum Signal {
    NoSignal,
    DrawScreen,
    WaitKeyUp(u8),
}

type OpcodeInstructions = fn(
    opcode: u16,
    stack: &mut Stack,
    memory: &mut RAM,
    registers: &mut Registers,
    keyboard: &Keyboard,
    screen: &mut Screen,
) -> Result<Signal, VMError>;

type OpcodeDump = fn(
    pattern: &'static str,
    opcode: u16,
    stack: &mut Stack,
    memory: &mut RAM,
    registers: &mut Registers,
    keyboard: &Keyboard,
    screen: &mut Screen,
);

const CLS: Opcode = Opcode {
    pattern: "00E0;CLS",
    bitmask: 0xFFFF,
    match_value: 0x00E0,
    instructions: cls,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const RET: Opcode = Opcode {
    pattern: "00EE;RET",
    bitmask: 0xFFFF,
    match_value: 0x00EE,
    instructions: ret,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SYS: Opcode = Opcode {
    pattern: "0nnn;SYS addr",
    bitmask: 0xF000,
    match_value: 0x0000,
    instructions: sys,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const JP: Opcode = Opcode {
    pattern: "1nnn;JP addr",
    bitmask: 0xF000,
    match_value: 0x1000,
    instructions: jp,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const CALL: Opcode = Opcode {
    pattern: "2nnn;CALL addr",
    bitmask: 0xF000,
    match_value: 0x2000,
    instructions: call,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SE_VX_BYTE: Opcode = Opcode {
    pattern: "3xkk;SE Vx, byte",
    bitmask: 0xF000,
    match_value: 0x3000,
    instructions: se_vx_kk,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SNE_VX_KK: Opcode = Opcode {
    pattern: "4xkk;SNE Vx, byte",
    bitmask: 0xF000,
    match_value: 0x4000,
    instructions: sne_vx_kk,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SE_VX_VY: Opcode = Opcode {
    pattern: "5xy0;SE Vx, Vy",
    bitmask: 0xF000,
    match_value: 0x5000,
    instructions: se_vx_vy,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_VX_BYTE: Opcode = Opcode {
    pattern: "6xkk;LD Vx, byte",
    bitmask: 0xF000,
    match_value: 0x6000,
    instructions: ld_vx_kk,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const ADD_VX_BYTE: Opcode = Opcode {
    pattern: "7xkk;ADD Vx, byte",
    bitmask: 0xF000,
    match_value: 0x7000,
    instructions: add_vx_kk,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_VX_VY: Opcode = Opcode {
    pattern: "8xy0;LD Vx, Vy",
    bitmask: 0xF00F,
    match_value: 0x8000,
    instructions: ld_vx_vy,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const OR_VX_VY: Opcode = Opcode {
    pattern: "8xy1;OR Vx, Vy",
    bitmask: 0xF00F,
    match_value: 0x8001,
    instructions: or_vx_vy,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const AND_VX_VY: Opcode = Opcode {
    pattern: "8xy2;AND Vx, Vy",
    bitmask: 0xF00F,
    match_value: 0x8002,
    instructions: and_vx_vy,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const XOR_VX_VY: Opcode = Opcode {
    pattern: "8xy3;XOR Vx, Vy",
    bitmask: 0xF00F,
    match_value: 0x8003,
    instructions: xor_vx_vy,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const ADD_VX_VY: Opcode = Opcode {
    pattern: "8xy4;ADD Vx, Vy",
    bitmask: 0xF00F,
    match_value: 0x8004,
    instructions: add_vx_vy,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SUB_VX_VY: Opcode = Opcode {
    pattern: "8xy5;SUB Vx, Vy",
    bitmask: 0xF00F,
    match_value: 0x8005,
    instructions: sub_vx_vy,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SHR_VX: Opcode = Opcode {
    pattern: "8xy6;SHR Vx",
    bitmask: 0xF00F,
    match_value: 0x8006,
    instructions: shr_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SUBN_VX_VY: Opcode = Opcode {
    pattern: "8xy7;SUBN Vx, Vy",
    bitmask: 0xF00F,
    match_value: 0x8007,
    instructions: subn_vx_vy,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SHL_VX: Opcode = Opcode {
    pattern: "8xyE;SHL Vx",
    bitmask: 0xF00F,
    match_value: 0x800E,
    instructions: shl_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SNE_VX_VY: Opcode = Opcode {
    pattern: "9xy0;SNE Vx, Vy ",
    bitmask: 0xF00F,
    match_value: 0x9000,
    instructions: sne_vx_vy,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_I_ADDR: Opcode = Opcode {
    pattern: "Annn;LD I, addr",
    bitmask: 0xF000,
    match_value: 0xA000,
    instructions: ld_i_addr,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const JP_V0_ADDR: Opcode = Opcode {
    pattern: "Bnnn;JP V0, addr",
    bitmask: 0xF000,
    match_value: 0xB000,
    instructions: jp_v0_addr,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const RND_VX_BYTE: Opcode = Opcode {
    pattern: "Cxkk;RND Vx, byte",
    bitmask: 0xF000,
    match_value: 0xC000,
    instructions: rnd_vx_byte,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const DRW_VX_VY_NB: Opcode = Opcode {
    pattern: "Dxyn;DRW Vx, Vy, nibble",
    bitmask: 0xF000,
    match_value: 0xD000,
    instructions: drw_vx_vy_nb,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SKP_VX: Opcode = Opcode {
    pattern: "Ex9E;SKP Vx",
    bitmask: 0xF0FF,
    match_value: 0xE09E,
    instructions: skp_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const SKNP_VX: Opcode = Opcode {
    pattern: "ExA1;SKNP Vx",
    bitmask: 0xF0FF,
    match_value: 0xE0A1,
    instructions: sknp_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_VX_DTIMER: Opcode = Opcode {
    pattern: "Fx07;LD Vx, DT",
    bitmask: 0xF0FF,
    match_value: 0xF007,
    instructions: ld_vx_dt,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_VX_K: Opcode = Opcode {
    pattern: "Fx0A;LD Vx, K",
    bitmask: 0xF0FF,
    match_value: 0xF00A,
    instructions: ld_vx_key,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_DTIMER_VX: Opcode = Opcode {
    pattern: "Fx15;LD DT, Vx,",
    bitmask: 0xF0FF,
    match_value: 0xF015,
    instructions: ld_dt_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_STIMER_VX: Opcode = Opcode {
    pattern: "Fx18;LD ST, Vx",
    bitmask: 0xF0FF,
    match_value: 0xF018,
    instructions: ld_st_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const ADD_I_VX: Opcode = Opcode {
    pattern: "Fx1E;ADD I, Vx",
    bitmask: 0xF0FF,
    match_value: 0xF01E,
    instructions: add_i_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_F_VX: Opcode = Opcode {
    pattern: "Fx29;LD I, Vx (hex sprite value)",
    bitmask: 0xF0FF,
    match_value: 0xF029,
    instructions: ld_f_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_BCD_VX: Opcode = Opcode {
    pattern: "Fx33;LD [I] (BDC of Vx), Vx",
    bitmask: 0xF0FF,
    match_value: 0xF033,
    instructions: ld_bcd_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_I_VX: Opcode = Opcode {
    pattern: "Fx55;LD [I], V0-Vx",
    bitmask: 0xF0FF,
    match_value: 0xF055,
    instructions: ld_i_vx,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};
const LD_VX_I: Opcode = Opcode {
    pattern: "Fx65;LD V0-Vx, [I]",
    bitmask: 0xF0FF,
    match_value: 0xF065,
    instructions: ld_vx_i,
    pre_ex_dump: dft_pre_ex_dump,
    post_ex_dump: dft_post_ex_dump,
};

pub const OPCODES: [Opcode; CHIP8_TOTAL_STANDARD_OPCODES] = [
    CLS,
    RET,
    SYS,
    JP,
    CALL,
    SE_VX_BYTE,
    SNE_VX_KK,
    SE_VX_VY,
    LD_VX_BYTE,
    ADD_VX_BYTE,
    LD_VX_VY,
    OR_VX_VY,
    AND_VX_VY,
    XOR_VX_VY,
    ADD_VX_VY,
    SUB_VX_VY,
    SHR_VX,
    SUBN_VX_VY,
    SHL_VX,
    SNE_VX_VY,
    LD_I_ADDR,
    JP_V0_ADDR,
    RND_VX_BYTE,
    DRW_VX_VY_NB,
    SKP_VX,
    SKNP_VX,
    LD_VX_DTIMER,
    LD_VX_K,
    LD_DTIMER_VX,
    LD_STIMER_VX,
    ADD_I_VX,
    LD_F_VX,
    LD_BCD_VX,
    LD_I_VX,
    LD_VX_I,
];

fn dft_pre_ex_dump(
    pattern: &'static str,
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) {
    let mut pre_ex_dump = format!("{:#06X}: {:#06X} /", registers.get_pc(), opcode);
    let split = pattern.split(";").collect::<Vec<&str>>();
    let opcode_str = *split.get(0).unwrap();
    let desc = format!(" {} /", *split.get(1).unwrap());

    pre_ex_dump.push_str(&desc);

    if opcode_str.contains("x") {
        let vx_index = ((opcode & 0x0F00) >> 8) as usize;
        let vx_value = registers.get_v_register(vx_index);
        let vx = format!(" V{:X} = {:#04X}", vx_index, vx_value);
        pre_ex_dump.push_str(&vx);
    }

    if opcode_str.contains("y") {
        let vy_index = ((opcode & 0x00F0) >> 4) as usize;
        let vy_value = registers.get_v_register(vy_index);
        let vy = format!(", V{:X} = {:#04X},", vy_index, vy_value);
        pre_ex_dump.push_str(&vy);
    }

    if opcode_str.contains("nnn") {
        let address = opcode & 0x0FFF;
        let vy = format!(" NNN = {:#05X}", address);
        pre_ex_dump.push_str(&vy);
    }

    if opcode_str.ends_with("n") && !opcode_str.ends_with("nnn") {
        let nbytes = opcode & 0x000F;
        let vy = format!(" N = {:#03X}", nbytes);
        pre_ex_dump.push_str(&vy);
    }

    if opcode_str.contains("kk") {
        let byte = opcode & 0x00FF;
        let vy = format!(" KK = {:#04X}", byte);
        pre_ex_dump.push_str(&vy);
    }

    println!("{}", pre_ex_dump);
}

fn dft_post_ex_dump(
    _pattern: &'static str,
    _opcode: u16,
    stack: &mut Stack,
    memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    screen: &mut Screen,
) {
    registers.dump();
    stack.dump();
    memory.dump();
    screen.dump();
}

pub struct Opcode {
    pattern: &'static str,
    bitmask: u16,
    match_value: u16,
    instructions: OpcodeInstructions,
    pre_ex_dump: OpcodeDump,
    post_ex_dump: OpcodeDump,
}

impl Opcode {
    pub fn check(&self, binary_opcode: u16) -> bool {
        let masked = binary_opcode & self.bitmask;
        masked == self.match_value
    }

    pub fn instructions(&self) -> OpcodeInstructions {
        self.instructions
    }

    pub fn pre_ex_dump(&self) -> OpcodeDump {
        self.pre_ex_dump
    }

    pub fn post_ex_dump(&self) -> OpcodeDump {
        self.post_ex_dump
    }

    pub fn pattern(&self) -> &'static str {
        self.pattern
    }
}

// Instructions for opcode pattern 00E0. Clear the display.
fn sys(
    _opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    // This instruction is only used on the old computers on which Chip-8 was originally implemented.
    // It is ignored by modern interpreters.
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 00E0. Clear the display.
fn cls(
    _opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    screen: &mut Screen,
) -> Result<Signal, VMError> {
    screen.clear()?;
    registers.inc_pc()?;
    Ok(Signal::DrawScreen)
}

// Instructions for opcode pattern 00EE. Return from subroutine.
fn ret(
    _opcode: u16,
    stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    registers.dec_sp()?;
    registers.set_pc(stack.get_at(registers.get_sp())?);
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 1nnn. Jump to location nnn.
fn jp(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let jump_address = opcode & 0x0FFF;
    registers.set_pc(jump_address);
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 2nnn. Call subroutine at address nnn.
fn call(
    opcode: u16,
    stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    registers.inc_pc()?;
    stack.set_at(registers.get_sp(), registers.get_pc())?;
    registers.inc_sp()?;
    let call_address = opcode & 0x0FFF;
    registers.set_pc(call_address);
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 3xkk. Skip next instruction if Vx = kk.
fn se_vx_kk(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let kk_value = (opcode & 0x00FF) as u8;
    if vx_value == kk_value {
        registers.inc_pc()?;
    }
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 4xkk. Skip next instruction if Vx != kk.
fn sne_vx_kk(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let kk_value: u8 = (opcode & 0x00FF) as u8;
    if vx_value != kk_value {
        registers.inc_pc()?;
    }
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 5xy0. Skip next instruction if Vx = Vy.
fn se_vx_vy(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let vy_value = registers.get_v_register(vy_index);
    if vx_value == vy_value {
        registers.inc_pc()?;
    }
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 6xkk. Set Vx = kk.
fn ld_vx_kk(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let kk_value = (opcode & 0x00FF) as u8;
    registers.set_v_register(vx_index, kk_value);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 7xkk. Set Vx = Vx + kk.
fn add_vx_kk(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let kk_value = (opcode & 0x00FF) as u8;
    registers.set_v_register(vx_index, vx_value.wrapping_add(kk_value));
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 8xy0. Set Vx = Vy.
fn ld_vx_vy(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vy_value = registers.get_v_register(vy_index);
    registers.set_v_register(vx_index, vy_value);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 8xy1. Set Vx = Vx OR Vy.
fn or_vx_vy(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let vy_value = registers.get_v_register(vy_index);
    registers.set_v_register(vx_index, vx_value | vy_value);
    registers.unset_vf();
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 8xy2. Set Vx = Vx AND Vy.
fn and_vx_vy(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let vy_value = registers.get_v_register(vy_index);
    registers.set_v_register(vx_index, vx_value & vy_value);
    registers.unset_vf();
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 8xy3. Set Vx = Vx OR Vy.
fn xor_vx_vy(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let vy_value = registers.get_v_register(vy_index);
    registers.set_v_register(vx_index, vx_value ^ vy_value);
    registers.unset_vf();
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 8xy4. Set Vx = Vx + Vy, with carry.
fn add_vx_vy(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vx_value = registers.get_v_register(vx_index) as u16;
    let vy_value = registers.get_v_register(vy_index) as u16;
    let addition = vx_value + vy_value;

    registers.unset_vf();
    if addition > std::u8::MAX as u16 {
        registers.set_vf();
    }
    registers.set_v_register(vx_index, addition as u8);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 8xy5. Set Vx = Vx - Vy, set VF = NOT borrow.
fn sub_vx_vy(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let vy_value = registers.get_v_register(vy_index);
    registers.unset_vf();
    if vx_value > vy_value {
        registers.set_vf();
    }
    registers.set_v_register(vx_index, vx_value.wrapping_sub(vy_value));
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 8xy6. Set Vx = Vx SHR 1, (shift right) set VF if truncation occuers.
fn shr_vx(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vy_value = registers.get_v_register(vy_index);

    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    registers.set_v_register(vx_index, vy_value >> 1);

    let vx_lsb = vy_value & 0b0000_0001;
    registers.unset_vf();
    if vx_lsb == 1 {
        registers.set_vf();
    }
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 8xy7. Set Vx = Vy - Vx, with carry (if Vy > Vx).
fn subn_vx_vy(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let vy_value = registers.get_v_register(vy_index);
    registers.unset_vf();
    if vy_value > vx_value {
        registers.set_vf();
    }
    registers.set_v_register(vx_index, vy_value.wrapping_sub(vx_value));
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 8xyE. Set Vx = Vx SHL 1, (shift right) set VF if truncation occurs.
fn shl_vx(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vy_value = registers.get_v_register(vy_index);

    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    registers.set_v_register(vx_index, vy_value << 1);

    let vx_msb = vy_value & 0b1000_0000;
    registers.unset_vf();
    if vx_msb > 0 {
        registers.set_vf();
    }
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern 9xy0. Skip next instruction if Vx != Vy.
fn sne_vx_vy(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vy_index = ((opcode & 0x00F0) >> 4) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let vy_value = registers.get_v_register(vy_index);
    if vx_value != vy_value {
        registers.inc_pc()?;
    }
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Annn. Set I = nnn.
fn ld_i_addr(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let nnn_value = opcode & 0x0FFF;
    registers.set_i(nnn_value);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Bnnn. Jump to location nnn + V0.
fn jp_v0_addr(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let jump_address = opcode & 0x0FFF;
    let offset = registers.get_v_register(0) as u16;
    registers.set_pc(jump_address + offset);
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Cxkk. Set Vx = random byte AND kk.
fn rnd_vx_byte(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let kk_value = (opcode & 0x00FF) as u8;
    let rnd = rand::random::<u8>();
    registers.set_v_register(vx_index, rnd & kk_value);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Dxyn. Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
fn drw_vx_vy_nb(
    opcode: u16,
    _stack: &mut Stack,
    memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_value = registers.get_v_register(((opcode & 0x0F00) >> 8) as usize);
    let vy_value = registers.get_v_register(((opcode & 0x00F0) >> 4) as usize);
    let nbytes = (opcode & 0x000F) as usize;
    let offset = registers.get_i();
    registers.unset_vf();
    if screen.draw_sprite(
        vx_value as usize,
        vy_value as usize,
        offset as usize,
        memory,
        nbytes,
    )? {
        registers.set_vf();
    }
    registers.inc_pc()?;
    Ok(Signal::DrawScreen)
}

// Instructions for opcode pattern Ex9E. Skip next instruction if key with the value of Vx is pressed.
fn skp_vx(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index);
    if keyboard.is_key_down(vx_value) {
        registers.inc_pc()?;
    }
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Ex9E. Skip next instruction if key with the value of Vx is not pressed.
fn sknp_vx(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index);
    if keyboard.is_key_up(vx_value) {
        registers.inc_pc()?;
    }
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Fx07. Set Vx = delay timer value.
fn ld_vx_dt(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let delay_timer = registers.get_dt();
    registers.set_v_register(vx_index, delay_timer);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

thread_local! {
    static WAIT_KEYUP: Cell<Option<u8>> = Cell::new(None);
}

// Instructions for opcode pattern Fx0A. Wait for a key press, store the value fo the key in Vx.
fn ld_vx_key(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    WAIT_KEYUP.with(|f| {
        let mut signal = Ok(Signal::NoSignal);
        match f.get() {
            Some(key) => {
                if keyboard.is_key_up(key) {
                    registers.set_v_register(((opcode & 0x0F00) >> 8) as usize, key);
                    registers.inc_pc()?;
                    f.set(None);
                }
            }
            None => {
                // check for key down through all keys
                for key in 0x0..=0xF as u8 {
                    if keyboard.is_key_down(key) {
                        signal = Ok(Signal::WaitKeyUp(key));
                        f.set(Some(key));
                    }
                }
            }
        }
        signal
    })
}

// Instructions for opcode pattern Fx15. Set delay timer = Vx.
fn ld_dt_vx(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index);
    registers.set_dt(vx_value);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Fx18. Set sound timer = Vx.
fn ld_st_vx(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index);
    registers.set_st(vx_value);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Fx1E. Set I = I + Vx.
fn add_i_vx(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index) as u16;
    registers.set_i(registers.get_i() + vx_value);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Fx29. Set I = location of sprite for digit Vx.
fn ld_f_vx(
    opcode: u16,
    _stack: &mut Stack,
    _memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index);
    let char_addr = vx_value * 5;
    registers.set_i(char_addr as u16);
    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Fx33. Store BCD representation of Vx in memory locations I, I+1, +2.
fn ld_bcd_vx(
    opcode: u16,
    _stack: &mut Stack,
    memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let vx_value = registers.get_v_register(vx_index);

    // Bitmasks for BCD units
    let ones_mask = 0x000000F00;
    let tens_mask = 0x00000F000;
    let hund_mask = 0x0000F0000;

    // BCD units
    let mut ones = 0u32;
    let mut tens = 0u32;
    let mut hund = 0u32;

    // Double dabble algorithm
    let mut bcd = vx_value as u32;
    for i in 0..8 {
        bcd = bcd << 1;
        ones = (bcd & ones_mask) >> 8;
        tens = (bcd & tens_mask) >> 12;
        hund = (bcd & hund_mask) >> 16;

        // In the last shift no need to handle carry
        if i == 7 {
            break;
        }

        // Handle carries
        if ones >= 5 {
            ones += 3;
        }

        if tens >= 5 {
            tens = tens + 3;
        }

        if hund >= 5 {
            hund = hund + 3;
        }

        // Return units to its position in BCD number
        ones = ones << 8;
        tens = tens << 12;
        hund = hund << 16;

        // Reassemble BCD for next shift after carries
        bcd = (bcd & !(hund_mask | tens_mask | ones_mask)) | (hund | tens | ones);
    }

    // Store BCD in memory: hundreds at I, tens at I+1, ones at I+2.
    let addr = registers.get_i() as usize;
    for (offset, unit) in [hund as u8, tens as u8, ones as u8].into_iter().enumerate() {
        memory.set(addr + offset, unit)?;
    }

    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Fx55. Store registers V0 through Vx in memory starting at location I.
fn ld_i_vx(
    opcode: u16,
    _stack: &mut Stack,
    memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let base_addr = registers.get_i() as usize;

    for vx in 0..=vx_index {
        memory.set(base_addr + vx, registers.get_v_register(vx))?;
        registers.inc_i();
    }
    registers.inc_i();

    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

// Instructions for opcode pattern Fx65. Read registers V0 through Vx from memory starting at location I.
fn ld_vx_i(
    opcode: u16,
    _stack: &mut Stack,
    memory: &mut RAM,
    registers: &mut Registers,
    _keyboard: &Keyboard,
    _screen: &mut Screen,
) -> Result<Signal, VMError> {
    let vx_index = ((opcode & 0x0F00) >> 8) as usize;
    let base_addr = registers.get_i() as usize;

    for vx in 0..=vx_index {
        let value = memory.get(base_addr + vx)?;
        registers.set_v_register(vx, value);
        registers.inc_i();
    }
    registers.inc_i();

    registers.inc_pc()?;
    Ok(Signal::NoSignal)
}
