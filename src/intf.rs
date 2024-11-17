#[rustfmt::skip]
#[derive(Clone)]
pub enum Flags {
    Vblank = 0,
    LCDStat = 1,
    Timer = 2,
    Serial = 3,
    Joypad = 4,
}

pub struct Intf {
    pub data: u8,
}

impl intf {
    pub fn power_up() -> Self {
        Self { data: 0x00 }
    }

    pub fn hi(&mut self, flag: Flags) {
        self.data |= 1 << flag as u8;
    }
}