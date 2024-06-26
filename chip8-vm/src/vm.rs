use crate::{
    config::CHIP8_PROGRAM_LOAD_ADDRESS,
    cpu::{Registers, Stack, VMContext, OPCODES},
    errors::VMError,
    io::{Keyboard, Screen},
    memory::RAM,
};

pub use crate::cpu::Signal;

#[derive(Default)]
pub struct VM {
    memory: RAM,
    registers: Registers,
    stack: Stack,
    keyboard: Keyboard,
    screen: Screen,
}

impl VM {
    pub fn new() -> Self {
        VM::default()
    }

    pub fn load_program(&mut self, buf: &[u8]) -> Result<(), VMError> {
        self.memory.load_program(buf)?;
        self.registers.set_pc(CHIP8_PROGRAM_LOAD_ADDRESS as u16);
        Ok(())
    }

    pub fn keyboard_key_down(&mut self, key: i32, keymap: &[(i32, usize)]) {
        self.keyboard.key_down(key, keymap)
    }

    pub fn keyboard_key_up(&mut self, key: i32, keymap: &[(i32, usize)]) {
        self.keyboard.key_up(key, keymap)
    }

    pub fn screen_is_pixel_set(&mut self, x: usize, y: usize) -> Result<bool, VMError> {
        self.screen.is_pixel_set(x, y)
    }

    pub fn registers_dt(&mut self) -> u8 {
        self.registers.dt
    }

    pub fn registers_st(&mut self) -> u8 {
        self.registers.st
    }

    pub fn exec_next_opcode(
        &mut self,
        debug_dump: bool,
        time_acc: &mut u32,
    ) -> Result<Signal, VMError> {
        let binary_opcode = self.memory.get_opcode(self.registers.get_pc() as usize)?;
        self.exec_opcode(binary_opcode, debug_dump, time_acc)
    }

    fn exec_opcode(
        &mut self,
        binary_opcode: u16,
        debug_dump: bool,
        time_acc: &mut u32,
    ) -> Result<Signal, VMError> {
        let time_per_delay = 60; // miliseconds
        for opcode_matcher in OPCODES {
            if opcode_matcher.check_matching(binary_opcode) {
                let mut ctx = self.build_vmcontext(binary_opcode, opcode_matcher.desc());

                if debug_dump {
                    opcode_matcher.pre_ex_dump()(&ctx);
                }

                // Execute Opcode instructions
                let signal = opcode_matcher.instructions()(&mut ctx)?;

                if debug_dump {
                    opcode_matcher.post_ex_dump()(&ctx);
                }

                // Update timers
                if *time_acc > time_per_delay {
                    if self.registers_dt() > 0 {
                        self.registers_dec_dt();
                    }
                    if self.registers_st() > 0 {
                        self.registers_dec_st();
                    }
                    *time_acc = 0;
                }
                return Ok(signal);
            }
        }
        Err(VMError::InvalidOpcode(binary_opcode))
    }

    fn build_vmcontext<'a>(&'a mut self, binary_opcode: u16, pattern: &'a str) -> VMContext<'a> {
        VMContext {
            opcode: binary_opcode,
            stack: &mut self.stack,
            memory: &mut self.memory,
            registers: &mut self.registers,
            keyboard: &self.keyboard,
            screen: &mut self.screen,
            pattern: pattern,
        }
    }

    fn registers_dec_dt(&mut self) {
        self.registers.dec_dt();
    }

    fn registers_dec_st(&mut self) {
        self.registers.dec_st();
    }
}

#[cfg(test)]
mod tests {
    use crate::VM;

