use std::cell::Cell;

use crate::{
    config::CHIP8_TOTAL_STANDARD_OPCODES,
    errors::VMError,
    io::{Keyboard, Screen},
    memory::RAM,
};

use super::{Registers, Stack};

/// Get the Vx index (0x0x00) from the opcode.
macro_rules! vx_index {
    ($ctx:ident) => {
        (($ctx.opcode & 0x0F00) >> 8) as usize
    };
}

/// Get the value from the Vx register (0x0x00).
macro_rules! vx_value {
    ($ctx:ident) => {{
        let vx_index = vx_index!($ctx);
        $ctx.registers.get_v_register(vx_index)
    }};
}

/// Get the Vy index (0x00y0) from the opcode.
macro_rules! vy_index {
    ($ctx:ident) => {
        (($ctx.opcode & 0x00F0) >> 4) as usize
    };
}

/// Get the value from the Vy register (0x00y0).
macro_rules! vy_value {
    ($ctx:ident) => {{
        let vy_index = vy_index!($ctx);
        $ctx.registers.get_v_register(vy_index)
    }};
}

/// Get the KK value (0x00KK) from the opcode.
macro_rules! kk_value {
    ($ctx:ident) => {
        ($ctx.opcode & 0x00FF) as u8
    };
}

/// Get the NNN value (0x0NNN) from the opcode.
macro_rules! nnn_value {
    ($ctx:ident) => {
        ($ctx.opcode & 0x0FFF) as u16
    };
}

/// Get the NBytes value (0x000N) from the opcode.
macro_rules! nbytes_value {
    ($ctx:ident) => {
        ($ctx.opcode & 0x000F) as usize
    };
}

/// `Signal` is the return type used by instruction functions of an opcode to signal the VM
/// front-end what behaviour is expected after the instructions execution.
pub enum Signal {
    NoSignal,
    DrawScreen,
    WaitKeyUp(u8),
}

/// The `VMContext` represent the current state of the VM that the instructions of an opcode
/// take as starting point of its own execution.
pub struct VMContext<'a> {
    pub opcode: u16,
    pub stack: &'a mut Stack,
    pub memory: &'a mut RAM,
    pub registers: &'a mut Registers,
    pub keyboard: &'a Keyboard,
    pub screen: &'a mut Screen,
    pub pattern: &'a str,
}

/// Opcode instructions function signature.
type OpcodeInstructions = fn(cxt: &mut VMContext) -> Result<Signal, VMError>;

/// Opcode dump function signature.
type OpcodeDump = fn(ctx: &VMContext);

/// The `OpcodeMatcher` holds a mapping between the matching info for a binary opcode and the 
/// opcode instructions. Also holds the debugging functions for the opcode.
pub struct OpcodeMatcher {
    bitmask: u16,
    match_value: u16,
    instructions: OpcodeInstructions,
    pre_ex_dump: OpcodeDump,
    post_ex_dump: OpcodeDump,
    desc: &'static str,
}

impl OpcodeMatcher {
    pub fn check_matching(&self, binary_opcode: u16) -> bool {
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

    pub fn desc(&self) -> &'static str {
        self.desc
    }
}

