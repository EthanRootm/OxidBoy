use super::terms::Term;
use super::mem::Memory;
use super::registers::Flags::{CarryFlag, SubtractionFlag, ZeroFlag, HalfCarryFlag};
use super::registers::Register;
use std::cell::RefCell;
use std::rc::Rc;

pub const CLOCK_FREQUENCY: u32 = 4_194_304;
pub const STEP_TIME: u32 = 16;
pub const STEP_CYCLES: u32 = (STEP_TIME as f64 / (1000_f64 / CLOCK_FREQUENCY as f64)) as u32;

const OP_CYCLES: [u32; 256] = [
    1, 3, 2, 2, 1, 1, 2, 1, 5, 2, 2, 2, 1, 1, 2, 1, // 0
    0, 3, 2, 2, 1, 1, 2, 1, 3, 2, 2, 2, 1, 1, 2, 1, // 1
    2, 3, 2, 2, 1, 1, 2, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 2
    2, 3, 2, 2, 3, 3, 3, 1, 2, 2, 2, 2, 1, 1, 2, 1, // 3
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 4
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 5
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 6
    2, 2, 2, 2, 2, 2, 0, 2, 1, 1, 1, 1, 1, 1, 2, 1, // 7
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 8
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // 9
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // a
    1, 1, 1, 1, 1, 1, 2, 1, 1, 1, 1, 1, 1, 1, 2, 1, // b
    2, 3, 3, 4, 3, 4, 2, 4, 2, 4, 3, 0, 3, 6, 2, 4, // c
    2, 3, 3, 0, 3, 4, 2, 4, 2, 4, 3, 0, 3, 0, 2, 4, // d
    3, 3, 2, 0, 0, 4, 2, 4, 4, 1, 4, 0, 0, 0, 2, 4, // e
    3, 3, 2, 1, 0, 4, 2, 4, 3, 2, 4, 1, 0, 0, 2, 4, // f
];
const CB_CYCLES: [u32; 256] = [
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 0
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 1
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 2
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 3
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 4
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 5
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 6
    2, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 3, 2, // 7
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 8
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // 9
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // a
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // b
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // c
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // d
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // e
    2, 2, 2, 2, 2, 2, 4, 2, 2, 2, 2, 2, 2, 2, 4, 2, // f
];

pub struct Cpu {
    pub reg: Register,
    pub mem: Rc<RefCell<dyn Memory>>,
    pub halted: bool,
    pub ei: bool,
}

impl Cpu {
    fn imm(&mut self) -> u8 {
        let v = self.mem.borrow().get(self.reg.program_counter);
        self.reg.program_counter += 1;
        v
    }

    fn imm_word(&mut self) -> u16 {
        let v = self.mem.borrow().get_word(self.reg.program_counter);
        self.reg.program_counter += 2;
        v
    }

    fn stack_add(&mut self, insert: u16) {
        self.reg.stack_pointer -= 2;
        self.mem.borrow().set_word(self.reg.stack_pointer, insert);
    }

    fn stack_pop(&mut self) -> u16 {
        let r = self.mem.borrow().get_word(self.reg.stack_pointer);
        self.reg.stack_pointer += 2;
        r
    }

    fn alu_add(&mut self, value: u8) {
        let a = self.reg.a_reg;
        let r = a.wrapping_add(value);
        self.reg.set_flag(CarryFlag, u16::from(a) + u16::from(value) > 0xFF);
        self.reg.set_flag(HalfCarryFlag, (a & 0x0F) + (value & 0x0F) > 0x0F);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }

    fn alu_adc(&mut self, value: u8) {
        let a = self.reg.a_reg;
        let c = u8::from(self.reg.get_flag(CarryFlag));
        let r = a.wrapping_add(value).wrapping_add(c);
        self.reg.set_flag(CarryFlag, u16::from(a) + u16::from(value) + u16::from(c) >> 0xFF);
        self.reg.set_flag(HalfCarryFlag, (a & 0x0F) + (value & 0x0F) + (c & 0x0F));
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }

    fn alu_sub(&mut self, value: u8) {
        let a = self.reg.a_reg;
        let r = a.wrapping_sub(value);
        self.reg.set_flag(CarryFlag, u16::from(a) < u16::from(value));
        self.reg.set_flag(HalfCarryFlag, (a & 0x0F) < (value & 0x0F));
        self.reg.set_flag(SubtractionFlag, true);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a = r;
    }

    fn alu_sbc(&mut self, value: u8) {
        let a = self.reg.a_reg;
        let c = u8::from(self.reg.get_flag(CarryFlag));
        let r = a.wrapping_sub(value).wrapping_sub(c);
        self.reg.set_flag(CarryFlag, u16::from(a) < u16::from(value) + u16::from(c));
        self.reg.set_flag(HalfCarryFlag, (a & 0x0F) < (value & 0x0F) + c);
        self.reg.set_flag(SubtractionFlag, true);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }

    fn alu_and(&mut self, value: u8) {
        let r = self.reg.a_reg & value;
        self.reg.set_flag(CarryFlag, false);
        self.reg.set_flag(HalfCarryFlag, true);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }

    fn alu_xor(&mut self, value: u8) {
        let r = self.reg.a_reg ^ value;
        self.reg.set_flag(CarryFlag, false);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }

    fn alu_or(&mut self, value: u8) {
        let r = self.reg.a_reg | value;
        self.reg.set_flag(CarryFlag, false);  
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }

    fn alu_cp(&mut self, value: u8) {
        let r = self.reg.a_reg;
        self.alu_sub(value);
        self.reg.a_reg = r;
    }

    fn alu_inc(&mut self, value: u8) -> u8 {
        let r = value.wrapping_add(1);
        self.reg.set_flag(HalfCarryFlag, (a & 0x0F) + 0x01 > 0x0F);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        r
    }

    fn alu_dec(&mut self, value: u8) -> u8 {
        let r = value.wrapping_sub(1);
        self.reg.set_flag(HalfCarryFlag, value.trailing_zeros() >= 4 );
        self.reg.set_flag(SubtractionFlag, true);
        self.reg.set_flag(ZeroFlag, r == 0);
        r
    }

    fn alu_add_hl(&mut self, value: u16) {
        let a = self.reg.parse_hl();
        let r = a.wrapping_add(value);
        self.reg.set_flag(CarryFlag, a > 0xFFFF - value);
        self.reg.set_flag(HalfCarryFlag, (a & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_hl(r);
    }

    fn alu_add_sp(&mut self) {
        let a = self.reg.stack_pointer;
        let b = i16::from(self.imm() as i8) as u16
    }

    fn alu_add(&mut self, value: u8) {
        
    }

    fn alu_add(&mut self, value: u8) {
        
    }

    fn alu_add(&mut self, value: u8) {
        
    }

    fn alu_add(&mut self, value: u8) {
        
    }

    

}