    #[test]
    fn call_ret() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);

        // Call
        chip8.exec_opcode(0x2300, false, &mut 0).expect("Call");

        assert_eq!(chip8.registers.get_sp(), 0x001);
        assert_eq!(
            chip8
                .stack
                .get_at(chip8.registers.get_sp() - 1)
                .expect("Get top stack value"),
            0x202
        );
        assert_eq!(chip8.registers.get_pc(), 0x0300);

        // Return
        chip8
            .exec_opcode(0x00EE, false, &mut 0)
            .expect("Return from subroutine");
        assert_eq!(chip8.registers.get_pc(), 0x0202);
        assert_eq!(chip8.registers.get_sp(), 0x0000);
    }

    #[test]
    fn jp() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);

        chip8.exec_opcode(0x1300, false, &mut 0).expect("Jump");

        assert_eq!(chip8.registers.get_pc(), 0x0300);
    }

    #[test]
    fn se_vx_byte() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x001);

        chip8
            .exec_opcode(0x3001, false, &mut 0)
            .expect("Skip next instruction");

        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn se_vx_byte_nojp() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x001);

        chip8
            .exec_opcode(0x3002, false, &mut 0)
            .expect("not skip next instruction");

        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn se_vx_vy() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x001);
        chip8.registers.set_v_register(1, 0x001);

        chip8
            .exec_opcode(0x5010, false, &mut 0)
            .expect("Skip next instruction if Vx = Vy");

        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn se_vx_vy_nojp() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x000);
        chip8.registers.set_v_register(1, 0x001);

        chip8
            .exec_opcode(0x5010, false, &mut 0)
            .expect("Not skip next instruction if Vx = Vy");

        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn sne_vx_byte() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x01);

        chip8.exec_opcode(0x4002, false, &mut 0).expect("Jump");

        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn sne_vx_byte_nojp() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x01);

        chip8.exec_opcode(0x4001, false, &mut 0).expect("No jump");

        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn add_vx_byte() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8
            .exec_opcode(0x60fe, false, &mut 0)
            .expect("Set V0 to 255");
        chip8
            .exec_opcode(0x7001, false, &mut 0)
            .expect("Set V0 = V0 + KK");

        assert_eq!(chip8.registers.v_0, 255);
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn add_vx_vy_with_carry() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 200);
        chip8.registers.set_v_register(1, 60);
        chip8
            .exec_opcode(0x8014, false, &mut 0)
            .expect("Set V0 = V0 + V1, with carry");

        assert_eq!(chip8.registers.v_0, 4);
        assert_eq!(chip8.registers.v_f, 1);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn add_vx_vy_not_carry() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 200);
        chip8.registers.set_v_register(1, 50);
        chip8
            .exec_opcode(0x8014, false, &mut 0)
            .expect("Set V0 = V0 + V1, not carry");

        assert_eq!(chip8.registers.v_0, 250);
        assert_eq!(chip8.registers.v_f, 0);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn sub_vx_vy_not_borrow() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 255);
        chip8.registers.set_v_register(1, 254);
        chip8
            .exec_opcode(0x8015, false, &mut 0)
            .expect("Set V0 = V0 - V1, with carry");

        assert_eq!(chip8.registers.v_0, 1);
        assert_eq!(chip8.registers.v_f, 1);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn sub_vx_vy_with_borrow() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 254);
        chip8.registers.set_v_register(1, 255);
        chip8
            .exec_opcode(0x8015, false, &mut 0)
            .expect("Set V0 = V0 - V1, not carry");

        assert_eq!(chip8.registers.v_0, 255);
        assert_eq!(chip8.registers.v_f, 0);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn shr_vx_with_carry() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.unset_vf();
        chip8
            .exec_opcode(0x7005, false, &mut 0)
            .expect("Set V0 to 5");
        chip8.exec_opcode(0x8006, false, &mut 0).expect("Set carry");

        assert_eq!(chip8.registers.v_0, 2);
        assert_eq!(chip8.registers.v_f, 1);
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn shr_vx_not_carry() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_vf();
        chip8
            .exec_opcode(0x600a, false, &mut 0)
            .expect("Set V0 to 10");
        chip8
            .exec_opcode(0x8006, false, &mut 0)
            .expect("Set not carry");

        assert_eq!(chip8.registers.v_0, 5);
        assert_eq!(chip8.registers.v_f, 0);
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn subn_vx_vy_with_borrow() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.unset_vf();
        chip8
            .exec_opcode(0x60fe, false, &mut 0)
            .expect("Set V0 to 254");
        chip8
            .exec_opcode(0x71ff, false, &mut 0)
            .expect("Set V1 to 255");
        chip8
            .exec_opcode(0x8017, false, &mut 0)
            .expect("Set V0 = V1 - V0, with borrow");

        assert_eq!(chip8.registers.v_0, 1);
        assert_eq!(chip8.registers.v_f, 1);
        assert_eq!(chip8.registers.get_pc(), 0x0206);
    }

    #[test]
    fn subn_vx_vy_not_borrow() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_vf();
        chip8
            .exec_opcode(0x60ff, false, &mut 0)
            .expect("Set V0 to 255");
        chip8
            .exec_opcode(0x71fe, false, &mut 0)
            .expect("Set V1 to 254");
        chip8
            .exec_opcode(0x8017, false, &mut 0)
            .expect("Set V0 = V1 - V0, not borrow");

        assert_eq!(chip8.registers.v_0, 255);
        assert_eq!(chip8.registers.v_f, 0);
        assert_eq!(chip8.registers.get_pc(), 0x0206);
    }

    #[test]
    fn shl_vx_with_carry() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.unset_vf();
        chip8
            .exec_opcode(0x6080, false, &mut 0)
            .expect("Set V0 to 128");
        chip8.exec_opcode(0x800E, false, &mut 0).expect("Set carry");

        assert_eq!(chip8.registers.v_0, 0);
        assert_eq!(chip8.registers.v_f, 1);
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn shl_vx_not_carry() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_vf();
        chip8
            .exec_opcode(0x607f, false, &mut 0)
            .expect("Set V0 to 127");
        chip8
            .exec_opcode(0x800E, false, &mut 0)
            .expect("Set not carry");

        assert_eq!(chip8.registers.v_0, 254);
        assert_eq!(chip8.registers.v_f, 0);
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn sne_vx_vy() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8
            .exec_opcode(0x60ff, false, &mut 0)
            .expect("Set V0 to 255");
        chip8
            .exec_opcode(0x61ee, false, &mut 0)
            .expect("Set V1 t0 255");
        chip8
            .exec_opcode(0x9010, false, &mut 0)
            .expect("Skip next instruction");

        assert_eq!(chip8.registers.get_pc(), 0x0208);
    }

    #[test]
    fn sne_vx_vy_not_skip() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8
            .exec_opcode(0x60FF, false, &mut 0)
            .expect("Set V0 to 255");
        chip8
            .exec_opcode(0x61FF, false, &mut 0)
            .expect("Set V1 t0 255");
        chip8
            .exec_opcode(0x9010, false, &mut 0)
            .expect("Skip next instruction");

        assert_eq!(chip8.registers.get_pc(), 0x0206);
    }

    #[test]
    fn ld_i_addr() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8
            .exec_opcode(0xAFFF, false, &mut 0)
            .expect("Set I to FFF");

        assert_eq!(chip8.registers.get_i(), 0x0FFF);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn jp_v0_addr() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8
            .exec_opcode(0x6002, false, &mut 0)
            .expect("Set V0 to 002");
        chip8
            .exec_opcode(0xB300, false, &mut 0)
            .expect("Set PC to V0 + 002");

        assert_eq!(chip8.registers.get_pc(), 0x0302);
    }

    #[test]
    fn drw_vx_vy_nbytes() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);

        // No collision, yet
        assert_eq!(chip8.registers.get_v_register(0xF), 0);

        chip8
            .exec_opcode(0xA000, false, &mut 0)
            .expect("Set I to 00");
        chip8
            .exec_opcode(0x600A, false, &mut 0)
            .expect("Set V0 to 10");
        chip8
            .exec_opcode(0x610A, false, &mut 0)
            .expect("Set V1 to 10");
        chip8
            .exec_opcode(0xD015, false, &mut 0)
            .expect("Draw 5 bytes sprite");

        // Expect that '0' is printed in screen at (10, 10)
        assert_eq!(chip8.screen.is_pixel_set(10, 10).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(10, 11).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(10, 12).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(10, 13).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(10, 14).unwrap(), true);

        assert_eq!(chip8.screen.is_pixel_set(11, 10).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(11, 11).unwrap(), false);
        assert_eq!(chip8.screen.is_pixel_set(11, 12).unwrap(), false);
        assert_eq!(chip8.screen.is_pixel_set(11, 13).unwrap(), false);
        assert_eq!(chip8.screen.is_pixel_set(11, 14).unwrap(), true);

        assert_eq!(chip8.screen.is_pixel_set(12, 10).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(12, 11).unwrap(), false);
        assert_eq!(chip8.screen.is_pixel_set(12, 12).unwrap(), false);
        assert_eq!(chip8.screen.is_pixel_set(12, 13).unwrap(), false);
        assert_eq!(chip8.screen.is_pixel_set(12, 14).unwrap(), true);

        assert_eq!(chip8.screen.is_pixel_set(13, 10).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(13, 11).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(13, 12).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(13, 13).unwrap(), true);
        assert_eq!(chip8.screen.is_pixel_set(13, 14).unwrap(), true);

        assert_eq!(chip8.registers.get_pc(), 0x0208);

        chip8
            .exec_opcode(0x600D, false, &mut 0)
            .expect("Set V0 to 13");
        chip8
            .exec_opcode(0x610E, false, &mut 0)
            .expect("Set V1 to 14");
        chip8
            .exec_opcode(0xD015, false, &mut 0)
            .expect("Draw 5 bytes sprite");
        // Collision!
        assert_eq!(chip8.registers.get_v_register(0xF), 1);

        assert_eq!(chip8.registers.get_pc(), 0x020E);

        chip8
            .exec_opcode(0x600A, false, &mut 0)
            .expect("Set V0 to 10");
        chip8
            .exec_opcode(0x6112, false, &mut 0)
            .expect("Set V1 to 18");
        chip8
            .exec_opcode(0xD015, false, &mut 0)
            .expect("Draw 5 bytes sprite");
        // Collision!
        assert_eq!(chip8.registers.get_v_register(0xF), 1);

        assert_eq!(chip8.registers.get_pc(), 0x0214);

        chip8
            .exec_opcode(0x6010, false, &mut 0)
            .expect("Set V0 to 16");
        chip8
            .exec_opcode(0x610A, false, &mut 0)
            .expect("Set V1 to 10");
        chip8
            .exec_opcode(0xD015, false, &mut 0)
            .expect("Draw 5 bytes sprite");
        // Collision!
        assert_eq!(chip8.registers.get_v_register(0xF), 1);

        chip8
            .exec_opcode(0x6010, false, &mut 0)
            .expect("Set V0 to 16");
        chip8
            .exec_opcode(0x6112, false, &mut 0)
            .expect("Set V1 to 18");
        chip8
            .exec_opcode(0xD015, false, &mut 0)
            .expect("Draw 5 bytes sprite");
        // Collision!
        assert_eq!(chip8.registers.get_v_register(0xF), 1);

        chip8
            .exec_opcode(0x6014, false, &mut 0)
            .expect("Set V0 to 13");
        chip8
            .exec_opcode(0x610E, false, &mut 0)
            .expect("Set V1 to 14");
        chip8
            .exec_opcode(0xD015, false, &mut 0)
            .expect("Draw 5 bytes sprite");

        // Collision!
        assert_eq!(chip8.registers.get_v_register(0xF), 0);
    }

    static KEYMAP: &'static [(i32, usize)] = &[
        (49, 0x1),
        (50, 0x2),
        (51, 0x3),
        (52, 0xC),
        (113, 0x4),
        (119, 0x5),
        (101, 0x6),
        (114, 0xD),
        (97, 0x7),
        (115, 0x8),
        (100, 0x9),
        (102, 0xE),
        (122, 0xA),
        (120, 0x0),
        (99, 0xB),
        (118, 0xF),
    ];

    #[test]
    fn skp_vx_key_down() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.keyboard_key_down(97, KEYMAP); // User press 'A' key
        chip8
            .exec_opcode(0x6007, false, &mut 0)
            .expect("Set V0 to match A key");
        assert_eq!(chip8.registers.get_v_register(0), 0x7);
        chip8
            .exec_opcode(0xE09E, false, &mut 0)
            .expect("Skip next instruction");
        assert_eq!(chip8.registers.get_pc(), 0x0206);
    }

    #[test]
    fn skp_vx_key_up() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.keyboard_key_up(97, KEYMAP); // User release 'A' key
        chip8
            .exec_opcode(0x600A, false, &mut 0)
            .expect("Set V0 to match A key");
        chip8
            .exec_opcode(0xE09E, false, &mut 0)
            .expect("Not skip next instruction");
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn ld_vx_dt() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_dt(0x0A);
        chip8
            .exec_opcode(0xF007, false, &mut 0)
            .expect("Set V0 to delay timer value");
        assert_eq!(chip8.registers.get_v_register(0), 0x0A);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn ld_vx_k() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.keyboard_key_down(97, KEYMAP);
        chip8
            .exec_opcode(0xF00A, false, &mut 0)
            .expect("Wait to key up");
        chip8.keyboard_key_up(97, KEYMAP);
        chip8
            .exec_opcode(0xF00A, false, &mut 0)
            .expect("Set V0 to 0xA key");
        assert_eq!(chip8.registers.get_v_register(0), 0x7);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn ld_dt_vx() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 10);
        chip8
            .exec_opcode(0xF015, false, &mut 0)
            .expect("Set delay timer to Vx");
        assert_eq!(chip8.registers.get_dt(), 0x0A);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn ld_st_vx() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 10);
        chip8
            .exec_opcode(0xF018, false, &mut 0)
            .expect("Set sound timer to Vx");
        assert_eq!(chip8.registers.st, 0x0A);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn add_i_vx() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 10);
        chip8.registers.set_i(10);
        chip8
            .exec_opcode(0xF01E, false, &mut 0)
            .expect("Set I = I + V0 = 20");
        assert_eq!(chip8.registers.get_i(), 0x14);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn ld_f_vx() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x02);
        chip8
            .exec_opcode(0xF029, false, &mut 0)
            .expect("Set I = location of sprite for digit Vx");
        assert_eq!(chip8.registers.get_i(), 0x0A);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn ld_bcd_vx() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_i(0x200);
        chip8.registers.set_v_register(0, 245);
        chip8
            .exec_opcode(0xF033, false, &mut 0)
            .expect("Store BCD representation of Vx in memory locations I, I+1, and I+2");

        let base_addr = 0x200;
        assert_eq!(chip8.memory.get_ref(base_addr)[0], 0b00000010);
        assert_eq!(chip8.memory.get_ref(base_addr)[1], 0b00000100);
        assert_eq!(chip8.memory.get_ref(base_addr)[2], 0b00000101);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn ld_i_vx() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_i(0x200);

        chip8.registers.set_v_register(0x0, 0x000);
        chip8.registers.set_v_register(0x1, 0x001);
        chip8.registers.set_v_register(0x2, 0x002);
        chip8.registers.set_v_register(0x3, 0x003);
        chip8.registers.set_v_register(0x4, 0x004);
        chip8.registers.set_v_register(0x5, 0x005);
        chip8.registers.set_v_register(0x6, 0x006);
        chip8.registers.set_v_register(0x7, 0x007);
        chip8.registers.set_v_register(0x8, 0x008);
        chip8.registers.set_v_register(0x9, 0x009);
        chip8.registers.set_v_register(0xA, 0x00A);
        chip8.registers.set_v_register(0xB, 0x00B);
        chip8.registers.set_v_register(0xC, 0x00C);
        chip8.registers.set_v_register(0xD, 0x00D);
        chip8.registers.set_v_register(0xE, 0x00E);
        chip8.registers.set_v_register(0xF, 0x00F);

        chip8
            .exec_opcode(0xFF55, false, &mut 0)
            .expect("Store registers V0 through Vx in memory starting at location I.");

        let base_addr = 0x200 as usize;
        assert_eq!(chip8.memory.get_ref(base_addr)[0x0], 0x000);
        assert_eq!(chip8.memory.get_ref(base_addr)[0x1], 0x001);
        assert_eq!(chip8.memory.get_ref(base_addr)[0x2], 0x002);
        assert_eq!(chip8.memory.get_ref(base_addr)[0x3], 0x003);
        assert_eq!(chip8.memory.get_ref(base_addr)[0x4], 0x004);
        assert_eq!(chip8.memory.get_ref(base_addr)[0x5], 0x005);
        assert_eq!(chip8.memory.get_ref(base_addr)[0x6], 0x006);
        assert_eq!(chip8.memory.get_ref(base_addr)[0x7], 0x007);
        assert_eq!(chip8.memory.get_ref(base_addr)[0x8], 0x008);
        assert_eq!(chip8.memory.get_ref(base_addr)[0x9], 0x009);
        assert_eq!(chip8.memory.get_ref(base_addr)[0xA], 0x00A);
        assert_eq!(chip8.memory.get_ref(base_addr)[0xB], 0x00B);
        assert_eq!(chip8.memory.get_ref(base_addr)[0xC], 0x00C);
        assert_eq!(chip8.memory.get_ref(base_addr)[0xD], 0x00D);
        assert_eq!(chip8.memory.get_ref(base_addr)[0xE], 0x00E);
        assert_eq!(chip8.memory.get_ref(base_addr)[0xF], 0x00F);

        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn ld_vx_i() {
        let base_addr = 0x200 as usize;
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_i(base_addr as u16);

        chip8
            .memory
            .set(base_addr + 0x0, 0x000)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0x1, 0x001)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0x2, 0x002)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0x3, 0x003)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0x4, 0x004)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0x5, 0x005)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0x6, 0x006)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0x7, 0x007)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0x8, 0x008)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0x9, 0x009)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0xA, 0x00A)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0xB, 0x00B)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0xC, 0x00C)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0xD, 0x00D)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0xE, 0x00E)
            .expect("Store value");
        chip8
            .memory
            .set(base_addr + 0xF, 0x00F)
            .expect("Store value");

        chip8
            .exec_opcode(0xFF65, false, &mut 0)
            .expect("Store registers V0 through Vx in memory starting at location I.");

        assert_eq!(chip8.registers.get_v_register(0x0), 0x000);
        assert_eq!(chip8.registers.get_v_register(0x1), 0x001);
        assert_eq!(chip8.registers.get_v_register(0x2), 0x002);
        assert_eq!(chip8.registers.get_v_register(0x3), 0x003);
        assert_eq!(chip8.registers.get_v_register(0x4), 0x004);
        assert_eq!(chip8.registers.get_v_register(0x5), 0x005);
        assert_eq!(chip8.registers.get_v_register(0x6), 0x006);
        assert_eq!(chip8.registers.get_v_register(0x7), 0x007);
        assert_eq!(chip8.registers.get_v_register(0x8), 0x008);
        assert_eq!(chip8.registers.get_v_register(0x9), 0x009);
        assert_eq!(chip8.registers.get_v_register(0xA), 0x00A);
        assert_eq!(chip8.registers.get_v_register(0xB), 0x00B);
        assert_eq!(chip8.registers.get_v_register(0xC), 0x00C);
        assert_eq!(chip8.registers.get_v_register(0xD), 0x00D);
        assert_eq!(chip8.registers.get_v_register(0xE), 0x00E);
        assert_eq!(chip8.registers.get_v_register(0xF), 0x00F);

        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }
}