/// Opcode matchers constant table, sets the configuration for each of the matchers.
#[rustfmt::skip] const CLS:          OpcodeMatcher = OpcodeMatcher { bitmask: 0xFFFF, match_value: 0x00E0, instructions: cls,          pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "00E0;CLS",                         };
#[rustfmt::skip] const RET:          OpcodeMatcher = OpcodeMatcher { bitmask: 0xFFFF, match_value: 0x00EE, instructions: ret,          pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "00EE;RET",                         };
#[rustfmt::skip] const SYS:          OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0x0000, instructions: sys,          pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "0nnn;SYS addr",                    };
#[rustfmt::skip] const JP:           OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0x1000, instructions: jp,           pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "1nnn;JP addr",                     };
#[rustfmt::skip] const CALL:         OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0x2000, instructions: call,         pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "2nnn;CALL addr",                   };
#[rustfmt::skip] const SE_VX_BYTE:   OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0x3000, instructions: se_vx_kk,     pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "3xkk;SE Vx, byte",                 };
#[rustfmt::skip] const SNE_VX_KK:    OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0x4000, instructions: sne_vx_kk,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "4xkk;SNE Vx, byte",                };
#[rustfmt::skip] const SE_VX_VY:     OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0x5000, instructions: se_vx_vy,     pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "5xy0;SE Vx, Vy",                   };
#[rustfmt::skip] const LD_VX_BYTE:   OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0x6000, instructions: ld_vx_kk,     pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "6xkk;LD Vx, byte",                 };
#[rustfmt::skip] const ADD_VX_BYTE:  OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0x7000, instructions: add_vx_kk,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "7xkk;ADD Vx, byte",                };
#[rustfmt::skip] const LD_VX_VY:     OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x8000, instructions: ld_vx_vy,     pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "8xy0;LD Vx, Vy",                   };
#[rustfmt::skip] const OR_VX_VY:     OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x8001, instructions: or_vx_vy,     pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "8xy1;OR Vx, Vy",                   };
#[rustfmt::skip] const AND_VX_VY:    OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x8002, instructions: and_vx_vy,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "8xy2;AND Vx, Vy",                  };
#[rustfmt::skip] const XOR_VX_VY:    OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x8003, instructions: xor_vx_vy,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "8xy3;XOR Vx, Vy",                  };
#[rustfmt::skip] const ADD_VX_VY:    OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x8004, instructions: add_vx_vy,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "8xy4;ADD Vx, Vy",                  };
#[rustfmt::skip] const SUB_VX_VY:    OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x8005, instructions: sub_vx_vy,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "8xy5;SUB Vx, Vy",                  };
#[rustfmt::skip] const SHR_VX:       OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x8006, instructions: shr_vx,       pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "8xy6;SHR Vx",                      };
#[rustfmt::skip] const SUBN_VX_VY:   OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x8007, instructions: subn_vx_vy,   pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "8xy7;SUBN Vx, Vy",                 };
#[rustfmt::skip] const SHL_VX:       OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x800E, instructions: shl_vx,       pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "8xyE;SHL Vx",                      };
#[rustfmt::skip] const SNE_VX_VY:    OpcodeMatcher = OpcodeMatcher { bitmask: 0xF00F, match_value: 0x9000, instructions: sne_vx_vy,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "9xy0;SNE Vx, Vy ",                 };
#[rustfmt::skip] const LD_I_ADDR:    OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0xA000, instructions: ld_i_addr,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Annn;LD I, addr",                  };
#[rustfmt::skip] const JP_V0_ADDR:   OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0xB000, instructions: jp_v0_addr,   pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Bnnn;JP V0, addr",                 };
#[rustfmt::skip] const RND_VX_BYTE:  OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0xC000, instructions: rnd_vx_byte,  pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Cxkk;RND Vx, byte",                };
#[rustfmt::skip] const DRW_VX_VY_NB: OpcodeMatcher = OpcodeMatcher { bitmask: 0xF000, match_value: 0xD000, instructions: drw_vx_vy_nb, pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Dxyn;DRW Vx, Vy, nibble",          };
#[rustfmt::skip] const SKP_VX:       OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xE09E, instructions: skp_vx,       pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Ex9E;SKP Vx",                      };
#[rustfmt::skip] const SKNP_VX:      OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xE0A1, instructions: sknp_vx,      pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "ExA1;SKNP Vx",                     };
#[rustfmt::skip] const LD_VX_DTIMER: OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xF007, instructions: ld_vx_dt,     pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Fx07;LD Vx, DT",                   };
#[rustfmt::skip] const LD_VX_K:      OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xF00A, instructions: ld_vx_key,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Fx0A;LD Vx, K",                    };
#[rustfmt::skip] const LD_DTIMER_VX: OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xF015, instructions: ld_dt_vx,     pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Fx15;LD DT, Vx,",                  };
#[rustfmt::skip] const LD_STIMER_VX: OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xF018, instructions: ld_st_vx,     pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Fx18;LD ST, Vx",                   };
#[rustfmt::skip] const ADD_I_VX:     OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xF01E, instructions: add_i_vx,     pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Fx1E;ADD I, Vx",                   };
#[rustfmt::skip] const LD_F_VX:      OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xF029, instructions: ld_f_vx,      pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Fx29;LD I, Vx (hex sprite value)", };
#[rustfmt::skip] const LD_BCD_VX:    OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xF033, instructions: ld_bcd_vx,    pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Fx33;LD [I] (BDC of Vx), Vx",      };
#[rustfmt::skip] const LD_I_VX:      OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xF055, instructions: ld_i_vx,      pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Fx55;LD [I], V0-Vx",               };
#[rustfmt::skip] const LD_VX_I:      OpcodeMatcher = OpcodeMatcher { bitmask: 0xF0FF, match_value: 0xF065, instructions: ld_vx_i,      pre_ex_dump: dft_pre_ex_dump, post_ex_dump: dft_post_ex_dump, desc: "Fx65;LD V0-Vx, [I]",               };

