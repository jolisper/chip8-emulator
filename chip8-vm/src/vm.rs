use crate::{errors::VMError, memory::RAM, cpu::Registers, cpu::Stack, io::{Keyboard, Screen}};

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

    pub fn stack_push(&mut self, value: u16) -> Result<(), VMError> {
        self.registers.inc_sp()?;
        self.stack.push(self.registers.sp - 1, value)?;
        Ok(())
    }

    pub fn stack_pop(&mut self) -> Result<u16, VMError>{
        self.registers.dec_sp()?;
        let value = self.stack.pop(self.registers.sp)?;
        Ok(value)
    }

    pub fn keyboard_key_down(&mut self, key: i32) {
        self.keyboard.key_down(key)
    }

    pub fn keyboard_key_up(&mut self, key: i32) {
        self.keyboard.key_up(key)
    }

    pub fn keyboard_is_key_down(&mut self, key: u32) -> bool {
        self.keyboard.is_key_down(key)
    }

    pub fn keyboard_map_to_vkey(&mut self, key: i32) -> Result<usize, VMError> {
        self.keyboard.map_to_vkey(key)
    }

    pub fn screen_is_pixel_set(&mut self, x: usize, y :usize) -> Result<bool, VMError> {
        self.screen.is_pixel_set(x, y)
    }

    pub fn screen_set_pixel(&mut self, x: usize, y :usize) -> Result<(), VMError> {
        self.screen.set_pixel(x, y)
    }

    pub fn screen_draw_sprite(&mut self, x: usize, y :usize, offset: usize, sprite_bytes: u32) -> Result<bool, VMError> {
        let pixel_collision = self.screen.draw_sprite(x, y, offset, &self.memory, sprite_bytes as usize);
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

}
