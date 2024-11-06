use super::terms::Term;
#[derive(Clone, Default)]
/// Struct of all registers to the cpu
pub struct Register {
    ///accumulator register
    pub a_reg: u8,
    ///flag register
    pub f_reg: u8,
    pub b_reg: u8,
    pub c_reg: u8,
    pub d_reg: u8,
    pub e_reg: u8,
    pub h_reg: u8,
    pub l_reg: u8,
    pub stack_pointer: u16,
    pub program_counter: u16,
}

impl Register {
    
    /// returns a 16 bit value with a as the upper byte and f as the lower byte
    ///  * 'self' - the cpu registers 
    pub fn parse_af(&self) -> u16 {
        (u16::from(self.a_reg) << 8) | u16::from(self.f_reg)
    }
    /// returns a 16 bit value with b as the upper byte and c as the lower byte
    ///  * 'self' - the cpu registers 
    pub fn parse_bc(&self) -> u16 {
        (u16::from(self.b_reg) << 8) | u16::from(self.c_reg)
    }
    /// returns a 16 bit value with d as the upper byte and e as the lower byte
    ///  * 'self' - the cpu registers 
    pub fn parse_de(&self) -> u16 {
        (u16::from(self.d_reg) << 8) | u16::from(self.e_reg)
    }
    /// returns a 16 bit value with h as the upper byte and l as the lower byte
    ///  * 'self' - the cpu registers 
    pub fn parse_hl(&self) -> u16 {
        (u16::from(self.h_reg) << 8) | u16::from(self.l_reg)
    }
    /// Sets the the A and F registers based on the 16-bit input registers
    /// * mut self: Important for setting the registers to their correct values
    /// * reg: The 16-bit register that gets set to both child registers
    pub fn set_af(&mut self, reg: u16) {
        self.a_reg = (reg >> 8) as u8;
        self.f_reg = (reg & 0x00F0) as u8;
    }
    /// Sets the the B and C registers based on the 16-bit input registers
    /// * mut self: Important for setting the registers to their correct values
    /// * reg: The 16-bit register that gets set to both child registers
    pub fn set_bc(&mut self, reg: u16) {
        self.b_reg = (reg >> 8) as u8;
        self.c_reg = (reg & 0x00FF) as u8;
    }
    /// Sets the the D and E registers based on the 16-bit input registers
    /// * mut self: Important for setting the registers to their correct values
    /// * reg: The 16-bit register that gets set to both child registers
    pub fn set_de(&mut self, reg: u16) {
        self.d_reg = (reg >> 8) as u8;
        self.e_reg = (reg & 0x00FF) as u8;
    }
    /// Sets the the H and L registers based on the 16-bit input registers
    /// * mut self: Important for setting the registers to their correct values
    /// * reg: The 16-bit register that gets set to both child registers
    pub fn set_hl(&mut self, reg: u16) {
        self.h_reg = (reg >> 8) as u8;
        self.l_reg = (reg & 0x00FF) as u8;
    }
}
pub enum Flags {
    ///This bit is set only when the resulting operation is zero
    ZeroFlag = 0b1000_0000,
    ///Indicates that the previous instruction was a subtraction
    SubtractionFlag = 0b0100_0000,
    ///Indicates that the upper 4 bits were carried
    HalfCarryFlag = 0b0010_0000,
    ///Set if result of 8-bit addition is higher than 0xFF or 0xFFFF for 16-bit addition or result of subtraction is < 0
    CarryFlag = 0b0001_0000,
}

impl Flags {
    fn orgin(self) -> u8 {
        self as u8
    }

    fn inverse(self) -> u8 {
        !self.og()
    }
}

impl Register {
    pub fn get_flag(&self, flag: Flags) -> bool {
        (self.f_reg & flag as u8) != 0
    }

    pub fn set_flag(&self, flag: Flags, v: bool) {
        if v {
            self.f_reg |= flag.orgin();
        } else {
            self.f_reg &= flag.inverse();
        }
    }
}

impl Register {
    ///Sets the power up sequence of the gameboy for specifically the Resisters and flags
    /// * Returns the state of Registers after the powerup sequence has been finished
    pub fn power_up(term: Term) -> Self {
        let mut registers = Self::default();
        registers.a_reg = match term {
            Term::GB => 0x01,
            Term::GBP => 0xFF,
            Term::GBC => 0x11,
            Term::SGB => 0x01,
        };

        registers.f_reg = 0xB0;
        registers.b_reg = 0x00;
        registers.c_reg = 0x13;
        registers.d_reg = 0x00;
        registers.e_reg = 0xD8;
        registers.h_reg = 0x01;
        registers.l_reg = 0x4D;
        registers.program_counter = 0x0100;
        registers.stack_pointer = 0xFFFFE;
        registers
    }
}

