use crate::{
    config::CHIP8_PROGRAM_LOAD_ADDRESS,
    cpu::{Registers, Stack, OPCODES},
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

    pub fn memory_set(&mut self, index: usize, value: u8) -> Result<(), VMError> {
        self.memory.set(index, value)
    }

    pub fn memory_get(&self, index: usize) -> Result<u8, VMError> {
        self.memory.get(index)
    }

    pub fn memory_get_ref(&mut self, index: usize) -> Result<&[u8], VMError> {
        let memref = self.memory.get_ref(index);
        Ok(memref)
    }

    pub fn memory_get_opcode(&self) -> Result<u16, VMError> {
        self.memory.get_opcode(self.registers.get_pc() as usize)
    }

    pub fn stack_push(&mut self, value: u16) -> Result<(), VMError> {
        self.registers.inc_sp()?;
        self.stack.set_at(self.registers.get_sp() - 1, value)?;
        Ok(())
    }

    pub fn stack_pop(&mut self) -> Result<u16, VMError> {
        self.registers.dec_sp()?;
        let value = self.stack.get_at(self.registers.get_sp())?;
        Ok(value)
    }

    pub fn keyboard_key_down(&mut self, key: i32, keymap: &[(i32, usize)]) {
        self.keyboard.key_down(key, keymap)
    }

    pub fn keyboard_key_up(&mut self, key: i32, keymap: &[(i32, usize)]) {
        self.keyboard.key_up(key, keymap)
    }

    pub fn keyboard_is_key_down(&mut self, key: u8) -> bool {
        self.keyboard.is_key_down(key)
    }

    pub fn keyboard_map_to_vkey(
        &mut self,
        key: i32,
        keymap: &[(i32, usize)],
    ) -> Result<usize, VMError> {
        self.keyboard.map_to_vkey(key, keymap)
    }

    pub fn screen_is_pixel_set(&mut self, x: usize, y: usize) -> Result<bool, VMError> {
        self.screen.is_pixel_set(x, y)
    }

    pub fn screen_set_pixel(&mut self, x: usize, y: usize) -> Result<(), VMError> {
        self.screen.set_pixel(x, y)
    }

    pub fn screen_draw_sprite(
        &mut self,
        x: usize,
        y: usize,
        offset: usize,
        sprite_bytes: u32,
    ) -> Result<bool, VMError> {
        let pixel_collision =
            self.screen
                .draw_sprite(x, y, offset, &self.memory, sprite_bytes as usize);
        pixel_collision
    }

    pub fn registers_set_dt(&mut self, value: u8) {
        self.registers.dt = value;
    }

    pub fn registers_dt(&mut self) -> u8 {
        self.registers.dt
    }

    pub fn registers_dec_dt(&mut self) {
        self.registers.dec_dt();
    }

    pub fn registers_set_st(&mut self, value: u8) {
        self.registers.st = value;
    }

    pub fn registers_st(&mut self) -> u8 {
        self.registers.st
    }

    pub fn registers_dec_st(&mut self) {
        self.registers.dec_st();
    }

    pub fn registers_get_v(&self, index: usize) -> u8 {
        self.registers.get_v_register(index)
    }

    pub fn load_program(&mut self, buf: &[u8]) -> Result<(), VMError> {
        self.memory.load_program(buf)?;
        self.registers.set_pc(CHIP8_PROGRAM_LOAD_ADDRESS as u16);
        Ok(())
    }

    pub fn exec_next_opcode(&mut self, debug_dump: bool) -> Result<Signal, VMError> {
        let binary_opcode = self.memory.get_opcode(self.registers.get_pc() as usize)?;
        self.exec_opcode(binary_opcode, debug_dump)
    }

    pub fn exec_opcode(&mut self, binary_opcode: u16, debug_dump: bool) -> Result<Signal, VMError> {
        for opcode in OPCODES {
            if opcode.check(binary_opcode) {
                if debug_dump {
                    opcode.pre_ex_dump()(
                        opcode.pattern(),
                        binary_opcode,
                        &mut self.stack,
                        &mut self.memory,
                        &mut self.registers,
                        &self.keyboard,
                        &mut self.screen,
                    );
                }
                let signal = opcode.instructions()(
                    binary_opcode,
                    &mut self.stack,
                    &mut self.memory,
                    &mut self.registers,
                    &self.keyboard,
                    &mut self.screen,
                )?;
                if debug_dump {
                    opcode.post_ex_dump()(
                        opcode.pattern(),
                        binary_opcode,
                        &mut self.stack,
                        &mut self.memory,
                        &mut self.registers,
                        &self.keyboard,
                        &mut self.screen,
                    );
                }
                return Ok(signal);
            }
        }
        Err(VMError::InvalidOpcode(binary_opcode))
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
        chip8.exec_opcode(0x2300, false).expect("Call");

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
            .exec_opcode(0x00EE, false)
            .expect("Return from subroutine");
        assert_eq!(chip8.registers.get_pc(), 0x0202);
        assert_eq!(chip8.registers.get_sp(), 0x0000);
    }

    #[test]
    fn jp() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);

        chip8.exec_opcode(0x1300, false).expect("Jump");

        assert_eq!(chip8.registers.get_pc(), 0x0300);
    }

    #[test]
    fn se_vx_byte() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x001);

        chip8
            .exec_opcode(0x3001, false)
            .expect("Skip next instruction");

        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn se_vx_byte_nojp() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x001);

        chip8
            .exec_opcode(0x3002, false)
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
            .exec_opcode(0x5010, false)
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
            .exec_opcode(0x5010, false)
            .expect("Not skip next instruction if Vx = Vy");

        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn sne_vx_byte() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x01);

        chip8.exec_opcode(0x4002, false).expect("Jump");

        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn sne_vx_byte_nojp() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 0x01);

        chip8.exec_opcode(0x4001, false).expect("No jump");

        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn add_vx_byte() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.exec_opcode(0x60fe, false).expect("Set V0 to 255");
        chip8.exec_opcode(0x7001, false).expect("Set V0 = V0 + KK");

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
            .exec_opcode(0x8014, false)
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
            .exec_opcode(0x8014, false)
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
            .exec_opcode(0x8015, false)
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
            .exec_opcode(0x8015, false)
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
        chip8.exec_opcode(0x7005, false).expect("Set V0 to 5");
        chip8.exec_opcode(0x8006, false).expect("Set carry");

        assert_eq!(chip8.registers.v_0, 2);
        assert_eq!(chip8.registers.v_f, 1);
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn shr_vx_not_carry() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_vf();
        chip8.exec_opcode(0x600a, false).expect("Set V0 to 10");
        chip8.exec_opcode(0x8006, false).expect("Set not carry");

        assert_eq!(chip8.registers.v_0, 5);
        assert_eq!(chip8.registers.v_f, 0);
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn subn_vx_vy_with_borrow() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.unset_vf();
        chip8.exec_opcode(0x60fe, false).expect("Set V0 to 254");
        chip8.exec_opcode(0x71ff, false).expect("Set V1 to 255");
        chip8
            .exec_opcode(0x8017, false)
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
        chip8.exec_opcode(0x60ff, false).expect("Set V0 to 255");
        chip8.exec_opcode(0x71fe, false).expect("Set V1 to 254");
        chip8
            .exec_opcode(0x8017, false)
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
        chip8.exec_opcode(0x6080, false).expect("Set V0 to 128");
        chip8.exec_opcode(0x800E, false).expect("Set carry");

        assert_eq!(chip8.registers.v_0, 0);
        assert_eq!(chip8.registers.v_f, 1);
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn shl_vx_not_carry() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_vf();
        chip8.exec_opcode(0x607f, false).expect("Set V0 to 127");
        chip8.exec_opcode(0x800E, false).expect("Set not carry");

        assert_eq!(chip8.registers.v_0, 254);
        assert_eq!(chip8.registers.v_f, 0);
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn sne_vx_vy() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.exec_opcode(0x60ff, false).expect("Set V0 to 255");
        chip8.exec_opcode(0x61ee, false).expect("Set V1 t0 255");
        chip8
            .exec_opcode(0x9010, false)
            .expect("Skip next instruction");

        assert_eq!(chip8.registers.get_pc(), 0x0208);
    }

    #[test]
    fn sne_vx_vy_not_skip() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.exec_opcode(0x60FF, false).expect("Set V0 to 255");
        chip8.exec_opcode(0x61FF, false).expect("Set V1 t0 255");
        chip8
            .exec_opcode(0x9010, false)
            .expect("Skip next instruction");

        assert_eq!(chip8.registers.get_pc(), 0x0206);
    }

    #[test]
    fn ld_i_addr() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.exec_opcode(0xAFFF, false).expect("Set I to FFF");

        assert_eq!(chip8.registers.get_i(), 0x0FFF);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn jp_v0_addr() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.exec_opcode(0x6002, false).expect("Set V0 to 002");
        chip8
            .exec_opcode(0xB300, false)
            .expect("Set PC to V0 + 002");

        assert_eq!(chip8.registers.get_pc(), 0x0302);
    }

    #[test]
    fn drw_vx_vy_nbytes() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);

        // No collision, yet
        assert_eq!(chip8.registers.get_v_register(0xF), 0);

        chip8.exec_opcode(0xA000, false).expect("Set I to 00");
        chip8.exec_opcode(0x600A, false).expect("Set V0 to 10");
        chip8.exec_opcode(0x610A, false).expect("Set V1 to 10");
        chip8
            .exec_opcode(0xD015, false)
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

        chip8.exec_opcode(0x600D, false).expect("Set V0 to 13");
        chip8.exec_opcode(0x610E, false).expect("Set V1 to 14");
        chip8
            .exec_opcode(0xD015, false)
            .expect("Draw 5 bytes sprite");
        // Collision!
        assert_eq!(chip8.registers.get_v_register(0xF), 1);

        assert_eq!(chip8.registers.get_pc(), 0x020E);

        chip8.exec_opcode(0x600A, false).expect("Set V0 to 10");
        chip8.exec_opcode(0x6112, false).expect("Set V1 to 18");
        chip8
            .exec_opcode(0xD015, false)
            .expect("Draw 5 bytes sprite");
        // Collision!
        assert_eq!(chip8.registers.get_v_register(0xF), 1);

        assert_eq!(chip8.registers.get_pc(), 0x0214);

        chip8.exec_opcode(0x6010, false).expect("Set V0 to 16");
        chip8.exec_opcode(0x610A, false).expect("Set V1 to 10");
        chip8
            .exec_opcode(0xD015, false)
            .expect("Draw 5 bytes sprite");
        // Collision!
        assert_eq!(chip8.registers.get_v_register(0xF), 1);

        chip8.exec_opcode(0x6010, false).expect("Set V0 to 16");
        chip8.exec_opcode(0x6112, false).expect("Set V1 to 18");
        chip8
            .exec_opcode(0xD015, false)
            .expect("Draw 5 bytes sprite");
        // Collision!
        assert_eq!(chip8.registers.get_v_register(0xF), 1);

        chip8.exec_opcode(0x6014, false).expect("Set V0 to 13");
        chip8.exec_opcode(0x610E, false).expect("Set V1 to 14");
        chip8
            .exec_opcode(0xD015, false)
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
            .exec_opcode(0x6007, false)
            .expect("Set V0 to match A key");
        assert_eq!(chip8.registers.get_v_register(0), 0x7);
        chip8
            .exec_opcode(0xE09E, false)
            .expect("Skip next instruction");
        assert_eq!(chip8.registers.get_pc(), 0x0206);
    }

    #[test]
    fn skp_vx_key_up() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.keyboard_key_up(97, KEYMAP); // User release 'A' key
        chip8
            .exec_opcode(0x600A, false)
            .expect("Set V0 to match A key");
        chip8
            .exec_opcode(0xE09E, false)
            .expect("Not skip next instruction");
        assert_eq!(chip8.registers.get_pc(), 0x0204);
    }

    #[test]
    fn ld_vx_dt() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_dt(0x0A);
        chip8
            .exec_opcode(0xF007, false)
            .expect("Set V0 to delay timer value");
        assert_eq!(chip8.registers.get_v_register(0), 0x0A);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn ld_vx_k() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.keyboard_key_down(97, KEYMAP);
        chip8.exec_opcode(0xF00A, false).expect("Set V0 to 0xA key");
        assert_eq!(chip8.registers.get_v_register(0), 0x7);
        assert_eq!(chip8.registers.get_pc(), 0x0202);
    }

    #[test]
    fn ld_dt_vx() {
        let mut chip8: VM = VM::new();
        chip8.registers.set_pc(0x0200);
        chip8.registers.set_v_register(0, 10);
        chip8
            .exec_opcode(0xF015, false)
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
            .exec_opcode(0xF018, false)
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
            .exec_opcode(0xF01E, false)
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
            .exec_opcode(0xF029, false)
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
            .exec_opcode(0xF033, false)
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
            .exec_opcode(0xFF55, false)
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
            .exec_opcode(0xFF65, false)
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
