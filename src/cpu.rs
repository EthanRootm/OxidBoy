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
    ///Shift value right into Carry, setting MSB to 0
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
            //LD r8, n8
            0x06 => self.reg.b_reg = self.imm(),
            0x0E => self.reg.c_reg = self.imm(),
            0x16 => self.reg.d_reg = self.imm(),
            0x1E => self.reg.e_reg = self.imm(),
            0x26 => self.reg.h_reg = self.imm(),
            0x2E => self.reg.l_reg = self.imm(),
            0x36 => {
                let hl = self.reg.parse_hl();
                let imm = self.imm();
                self.mem.borrow_mut().set(hl, imm);
            }
            0x3E => self.reg.a_reg = self.imm(),

            //LD r16, A
            0x02 => self.mem.borrow_mut().set(self.reg.parse_bc(), self.reg.a_reg),
            0x12 => self.mem.borrow_mut().set(self.reg.parse_de(), self.reg.a_reg),

            //LD A, r16
            0x0A => self.reg.a_reg = self.mem.borrow().get(self.reg.parse_bc()),
            0x1A => self.reg.a_reg = self.mem.borrow().get(self.reg.parse_de()),

            //LD r16+, A
            0x22 => {
                let a = self.reg.parse_hl();
                self.mem.borrow_mut().set(a, self.reg.a_reg);
                self.reg.set_hl(a + 1);
            }

            //LD r16-, A
            0x32 => {
                let a = self.reg.parse_hl();
                self.mem.borrow_mut().set(a, self.reg.a_reg);
                self.reg.set_hl(a - 1);
            }

            //LD A, r16+
            0x2A => {
                let a = self.reg.parse_hl();
                self.reg.a_reg = self.mem.borrow().get(a);
                self.reg.set_hl(a + 1);
            }

            //LD A, r16-
            0x3A => {
                let a = self.reg.parse_hl();
                self.reg.a_reg = self.mem.borrow().get(a);
                self.reg.set_hl(a - 1);
            }
            
            //LD r8, r8
            0x40 => {/* b_reg = b_reg */}
            0x50 => self.reg.d_reg = self.reg.b_reg,
            0x60 => self.reg.h_reg = self.reg.b_reg,

            0x41 => self.reg.b_reg = self.reg.c_reg,
            0x51 => self.reg.d_reg = self.reg.c_reg,
            0x61 => self.reg.h_reg = self.reg.c_reg,

            0x42 => self.reg.b_reg = self.reg.d_reg,
            0x52 => {/* d_reg = d_reg */}
            0x62 => self.reg.h_reg = self.reg.d_reg,

            0x43 => self.reg.b_reg = self.reg.e_reg,
            0x53 => self.reg.d_reg = self.reg.e_reg,
            0x63 => self.reg.h_reg = self.reg.e_reg,

            0x44 => self.reg.b_reg = self.reg.h_reg,
            0x54 => self.reg.d_reg = self.reg.h_reg,
            0x64 => {/* h_reg = h_reg */}

            0x45 => self.reg.b_reg = self.reg.l_reg,
            0x55 => self.reg.d_reg = self.reg.l_reg,
            0x65 => self.reg.h_reg = self.reg.l_reg,

            0x47 => self.reg.b_reg = self.reg.a_reg,
            0x57 => self.reg.d_reg = self.reg.a_reg,
            0x67 => self.reg.h_reg = self.reg.a_reg,

            0x48 => self.reg.c_reg = self.reg.b_reg,
            0x58 => self.reg.e_reg = self.reg.b_reg,
            0x68 => self.reg.l_reg = self.reg.b_reg,
            0x78 => self.reg.a_reg = self.reg.b_reg,

            0x49 => {/* c_reg = c_reg */}
            0x59 => self.reg.e_reg = self.reg.c_reg,
            0x69 => self.reg.l_reg = self.reg.c_reg,
            0x79 => self.reg.a_reg = self.reg.c_reg,

            0x4A => self.reg.c_reg = self.reg.d_reg,
            0x5A => self.reg.e_reg = self.reg.d_reg,
            0x6A => self.reg.l_reg = self.reg.d_reg,
            0x7A => self.reg.a_reg = self.reg.d_reg,

            0x4B => self.reg.c_reg = self.reg.e_reg,
            0x5B => {/* e_reg = e_reg */}
            0x6B => self.reg.l_reg = self.reg.e_reg,
            0x7B => self.reg.a_reg = self.reg.e_reg,

            0x4C => self.reg.c_reg = self.reg.h_reg,
            0x5C => self.reg.e_reg = self.reg.h_reg,
            0x6C => self.reg.l_reg = self.reg.h_reg,
            0x7C => self.reg.a_reg = self.reg.h_reg,

            0x4D => self.reg.c_reg = self.reg.l_reg,
            0x5D => self.reg.e_reg = self.reg.l_reg,
            0x6D => {/* l_reg = l_reg */}
            0x7D => self.reg.a_reg = self.reg.l_reg,

            0x4F => self.reg.c_reg = self.reg.a_reg,
            0x5F => self.reg.e_reg = self.reg.a_reg,
            0x6F => self.reg.l_reg = self.reg.a_reg,
            0x7F => {/* a_reg = a_reg */}

            //LD HL, r8
            0x70 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.b_reg),
            0x71 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.c_reg),
            0x72 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.d_reg),
            0x73 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.e_reg),
            0x74 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.h_reg),
            0x75 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.l_reg),
            0x77 => self.mem.borrow_mut().set(self.reg.parse_hl(), self.reg.a_reg),

            //LD r8, HL
            0x46 => self.reg.b_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x56 => self.reg.d_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x66 => self.reg.h_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x4E => self.reg.c_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x5E => self.reg.e_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x6E => self.reg.e_reg = self.mem.borrow().get(self.reg.parse_hl()),
            0x7E => self.reg.a_reg = self.mem.borrow().get(self.reg.parse_hl()),

            //HALT
            0x76 => self.halted = true,

            //LDH a8, A
            0xE0 => {
                let a = 0xFF00 | u16::from(self.imm());
                self.mem.borrow_mut().set(a, self.reg.a_reg);
            }

            //LDH A, a8
            0xF0 => {
                let a = 0xFF00 | u16::from(self.imm());
                self.reg.a_reg = self.mem.borrow().get(a);
            }

            //LD [C], A
            0xE2 => self.mem.borrow_mut().set(0xFF00 | u16::from(self.reg.c_reg), self.reg.a_reg),

            //LD A, [C]
            0xF2 => self.reg.a_reg = self.mem.borrow().get(0xFF00 | u16::from(self.reg.c_reg)),

            //LD [a16], A
            0xEA => {
                let a = self.imm_word();
                self.mem.borrow_mut().set(a, self.reg.a_reg);
            }

            //LD A, [a16]
            0xFA => {
                let a = self.imm_word();
                self.reg.a_reg = self.mem.borrow().get(a);
            }
            

            //NOP
            0x00 => {/* No Operation */}

            //STOP
            0x10 => {}

            //LD r16, n16
            0x01|0x11|0x21|0x31 => {
                let n16 = self.imm_word();
                match opcode {
                    0x01 => self.reg.set_bc(n16),
                    0x11 => self.reg.set_de(n16),
                    0x21 => self.reg.set_hl(n16),
                    0x31 => self.reg.stack_pointer = n16,
                    _ => {}
                }
            }
            
            //LD [a16], SP
            0x08 => {
                let a = self.imm_word();
                self.mem.borrow_mut().set_word(a, self.reg.stack_pointer);
            }

            //POP r16
            0xC1 | 0xD1 | 0xE1 | 0xF1 => {
                let v = self.stack_pop();
                match opcode {
                    0xC1 => self.reg.set_bc(v),
                    0xD1 => self.reg.set_de(v),
                    0xE1 => self.reg.set_hl(v),
                    0xF1 => self.reg.set_af(v),
                    _ => {}  
                }
             
        }

            //PUSH r16
            0xC5 => self.stack_add(self.reg.parse_bc()),
            0xD5 => self.stack_add(self.reg.parse_de()),
            0xE5 => self.stack_add(self.reg.parse_hl()),
            0xF5 => self.stack_add(self.reg.parse_af()),

            //LD HL, SP+e8
            0xF8 => {
                let sp = self.reg.stack_pointer;
                let e8 = i16::from(self.imm() as i8) as u16;
                self.reg.set_flag(CarryFlag, (sp & 0x00FF) + (e8 & 0x00FF) > 0x00FF);
                self.reg.set_flag(HalfCarryFlag, (sp + 0x000F) + (e8 & 0x000F) > 0x000F);
                self.reg.set_flag(ZeroFlag, false);
                self.reg.set_flag(SubtractionFlag, false);
                self.reg.set_hl(sp.wrapping_add(e8));
            }
            
            //LD SP, HL
            0xF9 => self.reg.stack_pointer = self.reg.parse_hl(),

            //ADD A, r8
            0x80 => self.alu_add(self.reg.b_reg),
            0x81 => self.alu_add(self.reg.c_reg),
            0x82 => self.alu_add(self.reg.d_reg),
            0x83 => self.alu_add(self.reg.e_reg),
            0x84 => self.alu_add(self.reg.h_reg),
            0x85 => self.alu_add(self.reg.l_reg),
            0x86 => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                self.alu_add(b);
            }
            0x87 => self.alu_add(self.reg.a_reg),
            0xC6 => {
                let a = self.imm();
                self.alu_add(a);
            }

            //SUB A, r8
            0x90 => self.alu_sub(self.reg.b_reg),
            0x91 => self.alu_sub(self.reg.c_reg),
            0x92 => self.alu_sub(self.reg.d_reg),
            0x93 => self.alu_sub(self.reg.e_reg),
            0x94 => self.alu_sub(self.reg.h_reg),
            0x95 => self.alu_sub(self.reg.l_reg),
            0x96 => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                self.alu_sub(b);
            }
            0x97 => self.alu_sub(self.reg.a_reg),
            0xD6 => {
                let a = self.imm();
                self.alu_sub(a);
            }

            //AND A, r8
            0xA0 => self.alu_and(self.reg.b_reg),
            0xA1 => self.alu_and(self.reg.c_reg),
            0xA2 => self.alu_and(self.reg.d_reg),
            0xA3 => self.alu_and(self.reg.e_reg),
            0xA4 => self.alu_and(self.reg.h_reg),
            0xA5 => self.alu_and(self.reg.l_reg),
            0xA6 => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                self.alu_and(b);
            }
            0xA7 => self.alu_and(self.reg.a_reg),
            0xE6 => {
                let a = self.imm();
                self.alu_and(a);
            }

            //OR A, r8
            0xB0 => self.alu_or(self.reg.b_reg),
            0xB1 => self.alu_or(self.reg.c_reg),
            0xB2 => self.alu_or(self.reg.d_reg),
            0xB3 => self.alu_or(self.reg.e_reg),
            0xB4 => self.alu_or(self.reg.h_reg),
            0xB5 => self.alu_or(self.reg.l_reg),
            0xB6 => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                self.alu_or(b);
            }
            0xB7 => self.alu_or(self.reg.a_reg),
            0xF6 => {
                let a = self.imm();
                self.alu_or(a);
            }

            //ADC A, r8
            0x88 => self.alu_adc(self.reg.b_reg),
            0x89 => self.alu_adc(self.reg.c_reg),
            0x8A => self.alu_adc(self.reg.d_reg),
            0x8B => self.alu_adc(self.reg.e_reg),
            0x8C => self.alu_adc(self.reg.h_reg),
            0x8D => self.alu_adc(self.reg.l_reg),
            0x8E => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                self.alu_adc(b);
            }
            0x8F => self.alu_adc(self.reg.a_reg),
            0xCE => {
                let a = self.imm();
                self.alu_adc(a);
            }

            //SBC A, r8
            0x98 => self.alu_sbc(self.reg.b_reg),
            0x99 => self.alu_sbc(self.reg.c_reg),
            0x9A => self.alu_sbc(self.reg.d_reg),
            0x9B => self.alu_sbc(self.reg.e_reg),
            0x9C => self.alu_sbc(self.reg.h_reg),
            0x9D => self.alu_sbc(self.reg.l_reg),
            0x9E => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                self.alu_sbc(b);
            }
            0x9F => self.alu_sbc(self.reg.a_reg),
            0xDE => {
                let a = self.imm();
                self.alu_sbc(a);
            }

            //XOR A, r8
            0xA8 => self.alu_xor(self.reg.b_reg),
            0xA9 => self.alu_xor(self.reg.c_reg),
            0xAA => self.alu_xor(self.reg.d_reg),
            0xAB => self.alu_xor(self.reg.e_reg),
            0xAC => self.alu_xor(self.reg.h_reg),
            0xAD => self.alu_xor(self.reg.l_reg),
            0xAE => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                self.alu_xor(b);
            }
            0xAF => self.alu_xor(self.reg.a_reg),
            0xEE => {
                let a = self.imm();
                self.alu_xor(a);   
            }

            //CP A, r8
            0xB8 => self.alu_cp(self.reg.b_reg), 
            0xB9 => self.alu_cp(self.reg.c_reg),
            0xBA => self.alu_cp(self.reg.d_reg),
            0xBB => self.alu_cp(self.reg.e_reg),
            0xBC => self.alu_cp(self.reg.h_reg),
            0xBD => self.alu_cp(self.reg.l_reg),
            0xBE => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                self.alu_cp(b);
            }
            0xBF => self.alu_cp(self.reg.a_reg),
            0xFE => {
                let a = self.imm();
                self.alu_cp(a);
            }

            //INC r8
            0x04 => self.reg.b_reg = self.alu_inc(self.reg.b_reg),
            0x14 => self.reg.d_reg = self.alu_inc(self.reg.d_reg),
            0x24 => self.reg.h_reg = self.alu_inc(self.reg.h_reg),
            0x34 => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                let c = self.alu_inc(b);
                self.mem.borrow_mut().set(a,c);
            }
            0x0C => self.reg.c_reg = self.alu_inc(self.reg.c_reg),
            0x1C => self.reg.e_reg = self.alu_inc(self.reg.e_reg),
            0x2C => self.reg.l_reg = self.alu_inc(self.reg.l_reg),
            0x3C => self.reg.a_reg = self.alu_inc(self.reg.a_reg),

            //DEC r8
            0x05 => self.reg.b_reg = self.alu_dec(self.reg.b_reg),
            0x15 => self.reg.d_reg = self.alu_dec(self.reg.d_reg),
            0x25 => self.reg.h_reg = self.alu_dec(self.reg.h_reg),
            0x35 => {
                let a = self.reg.parse_hl();
                let b = self.mem.borrow().get(a);
                let c = self.alu_dec(b);
                self.mem.borrow_mut().set(a, c);
            }
            0x0D => self.reg.c_reg = self.alu_dec(self.reg.c_reg),
            0x1D => self.reg.e_reg = self.alu_dec(self.reg.e_reg),
            0x2D => self.reg.l_reg = self.alu_dec(self.reg.l_reg),
            0x3D => self.reg.a_reg = self.alu_dec(self.reg.a_reg),

            //INC r16
            0x03 => self.reg.set_bc(self.reg.parse_bc().wrapping_add(1)),
            0x13 => self.reg.set_de(self.reg.parse_de().wrapping_add(1)),
            0x23 => self.reg.set_hl(self.reg.parse_hl().wrapping_add(1)),
            0x33 => self.reg.program_counter += 1,

            //DEC r16 
            0x0B => self.reg.set_bc(self.reg.parse_bc().wrapping_sub(1)),
            0x1B => self.reg.set_de(self.reg.parse_de().wrapping_sub(1)),
            0x2B => self.reg.set_hl(self.reg.parse_hl().wrapping_sub(1)),
            0x3B => self.reg.program_counter -= 1,

            //ADD HL, r16 - add 16 bit value to the HL register
            0x09 => self.alu_add_hl(self.reg.parse_bc()),
            0x19 => self.alu_add_hl(self.reg.parse_de()),
            0x29 => self.alu_add_hl(self.reg.parse_hl()),
            0x39 => self.alu_add_hl(self.reg.stack_pointer),

            //ADD SP, e8
            0xE8 => self.alu_add_sp(),

            //DI - 
            0xF3 => self.ei = false,

            //EI
            0xFB => self.ei = false,

            //RST
            0xC7 => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x00;
            }
            0xD7 => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x10;
            }
            0xE7 => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x20;
            }
            0xF7 => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x30;
            }
            0xCF => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x08;
            }
            0xDF => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x18;
            } 
            0xEF => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x28;
            }
            0xFF => {
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = 0x38;
            }

            //JR IF
            0x20 | 0x30 | 0x28 | 0x38 => {
                let cond = match opcode {
                    0x20 => !self.reg.get_flag(ZeroFlag),
                    0x30 => !self.reg.get_flag(CarryFlag),
                    0x28 => self.reg.get_flag(ZeroFlag),
                    0x38 => self.reg.get_flag(CarryFlag),
                    _ => panic!(""),
                };
                let n = self.imm();
                if cond {
                    self.alu_jr(n);
                }
            }

            //JR e8
            0x18 => {
                let a = self.imm();
                self.alu_jr(a);
            }

            //BIT Shifts
            0x07 => {
                self.reg.a_reg = self.alu_rlc(self.reg.a_reg);
                self.reg.set_flag(ZeroFlag, false);
            }
            0x17 => {
                self.reg.a_reg = self.alu_rl(self.reg.a_reg);
                self.reg.set_flag(ZeroFlag, false);
            }
            0x0F => {
                self.reg.a_reg = self.alu_rrc(self.reg.a_reg);
                self.reg.set_flag(ZeroFlag, false);
            }
            0x1F => {
                self.reg.a_reg = self.alu_rr(self.reg.a_reg);
                self.reg.set_flag(ZeroFlag, false);
            }

            //DAA
            0x27 => self.alu_daa(),

            //SCF
            0x37 => self.alu_scf(),

            //CPL
            0x2F => self.alu_cpl(),

            //CCF
            0x3F => self.alu_ccf(),

            //RET If
            0xC0 | 0xD0 | 0xC8 | 0xD8 => {
                let cond = match opcode {
                    0xC0 => !self.reg.get_flag(ZeroFlag),
                    0xD0 => !self.reg.get_flag(CarryFlag),
                    0xC8 => self.reg.get_flag(ZeroFlag),
                    0xD8 => self.reg.get_flag(CarryFlag),
                    _ => panic!(""),
                };
                if cond {
                    self.reg.program_counter = self.stack_pop();
                }
            }
            //RET
            0xC9 => self.reg.program_counter = self.stack_pop(),

            //RETI
            0xD9 => {
                self.reg.program_counter = self.stack_pop();
                self.ei = true;
            }

            //JP
            0xC3 => self.reg.program_counter = self.imm_word(),
            0xE9 => self.reg.program_counter = self.reg.parse_hl(),

            //JP IF
            0xC2 | 0xD2 | 0xCA | 0xDA => {
                let a = self.imm_word();
                let cond = match opcode {
                    0xC2 => !self.reg.get_flag(ZeroFlag),
                    0xD2 => !self.reg.get_flag(CarryFlag),
                    0xCA => self.reg.get_flag(ZeroFlag),
                    0xDA => self.reg.get_flag(CarryFlag),
                    _ => panic!()
                };
                if cond {
                    self.reg.program_counter = a;
                }
            }

            //CALL
            0xCD => {
                let a = self.imm_word();
                self.stack_add(self.reg.program_counter);
                self.reg.program_counter = a;
            }

            //CALL IF
            0xC4 | 0xD4 | 0xCC | 0xDC => {
                let a = self.imm_word();
                let cond = match opcode {
                    0xC4 => !self.reg.get_flag(ZeroFlag),
                    0xD4 => !self.reg.get_flag(CarryFlag),
                    0xCC => self.reg.get_flag(ZeroFlag),
                    0xDC => self.reg.get_flag(CarryFlag),
                    _ => panic!(),
                };
                if cond {
                    self.stack_add(self.reg.program_counter);
                    self.reg.program_counter = a;
                }
            }

            //Extended Bit Operations
            0xCB => {
                cbcode = self.mem.borrow().get(self.reg.program_counter);
                self.reg.program_counter += 1;
                match cbcode {
                    //RLC r8
                    0x00 => self.reg.b_reg = self.alu_rlc(self.reg.b_reg),
                    0x01 => self.reg.c_reg = self.alu_rlc(self.reg.c_reg),
                    0x02 => self.reg.d_reg = self.alu_rlc(self.reg.d_reg),
                    0x03 => self.reg.e_reg = self.alu_rlc(self.reg.e_reg),
                    0x04 => self.reg.h_reg = self.alu_rlc(self.reg.h_reg),
                    0x05 => self.reg.l_reg = self.alu_rlc(self.reg.l_reg),
                    0x06 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_rlc(b);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x07 => self.reg.a_reg = self.alu_rlc(self.reg.a_reg),

                    //RRC r8
                    0x08 => self.reg.b_reg = self.alu_rrc(self.reg.b_reg),
                    0x09 => self.reg.c_reg = self.alu_rrc(self.reg.c_reg),
                    0x0A => self.reg.d_reg = self.alu_rrc(self.reg.d_reg),
                    0x0B => self.reg.e_reg = self.alu_rrc(self.reg.e_reg),
                    0x0C => self.reg.h_reg = self.alu_rrc(self.reg.h_reg),
                    0x0D => self.reg.l_reg = self.alu_rrc(self.reg.l_reg),
                    0x0E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_rrc(b);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x0F => self.reg.a_reg = self.alu_rrc(self.reg.a_reg),

                    //RL r8
                    0x10 => self.reg.b_reg = self.alu_rl(self.reg.b_reg),
                    0x11 => self.reg.c_reg = self.alu_rl(self.reg.c_reg),
                    0x12 => self.reg.d_reg = self.alu_rl(self.reg.d_reg),
                    0x13 => self.reg.e_reg = self.alu_rl(self.reg.e_reg),
                    0x14 => self.reg.h_reg = self.alu_rl(self.reg.h_reg),
                    0x15 => self.reg.l_reg = self.alu_rl(self.reg.l_reg),
                    0x16 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_rl(b);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x17 => self.reg.a_reg = self.alu_rl(self.reg.a_reg),

                    //RR r8
                    0x18 => self.reg.b_reg = self.alu_rr(self.reg.b_reg),
                    0x19 => self.reg.c_reg = self.alu_rr(self.reg.c_reg),
                    0x1A => self.reg.d_reg = self.alu_rr(self.reg.d_reg),
                    0x1B => self.reg.e_reg = self.alu_rr(self.reg.e_reg),
                    0x1C => self.reg.h_reg = self.alu_rr(self.reg.h_reg),
                    0x1D => self.reg.l_reg = self.alu_rr(self.reg.l_reg),
                    0x1E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_rr(b);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x1F => self.reg.a_reg = self.alu_rr(self.reg.a_reg),

                    //SLA r8
                    0x20 => self.reg.b_reg = self.alu_sla(self.reg.b_reg),
                    0x21 => self.reg.c_reg = self.alu_sla(self.reg.c_reg),
                    0x22 => self.reg.d_reg = self.alu_sla(self.reg.d_reg),
                    0x23 => self.reg.e_reg = self.alu_sla(self.reg.e_reg),
                    0x24 => self.reg.h_reg = self.alu_sla(self.reg.h_reg),
                    0x25 => self.reg.l_reg = self.alu_sla(self.reg.l_reg),
                    0x26 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_sla(b);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x27 => self.reg.a_reg = self.alu_sla(self.reg.a_reg),

                    //SRA r8
                    0x28 => self.reg.b_reg = self.alu_sra(self.reg.b_reg),
                    0x29 => self.reg.c_reg = self.alu_sra(self.reg.c_reg),
                    0x2A => self.reg.d_reg = self.alu_sra(self.reg.d_reg),
                    0x2B => self.reg.e_reg = self.alu_sra(self.reg.e_reg),
                    0x2C => self.reg.h_reg = self.alu_sra(self.reg.h_reg),
                    0x2D => self.reg.l_reg = self.alu_sra(self.reg.l_reg),
                    0x2E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_sra(b);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x2F => self.reg.a_reg = self.alu_sra(self.reg.a_reg),

                    //SWAP r8
                    0x30 => self.reg.b_reg = self.alu_swap(self.reg.b_reg),
                    0x31 => self.reg.c_reg = self.alu_swap(self.reg.c_reg),
                    0x32 => self.reg.d_reg = self.alu_swap(self.reg.d_reg),
                    0x33 => self.reg.e_reg = self.alu_swap(self.reg.e_reg),
                    0x34 => self.reg.h_reg = self.alu_swap(self.reg.h_reg),
                    0x35 => self.reg.l_reg = self.alu_swap(self.reg.l_reg),
                    0x36 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_swap(b);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x37 => self.reg.a_reg = self.alu_swap(self.reg.a_reg),

                    //SRL r8
                    0x38 => self.reg.b_reg = self.alu_srl(self.reg.b_reg),
                    0x39 => self.reg.c_reg = self.alu_srl(self.reg.c_reg),
                    0x3A => self.reg.d_reg = self.alu_srl(self.reg.d_reg),
                    0x3B => self.reg.e_reg = self.alu_srl(self.reg.e_reg),
                    0x3C => self.reg.h_reg = self.alu_srl(self.reg.h_reg),
                    0x3D => self.reg.l_reg = self.alu_srl(self.reg.l_reg),
                    0x3E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_srl(b);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x3F => self.reg.a_reg = self.alu_srl(self.reg.a_reg),

                    //BIT u3, r8
                    0x40 => self.alu_bit(self.reg.b_reg, 0),
                    0x41 => self.alu_bit(self.reg.c_reg, 0),
                    0x42 => self.alu_bit(self.reg.d_reg, 0),
                    0x43 => self.alu_bit(self.reg.e_reg, 0),
                    0x44 => self.alu_bit(self.reg.h_reg, 0),
                    0x45 => self.alu_bit(self.reg.l_reg, 0),
                    0x46 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        self.alu_bit(b , 0);
                    }
                    0x47 => self.alu_bit(self.reg.a_reg, 1),
                    0x48 => self.alu_bit(self.reg.b_reg, 1),
                    0x49 => self.alu_bit(self.reg.c_reg, 1),
                    0x4A => self.alu_bit(self.reg.d_reg, 1),
                    0x4B => self.alu_bit(self.reg.e_reg, 1),
                    0x4C => self.alu_bit(self.reg.h_reg, 1),
                    0x4D => self.alu_bit(self.reg.l_reg, 1),
                    0x4E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        self.alu_bit(b , 1);
                    }
                    0x4F => self.alu_bit(self.reg.a_reg, 1),
                    0x50 => self.alu_bit(self.reg.b_reg, 2),
                    0x51 => self.alu_bit(self.reg.c_reg, 2),
                    0x52 => self.alu_bit(self.reg.d_reg, 2),
                    0x53 => self.alu_bit(self.reg.e_reg, 2),
                    0x54 => self.alu_bit(self.reg.h_reg, 2),
                    0x55 => self.alu_bit(self.reg.l_reg, 2),
                    0x56 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        self.alu_bit(b , 2);
                    }
                    0x57 => self.alu_bit(self.reg.a_reg, 2),
                    0x58 => self.alu_bit(self.reg.b_reg, 3),
                    0x59 => self.alu_bit(self.reg.c_reg, 3),
                    0x5A => self.alu_bit(self.reg.d_reg, 3),
                    0x5B => self.alu_bit(self.reg.e_reg, 3),
                    0x5C => self.alu_bit(self.reg.h_reg, 3),
                    0x5D => self.alu_bit(self.reg.l_reg, 3),
                    0x5E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        self.alu_bit(b , 3);
                    }
                    0x5F => self.alu_bit(self.reg.a_reg, 3),
                    0x60 => self.alu_bit(self.reg.b_reg, 4),
                    0x61 => self.alu_bit(self.reg.c_reg, 4),
                    0x62 => self.alu_bit(self.reg.d_reg, 4),
                    0x63 => self.alu_bit(self.reg.e_reg, 4),
                    0x64 => self.alu_bit(self.reg.h_reg, 4),
                    0x65 => self.alu_bit(self.reg.l_reg, 4),
                    0x66 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        self.alu_bit(b , 4);
                    }
                    0x67 => self.alu_bit(self.reg.a_reg, 4),
                    0x68 => self.alu_bit(self.reg.b_reg, 5),
                    0x69 => self.alu_bit(self.reg.c_reg, 5),
                    0x6A => self.alu_bit(self.reg.d_reg, 5),
                    0x6B => self.alu_bit(self.reg.e_reg, 5),
                    0x6C => self.alu_bit(self.reg.h_reg, 5),
                    0x6D => self.alu_bit(self.reg.l_reg, 5),
                    0x6E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        self.alu_bit(b , 5);
                    }
                    0x6F => self.alu_bit(self.reg.a_reg, 5),
                    0x70 => self.alu_bit(self.reg.b_reg, 6),
                    0x71 => self.alu_bit(self.reg.c_reg, 6),
                    0x72 => self.alu_bit(self.reg.d_reg, 6),
                    0x73 => self.alu_bit(self.reg.e_reg, 6),
                    0x74 => self.alu_bit(self.reg.h_reg, 6),
                    0x75 => self.alu_bit(self.reg.l_reg, 6),
                    0x76 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        self.alu_bit(b , 6);
                    }
                    0x77 => self.alu_bit(self.reg.a_reg, 6),
                    0x78 => self.alu_bit(self.reg.b_reg, 7),
                    0x79 => self.alu_bit(self.reg.c_reg, 7),
                    0x7A => self.alu_bit(self.reg.d_reg, 7),
                    0x7B => self.alu_bit(self.reg.e_reg, 7),
                    0x7C => self.alu_bit(self.reg.h_reg, 7),
                    0x7D => self.alu_bit(self.reg.l_reg, 7),
                    0x7E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        self.alu_bit(b , 7);
                    }
                    0x7F => self.alu_bit(self.reg.a_reg, 7),

                    //RES u3, r8
                    0x80 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 0),
                    0x81 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 0),
                    0x82 => self.reg.d_reg = self.alu_res(self.reg.d_reg, 0),
                    0x83 => self.reg.e_reg = self.alu_res(self.reg.e_reg, 0),
                    0x84 => self.reg.h_reg = self.alu_res(self.reg.h_reg, 0),
                    0x85 => self.reg.l_reg = self.alu_res(self.reg.l_reg, 0),
                    0x86 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_res(b, 0);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x87 => self.reg.a_reg = self.alu_res(self.reg.a_reg, 0),
                    0x88 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 1),
                    0x89 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 1),
                    0x8A => self.reg.d_reg = self.alu_res(self.reg.d_reg, 1),
                    0x8B => self.reg.e_reg = self.alu_res(self.reg.e_reg, 1),
                    0x8C => self.reg.h_reg = self.alu_res(self.reg.h_reg, 1),
                    0x8D => self.reg.l_reg = self.alu_res(self.reg.l_reg, 1),
                    0x8E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_res(b, 1);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x8F => self.reg.a_reg = self.alu_res(self.reg.a_reg, 1),
                    0x90 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 2),
                    0x91 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 2),
                    0x92 => self.reg.d_reg = self.alu_res(self.reg.d_reg, 2),
                    0x93 => self.reg.e_reg = self.alu_res(self.reg.e_reg, 2),
                    0x94 => self.reg.h_reg = self.alu_res(self.reg.h_reg, 2),
                    0x95 => self.reg.l_reg = self.alu_res(self.reg.l_reg, 2),
                    0x96 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_res(b, 2);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x97 => self.reg.a_reg = self.alu_res(self.reg.a_reg, 2),
                    0x98 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 3),
                    0x99 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 3),
                    0x9A => self.reg.d_reg = self.alu_res(self.reg.d_reg, 3),
                    0x9B => self.reg.e_reg = self.alu_res(self.reg.e_reg, 3),
                    0x9C => self.reg.h_reg = self.alu_res(self.reg.h_reg, 3),
                    0x9D => self.reg.l_reg = self.alu_res(self.reg.l_reg, 3),
                    0x9E => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_res(b, 3);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0x9F => self.reg.a_reg =self.alu_res(self.reg.a_reg, 3),
                    0xA0 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 4),
                    0xA1 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 4),
                    0xA2 => self.reg.d_reg = self.alu_res(self.reg.d_reg, 4),
                    0xA3 => self.reg.e_reg = self.alu_res(self.reg.e_reg, 4),
                    0xA4 => self.reg.h_reg = self.alu_res(self.reg.h_reg, 4),
                    0xA5 => self.reg.l_reg = self.alu_res(self.reg.l_reg, 4),
                    0xA6 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_res(b, 4);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xA7 => self.reg.a_reg = self.alu_res(self.reg.a_reg, 4),
                    0xA8 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 5),
                    0xA9 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 5),
                    0xAA => self.reg.d_reg = self.alu_res(self.reg.d_reg, 5),
                    0xAB => self.reg.e_reg = self.alu_res(self.reg.e_reg, 5),
                    0xAC => self.reg.h_reg = self.alu_res(self.reg.h_reg, 5),
                    0xAD => self.reg.l_reg = self.alu_res(self.reg.l_reg, 5),
                    0xAE => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_res(b, 5);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xAF => self.reg.a_reg = self.alu_res(self.reg.a_reg, 5),
                    0xB0 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 6),
                    0xB1 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 6),
                    0xB2 => self.reg.d_reg = self.alu_res(self.reg.d_reg, 6),
                    0xB3 => self.reg.e_reg = self.alu_res(self.reg.e_reg, 6),
                    0xB4 => self.reg.h_reg = self.alu_res(self.reg.h_reg, 6),
                    0xB5 => self.reg.l_reg = self.alu_res(self.reg.l_reg, 6),
                    0xB6 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_res(b, 6);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xB7 => self.reg.a_reg = self.alu_res(self.reg.a_reg, 6),
                    0xB8 => self.reg.b_reg = self.alu_res(self.reg.b_reg, 7),
                    0xB9 => self.reg.c_reg = self.alu_res(self.reg.c_reg, 7),
                    0xBA => self.reg.d_reg = self.alu_res(self.reg.d_reg, 7),
                    0xBB => self.reg.e_reg = self.alu_res(self.reg.e_reg, 7),
                    0xBC => self.reg.h_reg = self.alu_res(self.reg.h_reg, 7),
                    0xBD => self.reg.l_reg = self.alu_res(self.reg.l_reg, 7),
                    0xBE => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_res(b, 7);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xBF => self.reg.a_reg = self.alu_res(self.reg.a_reg, 7),

                    //SET u3, r8
                    0xC0 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 0),
                    0xC1 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 0),
                    0xC2 => self.reg.d_reg = self.alu_set(self.reg.d_reg, 0),
                    0xC3 => self.reg.e_reg = self.alu_set(self.reg.e_reg, 0),
                    0xC4 => self.reg.h_reg = self.alu_set(self.reg.h_reg, 0),
                    0xC5 => self.reg.l_reg = self.alu_set(self.reg.l_reg, 0),
                    0xC6 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_set(b, 0);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xC7 => self.reg.a_reg = self.alu_set(self.reg.a_reg, 0),
                    0xC8 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 1),
                    0xC9 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 1),
                    0xCA => self.reg.d_reg = self.alu_set(self.reg.d_reg, 1),
                    0xCB => self.reg.e_reg = self.alu_set(self.reg.e_reg, 1),
                    0xCC => self.reg.h_reg = self.alu_set(self.reg.h_reg, 1),
                    0xCD => self.reg.l_reg = self.alu_set(self.reg.l_reg, 1),
                    0xCE => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_set(b, 1);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xCF => self.reg.a_reg = self.alu_set(self.reg.a_reg, 1),
                    0xD0 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 2),
                    0xD1 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 2),
                    0xD2 => self.reg.d_reg = self.alu_set(self.reg.d_reg, 2),
                    0xD3 => self.reg.e_reg = self.alu_set(self.reg.e_reg, 2),
                    0xD4 => self.reg.h_reg = self.alu_set(self.reg.h_reg, 2),
                    0xD5 => self.reg.l_reg = self.alu_set(self.reg.l_reg, 2),
                    0xD6 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_set(b, 2);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xD7 => self.reg.a_reg = self.alu_set(self.reg.a_reg, 2),
                    0xD8 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 3),
                    0xD9 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 3),
                    0xDA => self.reg.d_reg = self.alu_set(self.reg.d_reg, 3),
                    0xDB => self.reg.e_reg = self.alu_set(self.reg.e_reg, 3),
                    0xDC => self.reg.h_reg = self.alu_set(self.reg.h_reg, 3),
                    0xDD => self.reg.l_reg = self.alu_set(self.reg.l_reg, 3),
                    0xDE => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_set(b, 3);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xDF => self.reg.a_reg = self.alu_set(self.reg.a_reg, 3),
                    0xE0 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 4),
                    0xE1 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 4),
                    0xE2 => self.reg.d_reg = self.alu_set(self.reg.d_reg, 4),
                    0xE3 => self.reg.e_reg = self.alu_set(self.reg.e_reg, 4),
                    0xE4 => self.reg.h_reg = self.alu_set(self.reg.h_reg, 4),
                    0xE5 => self.reg.l_reg = self.alu_set(self.reg.l_reg, 4),
                    0xE6 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_set(b, 4);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xE7 => self.reg.a_reg = self.alu_set(self.reg.a_reg, 4),
                    0xE8 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 5),
                    0xE9 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 5),
                    0xEA => self.reg.d_reg = self.alu_set(self.reg.d_reg, 5),
                    0xEB => self.reg.e_reg = self.alu_set(self.reg.e_reg, 5),
                    0xEC => self.reg.h_reg = self.alu_set(self.reg.h_reg, 5),
                    0xED => self.reg.l_reg = self.alu_set(self.reg.l_reg, 5),
                    0xEE => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_set(b, 5);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xEF => self.reg.a_reg = self.alu_set(self.reg.a_reg, 5),
                    0xF0 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 6),
                    0xF1 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 6),
                    0xF2 => self.reg.d_reg = self.alu_set(self.reg.d_reg, 6),
                    0xF3 => self.reg.e_reg = self.alu_set(self.reg.e_reg, 6),
                    0xF4 => self.reg.h_reg = self.alu_set(self.reg.h_reg, 6),
                    0xF5 => self.reg.l_reg = self.alu_set(self.reg.l_reg, 6),
                    0xF6 => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_set(b, 6);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xF7 => self.reg.a_reg = self.alu_set(self.reg.a_reg, 6),
                    0xF8 => self.reg.b_reg = self.alu_set(self.reg.b_reg, 7),
                    0xF9 => self.reg.c_reg = self.alu_set(self.reg.c_reg, 7),
                    0xFA => self.reg.d_reg = self.alu_set(self.reg.d_reg, 7),
                    0xFB => self.reg.e_reg = self.alu_set(self.reg.e_reg, 7),
                    0xFC => self.reg.h_reg = self.alu_set(self.reg.h_reg, 7),
                    0xFD => self.reg.l_reg = self.alu_set(self.reg.l_reg, 7),
                    0xFE => {
                        let a = self.reg.parse_hl();
                        let b = self.mem.borrow().get(a);
                        let c = self.alu_set(b, 7);
                        self.mem.borrow_mut().set(a, c);
                    }
                    0xFF => self.reg.a_reg = self.alu_set(self.reg.a_reg, 7),
                }
            }
            
            0xD3 => panic!("Mem adress not used"),
            0xE3 => panic!("Mem adress not used"),
            0xE4 => panic!("Mem adress not used"),
            0xF4 => panic!("Mem adress not used"),
            0xDB => panic!("Mem adress not used"),
            0xDD => panic!("Mem adress not used"),
            0xEB => panic!("Mem adress not used"),
            0xEC => panic!("Mem adress not used"),
            0xED => panic!("Mem adress not used"),
            0xFC => panic!("Mem adress not used"),
            0xFD => panic!("Mem adress not used"),
        };

        let ecycle = match opcode {
            0x20 | 0x30 => {
                if self.reg.get_flag(ZeroFlag) { 0x00 } else { 0x01 }
            }    
            0x28 | 0x38 => {
                if self.reg.get_flag(ZeroFlag) { 0x01 } else { 0x00 }
            }
            0xC0 | 0xD0 => {
                if self.reg.get_flag(ZeroFlag) { 0x00 } else { 0x03 }
            }
            0xC8 | 0xCC | 0xD8 | 0xDC => {
                if self.reg.get_flag(ZeroFlag) { 0x03 } else { 0x00 }
            }
            0xC2 | 0xD2 => {
                if self.reg.get_flag(ZeroFlag) { 0x00 } else { 0x01 }
            }
            0xCA | 0xDA => {
                if self.reg.get_flag(ZeroFlag) { 0x01 } else { 0x00 }
            }
            0xC4 | 0xD4 => {
                if self.reg.get_flag(ZeroFlag) { 0x00 } else { 0x03 }
            }
            _ => 0x00,
        };

        if opcode == 0xCB {
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
            let s = u64::from(STEP_TIME.saturating_add(d.as_millis() as u32));
            dbg!("Sleep {:?}", s);
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