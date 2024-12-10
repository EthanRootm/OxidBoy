use super::terms::Term;
use super::mem::Memory;
use super::registers::Flags::{CarryFlag, SubtractionFlag, ZeroFlag, HalfCarryFlag};
use super::registers::Register;
use std::cell::RefCell;
use std::rc::Rc;
use std::{thread, time};

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
        self.mem.borrow_mut().set_word(self.reg.stack_pointer, insert);
    }

    fn stack_pop(&mut self) -> u16 {
        let r = self.mem.borrow().get_word(self.reg.stack_pointer);
        self.reg.stack_pointer += 2;
        r
    }
    ///Adds value to A
    fn alu_add(&mut self, value: u8) {
        let a = self.reg.a_reg;
        let r = a.wrapping_add(value);
        self.reg.set_flag(CarryFlag, u16::from(a) + u16::from(value) > 0xFF);
        self.reg.set_flag(HalfCarryFlag, (a & 0x0F) + (value & 0x0F) > 0x0F);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }
    ///Adds value + Carry flag to A
    fn alu_adc(&mut self, value: u8) {
        let a = self.reg.a_reg;
        let c = u8::from(self.reg.get_flag(CarryFlag));
        let r = a.wrapping_add(value).wrapping_add(c);
        self.reg.set_flag(CarryFlag, u16::from(a) + u16::from(value) + u16::from(c) > 0xFF);
        self.reg.set_flag(HalfCarryFlag, (a & 0x0F) + (value & 0x0F) + (c & 0x0F) > 0x0F);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }
    ///Subtract value from A
    fn alu_sub(&mut self, value: u8) {
        let a = self.reg.a_reg;
        let r = a.wrapping_sub(value);
        self.reg.set_flag(CarryFlag, u16::from(a) < u16::from(value));
        self.reg.set_flag(HalfCarryFlag, (a & 0x0F) < (value & 0x0F));
        self.reg.set_flag(SubtractionFlag, true);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }
    ///Subtract value + Carry flag from a
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
    ///Logical AND with value and A, stored in A
    fn alu_and(&mut self, value: u8) {
        let r = self.reg.a_reg & value;
        self.reg.set_flag(CarryFlag, false);
        self.reg.set_flag(HalfCarryFlag, true);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }
    ///Logical Exclusive OR with value and A, stored in A
    fn alu_xor(&mut self, value: u8) {
        let r = self.reg.a_reg ^ value;
        self.reg.set_flag(CarryFlag, false);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }
    ///Logical OR with value and A, stored in A
    fn alu_or(&mut self, value: u8) {
        let r = self.reg.a_reg | value;
        self.reg.set_flag(CarryFlag, false);  
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        self.reg.a_reg = r;
    }
    ///Compare A with value
    fn alu_cp(&mut self, value: u8) {
        let r = self.reg.a_reg;
        self.alu_sub(value);
        self.reg.a_reg = r;
    }
    ///Incliment register value
    fn alu_inc(&mut self, value: u8) -> u8 {
        let r = value.wrapping_add(1);
        self.reg.set_flag(HalfCarryFlag, (value & 0x0F) + 0x01 > 0x0F);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        r
    }
    ///Decrement register value
    fn alu_dec(&mut self, value: u8) -> u8 {
        let r = value.wrapping_sub(1);
        self.reg.set_flag(HalfCarryFlag, value.trailing_zeros() >= 4 );
        self.reg.set_flag(SubtractionFlag, true);
        self.reg.set_flag(ZeroFlag, r == 0);
        r
    }
    ///Add value to HL
    fn alu_add_hl(&mut self, value: u16) {
        let a = self.reg.parse_hl();
        let r = a.wrapping_add(value);
        self.reg.set_flag(CarryFlag, a > 0xFFFF - value);
        self.reg.set_flag(HalfCarryFlag, (a & 0x0FFF) + (value & 0x0FFF) > 0x0FFF);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_hl(r);
    }
    ///Add one byte signed immediate value to Stack Pointer
    fn alu_add_sp(&mut self) {
        let a = self.reg.stack_pointer;
        let b = i16::from(self.imm() as i8) as u16;
        self.reg.set_flag(CarryFlag, (a & 0x00FF) + (b & 0x00FF) > 0x00FF);
        self.reg.set_flag(HalfCarryFlag, (a & 0x000F) + (b & 0x000F) > 0x00FF);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, false);
        self.reg.stack_pointer = a.wrapping_add(b);
    }
    ///Swaps the upper and lower nibbles of value
    fn alu_swap(&mut self, value: u8) -> u8 {
        self.reg.set_flag(CarryFlag, false);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, value == 0x00);
        (value >> 4) | (value << 4)
    }
    ///Decimal Adjust register A, sets register A to represent Binary Coded Decimal
    fn alu_daa(&mut self) {
        let mut a = self.reg.a_reg;
        let mut adjust = if self.reg.get_flag(CarryFlag) { 0x60 } else {0x00};
        if self.reg.get_flag(HalfCarryFlag) {
            adjust |= 0x06;
        };
        if !self.reg.get_flag(SubtractionFlag) {
            if a & 0x0F > 0x09 {
                adjust |= 0x06;
            };
            if a > 0x99 {
                adjust |= 0x60;
            };
            a = a.wrapping_add(adjust);
        } else {
            a = a.wrapping_sub(adjust);
        }
        self.reg.set_flag(CarryFlag, adjust >= 0x60);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(ZeroFlag, a == 0x00);
        self.reg.a_reg = a;
    }
    ///Flips all bits of register A
    fn alu_cpl(&mut self) {
        self.reg.a_reg = !self.reg.a_reg;
        self.reg.set_flag(HalfCarryFlag, true);
        self.reg.set_flag(SubtractionFlag, true);
    }
    ///Compliment Carry Flag
    fn alu_ccf(&mut self) {
        let v = !self.reg.get_flag(CarryFlag);
        self.reg.set_flag(CarryFlag, v);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
    }
    ///Set Carry Flag
    fn alu_scf(&mut self) {
        self.reg.set_flag(CarryFlag, true);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
    }
    ///Rotate Value left, set old bit 7 to Carry flag
    fn alu_rlc(&mut self, value: u8) -> u8 {
        let c = (value & 0x80) >> 7 == 0x01;
        let r = (value << 1) | u8::from(c);
        self.reg.set_flag(CarryFlag, c);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        r
    }
    ///Rotate Value left
    fn alu_rl(&mut self, value: u8) -> u8 {
        let c = (value & 0x80) >> 7 == 0x01;
        let r = (value << 1) + u8::from(self.reg.get_flag(CarryFlag));
        self.reg.set_flag(CarryFlag, c);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        r
    }
    ///Rotate Value right, set old bit 0 to Carry flag
    fn alu_rrc(&mut self, value: u8) -> u8{
        let c = value & 0x01 == 0x01;
        let r = if c { 0x80 | (value >> 1)} else { value >> 1};
        self.reg.set_flag(CarryFlag, c);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        r
    }
    ///Rotate Value right
    fn alu_rr(&mut self, value: u8) -> u8 {
        let c = value & 0x01 == 0x01;
        let r = if self.reg.get_flag(CarryFlag) { 0x80 | (value >> 1)} else { value >> 1};
        self.reg.set_flag(CarryFlag, c);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        r
    }
    ///Shift value left into Carry
    fn alu_sla(&mut self, value: u8) -> u8 {
        let c = (value & 0x80) >> 7 == 0x01;
        let r = value << 1;
        self.reg.set_flag(CarryFlag, c);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        r
    }
    ///Shift value right into Carry
    fn alu_sra(&mut self, value: u8) -> u8 {
        let c = value & 0x01 == 0x01;
        let r = (value >> 1) | (value & 0x80);
        self.reg.set_flag(CarryFlag, c);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        r
    }
    ///Shift value right into Carry, setting MeroFlaSB to 0
    fn alu_srl(&mut self, value: u8) -> u8 {
        let c = value & 0x01 == 0x01;
        let r = value >> 1;
        self.reg.set_flag(CarryFlag, c);
        self.reg.set_flag(HalfCarryFlag, false);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r == 0x00);
        r
    }
    ///Test bit in register value
    fn alu_bit(&mut self, value: u8, bit: u8) {
        let r = value & (1 << bit) == 0x00;
        self.reg.set_flag(HalfCarryFlag, true);
        self.reg.set_flag(SubtractionFlag, false);
        self.reg.set_flag(ZeroFlag, r);
    }
    ///Set bit in register value and return
    fn alu_set(&mut self, value: u8, bit: u8) -> u8 {
        value | (1 << bit)
    }
    ///Reset bit in register value
    fn alu_res(&mut self, value: u8, bit: u8) -> u8 {
        value & !(1 << bit)
    }
    ///Add value to current address and jump to it
    fn alu_jr(&mut self, value: u8) {
        let value = value as i8;
        self.reg.program_counter = ((u32::from(self.reg.program_counter) as i32) + i32::from(value)) as u16;
    }
}