// Constant list of opcodes matchers.
pub const OPCODES: [OpcodeMatcher; CHIP8_TOTAL_STANDARD_OPCODES] = [
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

/// Default pre instructions execution dump function.
fn dft_pre_ex_dump(ctx: &VMContext) {
    let mut pre_ex_dump = format!("{:#06X}: {:#06X} /", ctx.registers.get_pc(), ctx.opcode);
    let split = ctx.pattern.split(";").collect::<Vec<&str>>();
    let opcode_str = *split.get(0).unwrap();
    let desc = format!(" {} /", *split.get(1).unwrap());

    pre_ex_dump.push_str(&desc);

    if opcode_str.contains("x") {
        let vx_index = vx_index!(ctx);
        let vx_value = vx_value!(ctx);
        let vx = format!(" V{:X} = {:#04X}", vx_index, vx_value);
        pre_ex_dump.push_str(&vx);
    }

    if opcode_str.contains("y") {
        let vy_index = vy_index!(ctx);
        let vy_value = vy_value!(ctx);
        let vy = format!(", V{:X} = {:#04X},", vy_index, vy_value);
        pre_ex_dump.push_str(&vy);
    }

    if opcode_str.contains("nnn") {
        let address = nnn_value!(ctx);
        let vy = format!(" NNN = {:#05X}", address);
        pre_ex_dump.push_str(&vy);
    }

    if opcode_str.ends_with("n") && !opcode_str.ends_with("nnn") {
        let nbytes = nbytes_value!(ctx);
        let vy = format!(" N = {:#03X}", nbytes);
        pre_ex_dump.push_str(&vy);
    }

    if opcode_str.contains("kk") {
        let byte = kk_value!(ctx);
        let vy = format!(" KK = {:#04X}", byte);
        pre_ex_dump.push_str(&vy);
    }

    println!("{}", pre_ex_dump);
}

/// Default post instructions execution dump function.
fn dft_post_ex_dump(ctx: &VMContext) {
    ctx.registers.dump();
    ctx.stack.dump();
    ctx.memory.dump();
    ctx.screen.dump();
}

/// Instructions for opcode pattern 00E0. Clear the display.
fn sys(ctx: &mut VMContext) -> Result<Signal, VMError> {
    // This instruction is only used on the old computers on which Chip-8 was originally implemented.
    // It is ignored by modern interpreters.
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 00E0. Clear the display.
fn cls(ctx: &mut VMContext) -> Result<Signal, VMError> {
    ctx.screen.clear()?;
    ctx.registers.inc_pc()?;
    Ok(Signal::DrawScreen)
}

/// Instructions for opcode pattern 00EE. Return from subroutine.
fn ret(ctx: &mut VMContext) -> Result<Signal, VMError> {
    ctx.registers.dec_sp()?;
    ctx.registers
        .set_pc(ctx.stack.get_at(ctx.registers.get_sp())?);
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 1nnn. Jump to location nnn.
fn jp(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let jump_address = nnn_value!(ctx);
    ctx.registers.set_pc(jump_address);
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 2nnn. Call subroutine at address nnn.
fn call(ctx: &mut VMContext) -> Result<Signal, VMError> {
    ctx.registers.inc_pc()?;
    ctx.stack
        .set_at(ctx.registers.get_sp(), ctx.registers.get_pc())?;
    ctx.registers.inc_sp()?;
    let call_address = nnn_value!(ctx);
    ctx.registers.set_pc(call_address);
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 3xkk. Skip next instruction if Vx = kk.
fn se_vx_kk(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    let kk_value = kk_value!(ctx);
    if vx_value == kk_value {
        ctx.registers.inc_pc()?;
    }
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 4xkk. Skip next instruction if Vx != kk.
fn sne_vx_kk(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    let kk_value: u8 = kk_value!(ctx);
    if vx_value != kk_value {
        ctx.registers.inc_pc()?;
    }
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 5xy0. Skip next instruction if Vx = Vy.
fn se_vx_vy(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    let vy_value = vy_value!(ctx);
    if vx_value == vy_value {
        ctx.registers.inc_pc()?;
    }
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 6xkk. Set Vx = kk.
fn ld_vx_kk(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let kk_value = kk_value!(ctx);
    ctx.registers.set_v_register(vx_index, kk_value);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 7xkk. Set Vx = Vx + kk.
fn add_vx_kk(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let vx_value = vx_value!(ctx);
    let kk_value = kk_value!(ctx);
    ctx.registers
        .set_v_register(vx_index, vx_value.wrapping_add(kk_value));
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 8xy0. Set Vx = Vy.
fn ld_vx_vy(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let vy_value = vy_value!(ctx);
    ctx.registers.set_v_register(vx_index, vy_value);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 8xy1. Set Vx = Vx OR Vy.
fn or_vx_vy(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let vx_value = vx_value!(ctx);
    let vy_value = vy_value!(ctx);
    ctx.registers.set_v_register(vx_index, vx_value | vy_value);
    ctx.registers.unset_vf();
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 8xy2. Set Vx = Vx AND Vy.
fn and_vx_vy(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let vx_value = vx_value!(ctx);
    let vy_value = vy_value!(ctx);
    ctx.registers.set_v_register(vx_index, vx_value & vy_value);
    ctx.registers.unset_vf();
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 8xy3. Set Vx = Vx OR Vy.
fn xor_vx_vy(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let vx_value = vx_value!(ctx);
    let vy_value = vy_value!(ctx);
    ctx.registers.set_v_register(vx_index, vx_value ^ vy_value);
    ctx.registers.unset_vf();
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 8xy4. Set Vx = Vx + Vy, with carry.
fn add_vx_vy(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let vx_value = vx_value!(ctx) as u16;
    let vy_value = vy_value!(ctx) as u16;
    let addition = vx_value + vy_value;

    ctx.registers.unset_vf();
    if addition > std::u8::MAX as u16 {
        ctx.registers.set_vf();
    }
    ctx.registers.set_v_register(vx_index, addition as u8);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 8xy5. Set Vx = Vx - Vy, set VF = NOT borrow.
fn sub_vx_vy(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let vx_value = vx_value!(ctx);
    let vy_value = vy_value!(ctx);
    ctx.registers.unset_vf();
    if vx_value > vy_value {
        ctx.registers.set_vf();
    }
    ctx.registers
        .set_v_register(vx_index, vx_value.wrapping_sub(vy_value));
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 8xy6. Set Vx = Vx SHR 1, (shift right) set VF if truncation occurs.
fn shr_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    // let vy_index = ((ctx.opcode & 0x00F0) >> 4) as usize;
    // let vy_value = ctx.registers.get_v_register(vy_index);

    let vx_index = vx_index!(ctx);
    let vx_value = vx_value!(ctx);
    ctx.registers.set_v_register(vx_index, vx_value >> 1);

    let vx_lsb = vx_value & 0b0000_0001;
    ctx.registers.unset_vf();
    if vx_lsb == 1 {
        ctx.registers.set_vf();
    }
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 8xy7. Set Vx = Vy - Vx, with carry (if Vy > Vx).
fn subn_vx_vy(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let vx_value = vx_value!(ctx);
    let vy_value = vy_value!(ctx);
    ctx.registers.unset_vf();
    if vy_value > vx_value {
        ctx.registers.set_vf();
    }
    ctx.registers
        .set_v_register(vx_index, vy_value.wrapping_sub(vx_value));
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 8xyE. Set Vx = Vx SHL 1, (shift right) set VF if truncation occurs.
fn shl_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    // let vy_index = ((ctx.opcode & 0x00F0) >> 4) as usize;
    // let vy_value = ctx.registers.get_v_register(vy_index);

    let vx_index = vx_index!(ctx);
    let vx_value = vx_value!(ctx);
    ctx.registers.set_v_register(vx_index, vx_value << 1);

    let vx_msb = vx_value & 0b1000_0000;
    ctx.registers.unset_vf();
    if vx_msb > 0 {
        ctx.registers.set_vf();
    }
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern 9xy0. Skip next instruction if Vx != Vy.
fn sne_vx_vy(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    let vy_value = vy_value!(ctx);
    if vx_value != vy_value {
        ctx.registers.inc_pc()?;
    }
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Annn. Set I = nnn.
fn ld_i_addr(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let nnn_value = nnn_value!(ctx);
    ctx.registers.set_i(nnn_value);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Bnnn. Jump to location nnn + V0.
fn jp_v0_addr(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let jump_address = nnn_value!(ctx);
    let offset = ctx.registers.get_v_register(0) as u16;
    ctx.registers.set_pc(jump_address + offset);
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Cxkk. Set Vx = random byte AND kk.
fn rnd_vx_byte(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let kk_value = kk_value!(ctx);
    let rnd = rand::random::<u8>();
    ctx.registers.set_v_register(vx_index, rnd & kk_value);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Dxyn. Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
fn drw_vx_vy_nb(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    let vy_value = vy_value!(ctx);
    let nbytes = nbytes_value!(ctx);
    let offset = ctx.registers.get_i();
    ctx.registers.unset_vf();
    if ctx.screen.draw_sprite(
        vx_value as usize,
        vy_value as usize,
        offset as usize,
        ctx.memory,
        nbytes,
    )? {
        ctx.registers.set_vf();
    }
    ctx.registers.inc_pc()?;
    Ok(Signal::DrawScreen)
}

/// Instructions for opcode pattern Ex9E. Skip next instruction if key with the value of Vx is pressed.
fn skp_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    if ctx.keyboard.is_key_down(vx_value) {
        ctx.registers.inc_pc()?;
    }
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Ex9E. Skip next instruction if key with the value of Vx is not pressed.
fn sknp_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    if ctx.keyboard.is_key_up(vx_value) {
        ctx.registers.inc_pc()?;
    }
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Fx07. Set Vx = delay timer value.
fn ld_vx_dt(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let delay_timer = ctx.registers.get_dt();
    ctx.registers.set_v_register(vx_index, delay_timer);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

thread_local! {
    static WAIT_KEYUP: Cell<Option<u8>> = Cell::new(None);
}

/// Instructions for opcode pattern Fx0A. Wait for a key press, store the value fo the key in Vx.
fn ld_vx_key(ctx: &mut VMContext) -> Result<Signal, VMError> {
    WAIT_KEYUP.with(|f| {
        let mut signal = Ok(Signal::NoSignal);
        match f.get() {
            Some(key) => {
                if ctx.keyboard.is_key_up(key) {
                    let vx_index = vx_index!(ctx);
                    ctx.registers.set_v_register(vx_index, key);
                    ctx.registers.inc_pc()?;
                    f.set(None);
                }
            }
            None => {
                // check for key down through all keys
                for key in 0x0..=0xF as u8 {
                    if ctx.keyboard.is_key_down(key) {
                        signal = Ok(Signal::WaitKeyUp(key));
                        f.set(Some(key));
                    }
                }
            }
        }
        signal
    })
}

/// Instructions for opcode pattern Fx15. Set delay timer = Vx.
fn ld_dt_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    ctx.registers.set_dt(vx_value);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Fx18. Set sound timer = Vx.
fn ld_st_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    ctx.registers.set_st(vx_value);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Fx1E. Set I = I + Vx.
fn add_i_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    ctx.registers.set_i(ctx.registers.get_i() + vx_value as u16);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Fx29. Set I = location of sprite for digit Vx.
fn ld_f_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);
    let char_addr = vx_value * 5;
    ctx.registers.set_i(char_addr as u16);
    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Fx33. Store BCD representation of Vx in memory locations I, I+1, +2.
fn ld_bcd_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_value = vx_value!(ctx);

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
    let addr = ctx.registers.get_i() as usize;
    for (offset, unit) in [hund as u8, tens as u8, ones as u8].into_iter().enumerate() {
        ctx.memory.set(addr + offset, unit)?;
    }

    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Fx55. Store registers V0 through Vx in memory starting at location I.
fn ld_i_vx(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let base_addr = ctx.registers.get_i() as usize;

    for vx in 0..=vx_index {
        ctx.memory
            .set(base_addr + vx, ctx.registers.get_v_register(vx))?;
        ctx.registers.inc_i();
    }
    ctx.registers.inc_i();

    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}

/// Instructions for opcode pattern Fx65. Read registers V0 through Vx from memory starting at location I.
fn ld_vx_i(ctx: &mut VMContext) -> Result<Signal, VMError> {
    let vx_index = vx_index!(ctx);
    let base_addr = ctx.registers.get_i() as usize;

    for vx in 0..=vx_index {
        let value = ctx.memory.get(base_addr + vx)?;
        ctx.registers.set_v_register(vx, value);
        ctx.registers.inc_i();
    }
    ctx.registers.inc_i();

    ctx.registers.inc_pc()?;
    Ok(Signal::NoSignal)
}