impl Cpu {
    pub fn power_up(term: Term, mem: Rc<RefCell<dyn Memory>>) -> Self {
        Self { reg: Register::power_up(term), mem, halted: false, ei: true }
    }
    fn hi(&mut self) -> u32 {
        if !self.halted && !self.ei {
            return 0;
        }
        let intf = self.mem.borrow().get(0xFF0F);
        let inte = self.mem.borrow().get(0xFFFF);
        let ii = intf & inte;
        if ii == 0x00 {
            return 0;
        }
        self.halted = false;
        if !self.ei {
            return 0;
        }
        self.ei = false;

        let n = ii.trailing_zeros();
        let intf = intf & !(1 << n);
        self.mem.borrow_mut().set(0xFF0F, intf);

        self.stack_add(self.reg.program_counter);
        self.reg.program_counter = 0x0040 | ((n as u16) << 3);
        4
    }
    fn ex(&mut self) -> u32 {
        let opcode = self.imm();
        let mut cbcode: u8 = 0;
        match opcode {
            // LD r8, d8
            0x06 => self.reg.b_reg = self.imm(),
            0x0e => self.reg.c_reg = self.imm(),
            0x16 => self.reg.d_reg = self.imm(),
            0x1e => self.reg.e_reg = self.imm(),
            0x26 => self.reg.h_reg = self.imm(),
            0x2e => self.reg.l_reg = self.imm(),
            0x36 => {
                let a = self.reg.parse_hl();
                let v = self.imm();
                self.mem.borrow_mut().set(a, v);
            }
            0x3e => self.reg.a_reg = self.imm(),

            // LD (r16), A
            0x02 => self.mem.borrow_mut().set(self.reg.parse_bc(), self.reg.a_reg),
            0x12 => self.mem.borrow_mut().set(self.reg.parse_de(), self.reg.a_reg),

            // LD A, (r16)
            0x0a => self.reg.a_reg = self.mem.borrow().get(self.reg.parse_bc()),
            0x1a => self.reg.a_reg = self.mem.borrow().get(self.reg.parse_de()),

            // LD (HL+), A
            0x22 => {
                let a = self.reg.parse_hl();
                self.mem.borrow_mut().set(a, self.reg.a_reg);
                self.reg.set_hl(a + 1);
            }
            // LD (HL-), A
            0x32 => {
                let a = self.reg.parse_hl();
                self.mem.borrow_mut().set(a, self.reg.a_reg);
                self.reg.set_hl(a - 1);
            }
            // LD A, (HL+)
            0x2a => {
                let v = self.reg.parse_hl();
                self.reg.a_reg = self.mem.borrow().get(v);
                self.reg.set_hl(v + 1);
            }
            // LD A, (HL-)
            0x3a => {
                let v = self.reg.parse_hl();
                self.reg.a_reg = self.mem.borrow().get(v);
                self.reg.set_hl(v - 1);
            }

            // LD r8, r8
            0x40 => {}
            0x41 => self.reg.b_reg = self.reg.c_reg,
            0x42 => self.reg.b_reg = self.reg.d_reg,
            0x43 => self.reg.b_reg = self.reg.e_reg,
            0x44 => self.reg.b_reg = self.reg.h_reg,
            0x45 => self.reg.b_reg = self.reg.l_reg,
            0x46 => self.reg.b_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x47 => self.reg.b_reg = self.reg.a_reg,
            0x48 => self.reg.c_reg = self.reg.b_reg,
            0x49 => {}
            0x4a => self.reg.c_reg = self.reg.d_reg,
            0x4b => self.reg.c_reg = self.reg.e_reg,
            0x4c => self.reg.c_reg = self.reg.h_reg,
            0x4d => self.reg.c_reg = self.reg.l_reg,
            0x4e => self.reg.c_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x4f => self.reg.c_reg = self.reg.a_reg,
            0x50 => self.reg.d_reg = self.reg.b_reg,
            0x51 => self.reg.d_reg = self.reg.c_reg,
            0x52 => {}
            0x53 => self.reg.d_reg = self.reg.e_reg,
            0x54 => self.reg.d_reg = self.reg.h_reg,
            0x55 => self.reg.d_reg = self.reg.l_reg,
            0x56 => self.reg.d_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x57 => self.reg.d_reg = self.reg.a_reg,
            0x58 => self.reg.e_reg = self.reg.b_reg,
            0x59 => self.reg.e_reg = self.reg.c_reg,
            0x5a => self.reg.e_reg = self.reg.d_reg,
            0x5b => {}
            0x5c => self.reg.e_reg = self.reg.h_reg,
            0x5d => self.reg.e_reg = self.reg.l_reg,
            0x5e => self.reg.e_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x5f => self.reg.e_reg = self.reg.a_reg,
            0x60 => self.reg.h_reg = self.reg.b_reg,
            0x61 => self.reg.h_reg = self.reg.c_reg,
            0x62 => self.reg.h_reg = self.reg.d_reg,
            0x63 => self.reg.h_reg = self.reg.e_reg,
            0x64 => {}
            0x65 => self.reg.h_reg = self.reg.l_reg,
            0x66 => self.reg.h_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x67 => self.reg.h_reg = self.reg.a_reg,
            0x68 => self.reg.l_reg = self.reg.b_reg,
            0x69 => self.reg.l_reg = self.reg.c_reg,
            0x6a => self.reg.l_reg = self.reg.d_reg,
            0x6b => self.reg.l_reg = self.reg.e_reg,
            0x6c => self.reg.l_reg = self.reg.h_reg,
            0x6d => {}
            0x6e => self.reg.l_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x6f => self.reg.l_reg = self.reg.a_reg,
            0x70 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.b_reg),
            0x71 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.c_reg),
            0x72 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.d_reg),
            0x73 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.e_reg),
            0x74 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.h_reg),
            0x75 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.l_reg),
            0x77 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.a_reg),
            0x78 => self.reg.a_reg = self.reg.b_reg,
            0x79 => self.reg.a_reg = self.reg.c_reg,
            0x7a => self.reg.a_reg = self.reg.d_reg,
            0x7b => self.reg.a_reg = self.reg.e_reg,
            0x7c => self.reg.a_reg = self.reg.h_reg,
            0x7d => self.reg.a_reg = self.reg.l_reg,
            0x7e => self.reg.a_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x7f => {}

            // LDH (a8), A
            0xe0 => {
                let a = 0xff00 | u16::from(self.imm());
                self.mem.borrow_mut().set(a, self.reg.a_reg);
            }
            // LDH A, (a8)
            0xf0 => {
                let a = 0xff00 | u16::from(self.imm());
                self.reg.a_reg = self.mem.borrow().get(a);
            }

            // LD (C), A
            0xe2 => self.mem.borrow_mut().set(0xff00 | u16::from(self.reg.c_reg), self.reg.a_reg),
            // LD A, (C)
            0xf2 => self.reg.a_reg = self.mem.borrow().get(0xff00 | u16::from(self.reg.c_reg)),

            // LD (a16), A
            0xea => {
                let a = self.imm_word();
                self.mem.borrow_mut().set(a, self.reg.a_reg);
            }
            // LD A, (a16)
            0xfa => {
                let a = self.imm_word();
                self.reg.a_reg = self.mem.borrow().get(a);
            }

            // LD r16, d16
            0x01 | 0x11 | 0x21 | 0x31 => {
                let v = self.imm_word();
                match opcode {
                    0x01 => self.reg.set_bc(v),
                    0x11 => self.reg.set_de(v),
                    0x21 => self.reg.set_hl(v),
                    0x31 => self.reg.stack_pointer = v,
                    _ => {}
                }
            }

            // LD SP, HL
            0xf9 => self.reg.stack_pointer = self.reg.parse_hl(),
            // LD SP, d8
            0xf8 => {
                let a = self.reg.stack_pointer;
                let b = i16::from(self.imm() as i8) as u16;
                self.reg.set_flag(CarryFlag, (a & 0x00ff) + (b & 0x00ff) > 0x00ff);
                self.reg.set_flag(HalfCarryFlag, (a & 0x000f) + (b & 0x000f) > 0x000f);
                self.reg.set_flag(SubtractionFlag, false);
                self.reg.set_flag(ZeroFlag, false);
                self.reg.set_hl(a.wrapping_add(b));
            }
            // LD (d16), SP
            0x08 => {
                let a = self.imm_word();
                self.mem.borrow_mut().set_word(a, self.reg.stack_pointer);
            }

            // PUSH
            0xc5 => self.stack_add(self.reg.parse_bc()),
            0xd5 => self.stack_add(self.reg.parse_de()),
            0xe5 => self.stack_add(self.reg.parse_hl()),
            0xf5 => self.stack_add(self.reg.parse_af()),

            // POP
            0xc1 | 0xf1 | 0xd1 | 0xe1 => {
                let v = self.stack_pop();
                match opcode {
                    0xc1 => self.reg.set_bc(v),
                    0xd1 => self.reg.set_de(v),
                    0xe1 => self.reg.set_hl(v),
                    0xf1 => self.reg.set_af(v),
                    _ => {}
                }
            }

            // ADD A, r8/d8
            0x80 => self.alu_add(self.reg.b_reg),
            0x81 => self.alu_add(self.reg.c_reg),
            0x82 => self.alu_add(self.reg.d_reg),
            0x83 => self.alu_add(self.reg.e_reg),
            0x84 => self.alu_add(self.reg.h_reg),
            0x85 => self.alu_add(self.reg.l_reg),
            0x86 => {
                let v = self.mem.borrow().get(self.reg.parse_hl());
                self.alu_add(v);
            }
            0x87 => self.alu_add(self.reg.a_reg),
            0xc6 => {
                let v = self.imm();
                self.alu_add(v);
            }

            // ADC A, r8/d8
            0x88 => self.alu_adc(self.reg.b_reg),
            0x89 => self.alu_adc(self.reg.c_reg),
            0x8a => self.alu_adc(self.reg.d_reg),
            0x8b => self.alu_adc(self.reg.e_reg),
            0x8c => self.alu_adc(self.reg.h_reg),
            0x8d => self.alu_adc(self.reg.l_reg),
            0x8e => {
                let a = self.mem.borrow().get(self.reg.parse_hl());
                self.alu_adc(a);
            }
            0x8f => self.alu_adc(self.reg.a_reg),
            0xce => {
                let v = self.imm();
                self.alu_adc(v);
            }

            // SUB A, r8/d8
            0x90 => self.alu_sub(self.reg.b_reg),
            0x91 => self.alu_sub(self.reg.c_reg),
            0x92 => self.alu_sub(self.reg.d_reg),
            0x93 => self.alu_sub(self.reg.e_reg),
            0x94 => self.alu_sub(self.reg.h_reg),
            0x95 => self.alu_sub(self.reg.l_reg),
            0x96 => {
                let a = self.mem.borrow().get(self.reg.parse_hl());
                self.alu_sub(a);
            }
            0x97 => self.alu_sub(self.reg.a_reg),
            0xd6 => {
                let v = self.imm();
                self.alu_sub(v);
            }

            // SBC A, r8/d8
            0x98 => self.alu_sbc(self.reg.b_reg),
            0x99 => self.alu_sbc(self.reg.c_reg),
            0x9a => self.alu_sbc(self.reg.d_reg),
            0x9b => self.alu_sbc(self.reg.e_reg),
            0x9c => self.alu_sbc(self.reg.h_reg),
            0x9d => self.alu_sbc(self.reg.l_reg),
            0x9e => {
                let a = self.mem.borrow().get(self.reg.parse_hl());
                self.alu_sbc(a);
            }
            0x9f => self.alu_sbc(self.reg.a_reg),
            0xde => {
                let v = self.imm();
                self.alu_sbc(v);
            }

            // AND A, r8/d8
            0xa0 => self.alu_and(self.reg.b_reg),
            0xa1 => self.alu_and(self.reg.c_reg),
            0xa2 => self.alu_and(self.reg.d_reg),
            0xa3 => self.alu_and(self.reg.e_reg),
            0xa4 => self.alu_and(self.reg.h_reg),
            0xa5 => self.alu_and(self.reg.l_reg),
            0xa6 => {
                let a = self.mem.borrow().get(self.reg.parse_hl());
                self.alu_and(a);
            }
            0xa7 => self.alu_and(self.reg.a_reg),
            0xe6 => {
                let v = self.imm();
                self.alu_and(v);
            }

            // OR A, r8/d8
            0xb0 => self.alu_or(self.reg.b_reg),
            0xb1 => self.alu_or(self.reg.c_reg),
            0xb2 => self.alu_or(self.reg.d_reg),
            0xb3 => self.alu_or(self.reg.e_reg),
            0xb4 => self.alu_or(self.reg.h_reg),
            0xb5 => self.alu_or(self.reg.l_reg),
            0xb6 => {
                let a = self.mem.borrow().get(self.reg.parse_hl());
                self.alu_or(a);
            }
            0xb7 => self.alu_or(self.reg.a_reg),
            0xf6 => {
                let v = self.imm();
                self.alu_or(v);
            }

            // XOR A, r8/d8
            0xa8 => self.alu_xor(self.reg.b_reg),
            0xa9 => self.alu_xor(self.reg.c_reg),
            0xaa => self.alu_xor(self.reg.d_reg),
            0xab => self.alu_xor(self.reg.e_reg),
            0xac => self.alu_xor(self.reg.h_reg),
            0xad => self.alu_xor(self.reg.l_reg),
            0xae => {
                let a = self.mem.borrow().get(self.reg.parse_hl());
                self.alu_xor(a);
            }
            0xaf => self.alu_xor(self.reg.a_reg),
            0xee => {
                let v = self.imm();
                self.alu_xor(v);
            }

            // CP A, r8/d8
            0xb8 => self.alu_cp(self.reg.b_reg),
            0xb9 => self.alu_cp(self.reg.c_reg),
            0xba => self.alu_cp(self.reg.d_reg),
            0xbb => self.alu_cp(self.reg.e_reg),
            0xbc => self.alu_cp(self.reg.h_reg),
            0xbd => self.alu_cp(self.reg.l_reg),
            0xbe => {
                let a = self.mem.borrow().get(self.reg.parse_hl());
                self.alu_cp(a);
            }
            0xbf => self.alu_cp(self.reg.a_reg),
            0xfe => {
                let v = self.imm();
                self.alu_cp(v);
            }

            // INC r8
            0x04 => self.reg.b_reg = self.alu_inc(self.reg.b_reg),
            0x0c => self.reg.c_reg = self.alu_inc(self.reg.c_reg),
            0x14 => self.reg.d_reg = self.alu_inc(self.reg.d_reg),
            0x1c => self.reg.e_reg = self.alu_inc(self.reg.e_reg),
            0x24 => self.reg.h_reg = self.alu_inc(self.reg.h_reg),
            0x2c => self.reg.l_reg = self.alu_inc(self.reg.l_reg),
            0x34 => {
                let a = self.reg.parse_hl();
                let v = self.mem.borrow().get(a);
                let h = self.alu_inc(v);
                self.mem.borrow_mut().set(a, h);
            }
            0x3c => self.reg.a_reg = self.alu_inc(self.reg.a_reg),

            // DEC r8
            0x05 => self.reg.b_reg = self.alu_dec(self.reg.b_reg),
            0x0d => self.reg.c_reg = self.alu_dec(self.reg.c_reg),
            0x15 => self.reg.d_reg = self.alu_dec(self.reg.d_reg),
            0x1d => self.reg.e_reg = self.alu_dec(self.reg.e_reg),
            0x25 => self.reg.h_reg = self.alu_dec(self.reg.h_reg),
            0x2d => self.reg.l_reg = self.alu_dec(self.reg.l_reg),
            0x35 => {
                let a = self.reg.parse_hl();
                let v = self.mem.borrow().get(a);
                let h = self.alu_dec(v);
                self.mem.borrow_mut().set(a, h);
            }
            0x3d => self.reg.a_reg = self.alu_dec(self.reg.a_reg),

            // ADD HL, r16
            0x09 => self.alu_add_hl(self.reg.parse_bc()),
            0x19 => self.alu_add_hl(self.reg.parse_de()),
            0x29 => self.alu_add_hl(self.reg.parse_hl()),
            0x39 => self.alu_add_hl(self.reg.stack_pointer),

            // ADD SP, d8
            0xe8 => self.alu_add_sp(),

            // INC r16
            0x03 => {
                let v = self.reg.parse_bc().wrapping_add(1);
                self.reg.set_bc(v);
            }
            0x13 => {
                let v = self.reg.parse_de().wrapping_add(1);
                self.reg.set_de(v);
            }
            0x23 => {
                let v = self.reg.parse_hl().wrapping_add(1);
                self.reg.set_hl(v);
            }
            0x33 => {
                let v = self.reg.stack_pointer.wrapping_add(1);
                self.reg.stack_pointer = v;
            }

            // DEC r16
            0x0b => {
                let v = self.reg.parse_bc().wrapping_sub(1);
                self.reg.set_bc(v);
            }
            0x1b => {
                let v = self.reg.parse_de().wrapping_sub(1);
                self.reg.set_de(v);
            }
            0x2b => {
                let v = self.reg.parse_hl().wrapping_sub(1);
                self.reg.set_hl(v);
            }
            0x3b => {
                let v = self.reg.stack_pointer.wrapping_sub(1);
                self.reg.stack_pointer = v;
            }

            // DAA
            0x27 => self.alu_daa(),

            // CPL
            0x2f => self.alu_cpl(),

            // CCF
            0x3f => self.alu_ccf(),

            // SCF
            0x37 => self.alu_scf(),

            // NOP
            0x00 => {}

            // HALT
            0x76 => self.halted = true,

            // STOP
            0x10 => {}

            // DI/EI
            0xf3 => self.ei = false,
            0xfb => self.ei = true,

            // RLCA
            0x07 => {
                self.reg.a_reg = self.alu_rlc(self.reg.a_reg);
                self.reg.set_flag(ZeroFlag, false);
            }

            // RLA
            0x17 => {
                self.reg.a_reg = self.alu_rl(self.reg.a_reg);
                self.reg.set_flag(ZeroFlag, false);
            }

            // RRCA
            0x0f => {
                self.reg.a_reg = self.alu_rrc(self.reg.a_reg);
                self.reg.set_flag(ZeroFlag, false);
            }

            // RRA
            0x1f => {
                self.reg.a_reg = self.alu_rr(self.reg.a_reg);
                self.reg.set_flag(ZeroFlag, false);
            }

            // JUMP
            0xc3 => self.reg.program_counter = self.imm_word(),
            0xe9 => self.reg.program_counter = self.reg.parse_hl(),

            // JUMP IF
            0xc2 | 0xca | 0xd2 | 0xda => {
                let pc = self.imm_word();
                let cond = match opcode {
                    0xc2 => !self.reg.get_flag(ZeroFlag),
                    0xca => self.reg.get_flag(ZeroFlag),
                    0xd2 => !self.reg.get_flag(CarryFlag),
                    0xda => self.reg.get_flag(CarryFlag),
                    _ => panic!(""),
                };
                if cond {
                    self.reg.program_counter = pc;
                }
            }

            // JR
            0x18 => {
                let n = self.imm();
                self.alu_jr(n);
            }

            // JR IF
            0x20 | 0x28 | 0x30 | 0x38 => {
                let cond = match opcode {
                    0x20 => !self.reg.get_flag(ZeroFlag),
                    0x28 => self.reg.get_flag(ZeroFlag),
                    0x30 => !self.reg.get_flag(CarryFlag),
                    0x38 => self.reg.get_flag(CarryFlag),
                    _ => panic!(""),
                };
                let n = self.imm();
                if cond {
                    self.alu_jr(n);
                }
            }

            // CALL
            0xcd => {
                let nn = self.imm_word();
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = nn;
            }

            // CALL IF
            0xc4 | 0xcc | 0xd4 | 0xdc => {
                let cond = match opcode {
                    0xc4 => !self.reg.get_flag(ZeroFlag),
                    0xcc => self.reg.get_flag(ZeroFlag),
                    0xd4 => !self.reg.get_flag(CarryFlag),
                    0xdc => self.reg.get_flag(CarryFlag),
                    _ => panic!(""),
                };
                let nn = self.imm_word();
                if cond {
                    self.stack_add(self.reg.program_counter);
                    self.reg.program_counter = nn;
                }
            }

            // RST
            0xc7 => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x00;
            }
            0xcf => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x08;
            }
            0xd7 => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x10;
            }
            0xdf => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x18;
            }
            0xe7 => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x20;
            }
            0xef => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x28;
            }
            0xf7 => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x30;
            }
            0xff => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x38;
            }

            // RET
            0xc9 => self.reg.program_counter = self.stack_pop(),

            // RET IF
            0xc0 | 0xc8 | 0xd0 | 0xd8 => {
                let cond = match opcode {
                    0xc0 => !self.reg.get_flag(ZeroFlag),
                    0xc8 => self.reg.get_flag(ZeroFlag),
                    0xd0 => !self.reg.get_flag(CarryFlag),
                    0xd8 => self.reg.get_flag(CarryFlag),
                    _ => panic!(""),
                };
                if cond {
                    self.reg.program_counter = self.stack_pop();
                }
            }

            // RETI
            0xd9 => {
                self.reg.program_counter = self.stack_pop();
                self.ei = true;
            }

            // Extended Bit Operations
            0xcb => {
                cbcode = self.mem.borrow().get(self.reg.program_counter);
                self.reg.program_counter += 1;
                match cbcode {
                    // RLC r8
                    0x00 => self.reg.b_reg = self.alu_rlc(self.reg.b_reg),
                    0x01 => self.reg.c_reg = self.alu_rlc(self.reg.c_reg),
                    0x02 => self.reg.d_reg = self.alu_rlc(self.reg.d_reg),
                    0x03 => self.reg.e_reg = self.alu_rlc(self.reg.e_reg),
                    0x04 => self.reg.h_reg = self.alu_rlc(self.reg.h_reg),
                    0x05 => self.reg.l_reg = self.alu_rlc(self.reg.l_reg),
                    0x06 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_rlc(v);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x07 => self.reg.a_reg = self.alu_rlc(self.reg.a_reg),

                    // RRC r8
                    0x08 => self.reg.b_reg = self.alu_rrc(self.reg.b_reg),
                    0x09 => self.reg.c_reg = self.alu_rrc(self.reg.c_reg),
                    0x0a => self.reg.d_reg = self.alu_rrc(self.reg.d_reg),
                    0x0b => self.reg.e_reg = self.alu_rrc(self.reg.e_reg),
                    0x0c => self.reg.h_reg = self.alu_rrc(self.reg.h_reg),
                    0x0d => self.reg.l_reg = self.alu_rrc(self.reg.l_reg),
                    0x0e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_rrc(v);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x0f => self.reg.a_reg = self.alu_rrc(self.reg.a_reg),

                    // RL r8
                    0x10 => self.reg.b_reg = self.alu_rl(self.reg.b_reg),
                    0x11 => self.reg.c_reg = self.alu_rl(self.reg.c_reg),
                    0x12 => self.reg.d_reg = self.alu_rl(self.reg.d_reg),
                    0x13 => self.reg.e_reg = self.alu_rl(self.reg.e_reg),
                    0x14 => self.reg.h_reg = self.alu_rl(self.reg.h_reg),
                    0x15 => self.reg.l_reg = self.alu_rl(self.reg.l_reg),
                    0x16 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_rl(v);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x17 => self.reg.a_reg = self.alu_rl(self.reg.a_reg),

                    // RR r8
                    0x18 => self.reg.b_reg = self.alu_rr(self.reg.b_reg),
                    0x19 => self.reg.c_reg = self.alu_rr(self.reg.c_reg),
                    0x1a => self.reg.d_reg = self.alu_rr(self.reg.d_reg),
                    0x1b => self.reg.e_reg = self.alu_rr(self.reg.e_reg),
                    0x1c => self.reg.h_reg = self.alu_rr(self.reg.h_reg),
                    0x1d => self.reg.l_reg = self.alu_rr(self.reg.l_reg),
                    0x1e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_rr(v);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x1f => self.reg.a_reg = self.alu_rr(self.reg.a_reg),

                    // SLA r8
                    0x20 => self.reg.b_reg = self.alu_sla(self.reg.b_reg),
                    0x21 => self.reg.c_reg = self.alu_sla(self.reg.c_reg),
                    0x22 => self.reg.d_reg = self.alu_sla(self.reg.d_reg),
                    0x23 => self.reg.e_reg = self.alu_sla(self.reg.e_reg),
                    0x24 => self.reg.h_reg = self.alu_sla(self.reg.h_reg),
                    0x25 => self.reg.l_reg = self.alu_sla(self.reg.l_reg),
                    0x26 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_sla(v);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x27 => self.reg.a_reg = self.alu_sla(self.reg.a_reg),

                    // SRA r8
                    0x28 => self.reg.b_reg = self.alu_sra(self.reg.b_reg),
                    0x29 => self.reg.c_reg = self.alu_sra(self.reg.c_reg),
                    0x2a => self.reg.d_reg = self.alu_sra(self.reg.d_reg),
                    0x2b => self.reg.e_reg = self.alu_sra(self.reg.e_reg),
                    0x2c => self.reg.h_reg = self.alu_sra(self.reg.h_reg),
                    0x2d => self.reg.l_reg = self.alu_sra(self.reg.l_reg),
                    0x2e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_sra(v);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x2f => self.reg.a_reg = self.alu_sra(self.reg.a_reg),

                    // SWAP r8
                    0x30 => self.reg.b_reg = self.alu_swap(self.reg.b_reg),
                    0x31 => self.reg.c_reg = self.alu_swap(self.reg.c_reg),
                    0x32 => self.reg.d_reg = self.alu_swap(self.reg.d_reg),
                    0x33 => self.reg.e_reg = self.alu_swap(self.reg.e_reg),
                    0x34 => self.reg.h_reg = self.alu_swap(self.reg.h_reg),
                    0x35 => self.reg.l_reg = self.alu_swap(self.reg.l_reg),
                    0x36 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_swap(v);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x37 => self.reg.a_reg = self.alu_swap(self.reg.a_reg),

                    // SRL r8
                    0x38 => self.reg.b_reg = self.alu_srl(self.reg.b_reg),
                    0x39 => self.reg.c_reg = self.alu_srl(self.reg.c_reg),
                    0x3a => self.reg.d_reg = self.alu_srl(self.reg.d_reg),
                    0x3b => self.reg.e_reg = self.alu_srl(self.reg.e_reg),
                    0x3c => self.reg.h_reg = self.alu_srl(self.reg.h_reg),
                    0x3d => self.reg.l_reg = self.alu_srl(self.reg.l_reg),
                    0x3e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_srl(v);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x3f => self.reg.a_reg = self.alu_srl(self.reg.a_reg),

                    // BIT b, r8
                    0x40 => self.alu_bit(self.reg.b_reg, 0),
                    0x41 => self.alu_bit(self.reg.c_reg, 0),
                    0x42 => self.alu_bit(self.reg.d_reg, 0),
                    0x43 => self.alu_bit(self.reg.e_reg, 0),
                    0x44 => self.alu_bit(self.reg.h_reg, 0),
                    0x45 => self.alu_bit(self.reg.l_reg, 0),
                    0x46 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        self.alu_bit(v, 0);
                    }
                    0x47 => self.alu_bit(self.reg.a_reg, 0),
                    0x48 => self.alu_bit(self.reg.b_reg, 1),
                    0x49 => self.alu_bit(self.reg.c_reg, 1),
                    0x4a => self.alu_bit(self.reg.d_reg, 1),
                    0x4b => self.alu_bit(self.reg.e_reg, 1),
                    0x4c => self.alu_bit(self.reg.h_reg, 1),
                    0x4d => self.alu_bit(self.reg.l_reg, 1),
                    0x4e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        self.alu_bit(v, 1);
                    }
                    0x4f => self.alu_bit(self.reg.a_reg, 1),
                    0x50 => self.alu_bit(self.reg.b_reg, 2),
                    0x51 => self.alu_bit(self.reg.c_reg, 2),
                    0x52 => self.alu_bit(self.reg.d_reg, 2),
                    0x53 => self.alu_bit(self.reg.e_reg, 2),
                    0x54 => self.alu_bit(self.reg.h_reg, 2),
                    0x55 => self.alu_bit(self.reg.l_reg, 2),
                    0x56 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        self.alu_bit(v, 2);
                    }
                    0x57 => self.alu_bit(self.reg.a_reg, 2),
                    0x58 => self.alu_bit(self.reg.b_reg, 3),
                    0x59 => self.alu_bit(self.reg.c_reg, 3),
                    0x5a => self.alu_bit(self.reg.d_reg, 3),
                    0x5b => self.alu_bit(self.reg.e_reg, 3),
                    0x5c => self.alu_bit(self.reg.h_reg, 3),
                    0x5d => self.alu_bit(self.reg.l_reg, 3),
                    0x5e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        self.alu_bit(v, 3);
                    }
                    0x5f => self.alu_bit(self.reg.a_reg, 3),
                    0x60 => self.alu_bit(self.reg.b_reg, 4),
                    0x61 => self.alu_bit(self.reg.c_reg, 4),
                    0x62 => self.alu_bit(self.reg.d_reg, 4),
                    0x63 => self.alu_bit(self.reg.e_reg, 4),
                    0x64 => self.alu_bit(self.reg.h_reg, 4),
                    0x65 => self.alu_bit(self.reg.l_reg, 4),
                    0x66 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        self.alu_bit(v, 4);
                    }
                    0x67 => self.alu_bit(self.reg.a_reg, 4),
                    0x68 => self.alu_bit(self.reg.b_reg, 5),
                    0x69 => self.alu_bit(self.reg.c_reg, 5),
                    0x6a => self.alu_bit(self.reg.d_reg, 5),
                    0x6b => self.alu_bit(self.reg.e_reg, 5),
                    0x6c => self.alu_bit(self.reg.h_reg, 5),
                    0x6d => self.alu_bit(self.reg.l_reg, 5),
                    0x6e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        self.alu_bit(v, 5);
                    }
                    0x6f => self.alu_bit(self.reg.a_reg, 5),
                    0x70 => self.alu_bit(self.reg.b_reg, 6),
                    0x71 => self.alu_bit(self.reg.c_reg, 6),
                    0x72 => self.alu_bit(self.reg.d_reg, 6),
                    0x73 => self.alu_bit(self.reg.e_reg, 6),
                    0x74 => self.alu_bit(self.reg.h_reg, 6),
                    0x75 => self.alu_bit(self.reg.l_reg, 6),
                    0x76 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        self.alu_bit(v, 6);
                    }
                    0x77 => self.alu_bit(self.reg.a_reg, 6),
                    0x78 => self.alu_bit(self.reg.b_reg, 7),
                    0x79 => self.alu_bit(self.reg.c_reg, 7),
                    0x7a => self.alu_bit(self.reg.d_reg, 7),
                    0x7b => self.alu_bit(self.reg.e_reg, 7),
                    0x7c => self.alu_bit(self.reg.h_reg, 7),
                    0x7d => self.alu_bit(self.reg.l_reg, 7),
                    0x7e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        self.alu_bit(v, 7);
                    }
                    0x7f => self.alu_bit(self.reg.a_reg, 7),

                    // RES b, r8
                    0x80 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 0),
                    0x81 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 0),
                    0x82 => self.reg.d_reg = self.alu_res(self.reg.d_reg, 0),
                    0x83 => self.reg.e_reg = self.alu_res(self.reg.e_reg, 0),
                    0x84 => self.reg.h_reg = self.alu_res(self.reg.h_reg, 0),
                    0x85 => self.reg.l_reg = self.alu_res(self.reg.l_reg, 0),
                    0x86 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_res(v, 0);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x87 => self.reg.a_reg = self.alu_res(self.reg.a_reg, 0),
                    0x88 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 1),
                    0x89 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 1),
                    0x8a => self.reg.d_reg = self.alu_res(self.reg.d_reg, 1),
                    0x8b => self.reg.e_reg = self.alu_res(self.reg.e_reg, 1),
                    0x8c => self.reg.h_reg = self.alu_res(self.reg.h_reg, 1),
                    0x8d => self.reg.l_reg = self.alu_res(self.reg.l_reg, 1),
                    0x8e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_res(v, 1);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x8f => self.reg.a_reg = self.alu_res(self.reg.a_reg, 1),
                    0x90 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 2),
                    0x91 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 2),
                    0x92 => self.reg.d_reg = self.alu_res(self.reg.d_reg, 2),
                    0x93 => self.reg.e_reg = self.alu_res(self.reg.e_reg, 2),
                    0x94 => self.reg.h_reg = self.alu_res(self.reg.h_reg, 2),
                    0x95 => self.reg.l_reg = self.alu_res(self.reg.l_reg, 2),
                    0x96 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_res(v, 2);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x97 => self.reg.a_reg = self.alu_res(self.reg.a_reg, 2),
                    0x98 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 3),
                    0x99 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 3),
                    0x9a => self.reg.d_reg = self.alu_res(self.reg.d_reg, 3),
                    0x9b => self.reg.e_reg = self.alu_res(self.reg.e_reg, 3),
                    0x9c => self.reg.h_reg = self.alu_res(self.reg.h_reg, 3),
                    0x9d => self.reg.l_reg = self.alu_res(self.reg.l_reg, 3),
                    0x9e => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_res(v, 3);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0x9f => self.reg.a_reg = self.alu_res(self.reg.a_reg, 3),
                    0xa0 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 4),
                    0xa1 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 4),
                    0xa2 => self.reg.d_reg = self.alu_res(self.reg.d_reg, 4),
                    0xa3 => self.reg.e_reg = self.alu_res(self.reg.e_reg, 4),
                    0xa4 => self.reg.h_reg = self.alu_res(self.reg.h_reg, 4),
                    0xa5 => self.reg.l_reg = self.alu_res(self.reg.l_reg, 4),
                    0xa6 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_res(v, 4);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xa7 => self.reg.a_reg = self.alu_res(self.reg.a_reg, 4),
                    0xa8 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 5),
                    0xa9 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 5),
                    0xaa => self.reg.d_reg = self.alu_res(self.reg.d_reg, 5),
                    0xab => self.reg.e_reg = self.alu_res(self.reg.e_reg, 5),
                    0xac => self.reg.h_reg = self.alu_res(self.reg.h_reg, 5),
                    0xad => self.reg.l_reg = self.alu_res(self.reg.l_reg, 5),
                    0xae => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_res(v, 5);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xaf => self.reg.a_reg = self.alu_res(self.reg.a_reg, 5),
                    0xb0 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 6),
                    0xb1 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 6),
                    0xb2 => self.reg.d_reg = self.alu_res(self.reg.d_reg, 6),
                    0xb3 => self.reg.e_reg = self.alu_res(self.reg.e_reg, 6),
                    0xb4 => self.reg.h_reg = self.alu_res(self.reg.h_reg, 6),
                    0xb5 => self.reg.l_reg = self.alu_res(self.reg.l_reg, 6),
                    0xb6 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_res(v, 6);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xb7 => self.reg.a_reg = self.alu_res(self.reg.a_reg, 6),
                    0xb8 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 7),
                    0xb9 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 7),
                    0xba => self.reg.d_reg = self.alu_res(self.reg.d_reg, 7),
                    0xbb => self.reg.e_reg = self.alu_res(self.reg.e_reg, 7),
                    0xbc => self.reg.h_reg = self.alu_res(self.reg.h_reg, 7),
                    0xbd => self.reg.l_reg = self.alu_res(self.reg.l_reg, 7),
                    0xbe => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_res(v, 7);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xbf => self.reg.a_reg = self.alu_res(self.reg.a_reg, 7),

                    // SET b, r8
                    0xc0 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 0),
                    0xc1 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 0),
                    0xc2 => self.reg.d_reg = self.alu_set(self.reg.d_reg, 0),
                    0xc3 => self.reg.e_reg = self.alu_set(self.reg.e_reg, 0),
                    0xc4 => self.reg.h_reg = self.alu_set(self.reg.h_reg, 0),
                    0xc5 => self.reg.l_reg = self.alu_set(self.reg.l_reg, 0),
                    0xc6 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_set(v, 0);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xc7 => self.reg.a_reg = self.alu_set(self.reg.a_reg, 0),
                    0xc8 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 1),
                    0xc9 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 1),
                    0xca => self.reg.d_reg = self.alu_set(self.reg.d_reg, 1),
                    0xcb => self.reg.e_reg = self.alu_set(self.reg.e_reg, 1),
                    0xcc => self.reg.h_reg = self.alu_set(self.reg.h_reg, 1),
                    0xcd => self.reg.l_reg = self.alu_set(self.reg.l_reg, 1),
                    0xce => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_set(v, 1);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xcf => self.reg.a_reg = self.alu_set(self.reg.a_reg, 1),
                    0xd0 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 2),
                    0xd1 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 2),
                    0xd2 => self.reg.d_reg = self.alu_set(self.reg.d_reg, 2),
                    0xd3 => self.reg.e_reg = self.alu_set(self.reg.e_reg, 2),
                    0xd4 => self.reg.h_reg = self.alu_set(self.reg.h_reg, 2),
                    0xd5 => self.reg.l_reg = self.alu_set(self.reg.l_reg, 2),
                    0xd6 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_set(v, 2);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xd7 => self.reg.a_reg = self.alu_set(self.reg.a_reg, 2),
                    0xd8 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 3),
                    0xd9 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 3),
                    0xda => self.reg.d_reg = self.alu_set(self.reg.d_reg, 3),
                    0xdb => self.reg.e_reg = self.alu_set(self.reg.e_reg, 3),
                    0xdc => self.reg.h_reg = self.alu_set(self.reg.h_reg, 3),
                    0xdd => self.reg.l_reg = self.alu_set(self.reg.l_reg, 3),
                    0xde => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_set(v, 3);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xdf => self.reg.a_reg = self.alu_set(self.reg.a_reg, 3),
                    0xe0 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 4),
                    0xe1 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 4),
                    0xe2 => self.reg.d_reg = self.alu_set(self.reg.d_reg, 4),
                    0xe3 => self.reg.e_reg = self.alu_set(self.reg.e_reg, 4),
                    0xe4 => self.reg.h_reg = self.alu_set(self.reg.h_reg, 4),
                    0xe5 => self.reg.l_reg = self.alu_set(self.reg.l_reg, 4),
                    0xe6 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_set(v, 4);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xe7 => self.reg.a_reg = self.alu_set(self.reg.a_reg, 4),
                    0xe8 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 5),
                    0xe9 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 5),
                    0xea => self.reg.d_reg = self.alu_set(self.reg.d_reg, 5),
                    0xeb => self.reg.e_reg = self.alu_set(self.reg.e_reg, 5),
                    0xec => self.reg.h_reg = self.alu_set(self.reg.h_reg, 5),
                    0xed => self.reg.l_reg = self.alu_set(self.reg.l_reg, 5),
                    0xee => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_set(v, 5);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xef => self.reg.a_reg = self.alu_set(self.reg.a_reg, 5),
                    0xf0 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 6),
                    0xf1 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 6),
                    0xf2 => self.reg.d_reg = self.alu_set(self.reg.d_reg, 6),
                    0xf3 => self.reg.e_reg = self.alu_set(self.reg.e_reg, 6),
                    0xf4 => self.reg.h_reg = self.alu_set(self.reg.h_reg, 6),
                    0xf5 => self.reg.l_reg = self.alu_set(self.reg.l_reg, 6),
                    0xf6 => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_set(v, 6);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xf7 => self.reg.a_reg = self.alu_set(self.reg.a_reg, 6),
                    0xf8 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 7),
                    0xf9 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 7),
                    0xfa => self.reg.d_reg = self.alu_set(self.reg.d_reg, 7),
                    0xfb => self.reg.e_reg = self.alu_set(self.reg.e_reg, 7),
                    0xfc => self.reg.h_reg = self.alu_set(self.reg.h_reg, 7),
                    0xfd => self.reg.l_reg = self.alu_set(self.reg.l_reg, 7),
                    0xfe => {
                        let a = self.reg.parse_hl();
                        let v = self.mem.borrow().get(a);
                        let h = self.alu_set(v, 7);
                        self.mem.borrow_mut().set(a, h);
                    }
                    0xff => self.reg.a_reg = self.alu_set(self.reg.a_reg, 7),
                }
            }
            0xd3 => panic!("Opcode 0xd3 is not implemented"),
            0xdb => panic!("Opcode 0xdb is not implemented"),
            0xdd => panic!("Opcode 0xdd is not implemented"),
            0xe3 => panic!("Opcode 0xe3 is not implemented"),
            0xe4 => panic!("Opcode 0xd4 is not implemented"),
            0xeb => panic!("Opcode 0xeb is not implemented"),
            0xec => panic!("Opcode 0xec is not implemented"),
            0xed => panic!("Opcode 0xed is not implemented"),
            0xf4 => panic!("Opcode 0xf4 is not implemented"),
            0xfc => panic!("Opcode 0xfc is not implemented"),
            0xfd => panic!("Opcode 0xfd is not implemented"),
        };

        let ecycle = match opcode {
            0x20 | 0x30 => {
                if self.reg.get_flag(ZeroFlag) {
                    0x00
                } else {
                    0x01
                }
            }
            0x28 | 0x38 => {
                if self.reg.get_flag(ZeroFlag) {
                    0x01
                } else {
                    0x00
                }
            }
            0xc0 | 0xd0 => {
                if self.reg.get_flag(ZeroFlag) {
                    0x00
                } else {
                    0x03
                }
            }
            0xc8 | 0xcc | 0xd8 | 0xdc => {
                if self.reg.get_flag(ZeroFlag) {
                    0x03
                } else {
                    0x00
                }
            }
            0xc2 | 0xd2 => {
                if self.reg.get_flag(ZeroFlag) {
                    0x00
                } else {
                    0x01
                }
            }
            0xca | 0xda => {
                if self.reg.get_flag(ZeroFlag) {
                    0x01
                } else {
                    0x00
                }
            }
            0xc4 | 0xd4 => {
                if self.reg.get_flag(ZeroFlag) {
                    0x00
                } else {
                    0x03
                }
            }
            _ => 0x00,
        };
        if opcode == 0xcb {
            CB_CYCLES[cbcode as usize]
        } else {
            OP_CYCLES[opcode as usize] + ecycle
        }
    }

    pub fn next(&mut self) -> u32 {
        let mac = {
            let c = self.hi();
            if c != 0 {
                c
            } else if self.halted {
                OP_CYCLES[0]
            } else {
                self.ex()
            }
        };
        mac * 4
    }
}

pub struct RTC {
    pub cpu: Cpu,
    step_cycles: u32,
    step_zero: time::Instant,
    step_flip: bool,
}

impl RTC {
    pub fn power_up(term: Term, mem: Rc<RefCell<dyn Memory>>) -> Self {
        let cpu = Cpu::power_up(term, mem);
        Self { cpu, step_cycles: 0, step_zero: time::Instant::now(), step_flip: false }
    }
    pub fn next(&mut self) -> u32 {
        if self.step_cycles > STEP_CYCLES {
            self.step_flip = true;
            self.step_cycles -= STEP_CYCLES;
            let now = time::Instant::now();
            let d = now.duration_since(self.step_zero);
            let s = u64::from(STEP_TIME.saturating_sub(d.as_millis() as u32));
            thread::sleep(time::Duration::from_millis(s));
            self.step_zero = self.step_zero.checked_add(time::Duration::from_millis(u64::from(STEP_TIME))).unwrap();


            if now.checked_duration_since(self.step_zero).is_some() {
                self.step_zero = now;
            }
        }
        let cycles = self.cpu.next();
        self.step_cycles += cycles;
        cycles
    }
    
    pub fn flip(&mut self) -> bool {
        let r = self.step_flip;
        if r {
            self.step_flip = false;
        }
        r
    }
}