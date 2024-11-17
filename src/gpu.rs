use super::terms::Term;
use super::intf::{Flags, Intf};
use super::mem::Memory;
use std::cell::RefCell;
use std::rc::Rc;

pub enum HdmaMode {
    Gdma,
    Hdma,
}
pub struct Hdma{
    pub src: u16,
    pub dst: u16,
    pub active: bool,
    pub mode: HdmaMode,
    pub remain: u8,
}
impl Hdma {
    pub fn power_up() -> Self {
        Self { src: 0x0000, dst: 0x8000, active: false, mode:HdmaMode::Gdma, remain: 0x00 }
    }
}
impl Memory for Hdma {
    fn get(&self, a: u16) -> u8 {
        match a {
            0xFF51 => (self.src >> 8) as u8,
            0xFF52 =>  self.src as u8,
            0xFF53 => (self.dst >> 8) as u8,
            0xFF54 => self.dst as u8,
            0xFF55 => self.remain | if self.active { 0x00 } else { 0x80 },
            _ => panic!(""),
        }
    }

    fn set(&mut self, a: u16, v: u8) {
        match a {
            0xFF51 => self.src = (u16::from(v) << 8) | (self.src & 0x00FF),
            0xFF52 => self.src = (self.src & 0xFF00) | u16::from(v & 0xF0),
            0xFF53 => self.dst = 0x8000 | (u16::from(v & 0x1F) << 8) | (self.dst & 0x00FF),
            0xFF54 => self.dst = (self.dst & 0xFF00) | u16::from(v & 0xF0),
            0xFF55 => {
                if self.active && self.mode == HdmaMode::Hdma {
                    if v & 0x80 == 0x00 {
                        self.active = false;
                    };
                return;
            }
            self.active = true;
            self.remain = v & 0x7F;
            self.mode = if v & 0x80 != 0x00 { HdmaMode::Hdma } else { HdmaMode:: Gdma};
        }
        _ => panic!(""),
        };
    }
}

pub struct Lcdc {
    data: u8,
}
impl Lcdc {
    pub fn power_up() -> Self {
        Self { data: 0b0100_1000 }
    }

    fn bit7(&self) -> bool {
    self.data & 0b1000_0000 != 0x00       
    }

    fn bit6(&self) -> bool {
        self.data & 0b0100_0000 != 0x00       
    }

    fn bit5(&self) -> bool {
        self.data & 0b0010_0000 != 0x00       
    }

    fn bit4(&self) -> bool {
        self.data & 0b0001_0000 != 0x00       
    }

    fn bit3(&self) -> bool {
        self.data & 0b0000_1000 != 0x00       
    }

    fn bit2(&self) -> bool {
        self.data & 0b0000_0100 != 0x00       
    }

    fn bit1(&self) -> bool {
        self.data & 0b0000_0010 != 0x00       
    }

    fn bit0(&self) -> bool {
        self.data & 0b0000_0001 != 0x00       
    }
}

pub struct Stat {
    ly_interrupt: bool,
    m2_interrupt: bool,
    m1_interrupt: bool,
    m0_interrupt: bool,
    mode: u8,
}

impl Stat {
    pub fn power_up() -> Self {
        Self { ly_interrupt: false, m2_interrupt: false, m1_interrupt: false, m0_interrupt: false, mode: 0x00 }
    }
}

struct Bgpi {
    i: u8,
    auto_increment: bool,
}

impl Bgpi {
    fn power_up() -> Self {
        Self { i: 0x00, auto_increment: false }
    }

    fn get(&self) -> u8 {
        let a = if self.auto_increment { 0x80 } else { 0x00 };
        a | self.i
    }

    fn set(&mut self, v: u8) {
        self.auto_increment = v & 0x80 != 0x00;
        self.i = v & 0x3F;
    }
}

pub enum GrayShades {
    White = 0xFF,
    Light = 0xC0,
    Dark = 0x60,
    Black = 0x00,
}

struct Attr {
    priority: bool,
    yflip: bool,
    xflip: bool,
    palette_num_0: usize,
    bank: bool,
    palette_num_1: usize,
}

impl From<u8> for Attr {
    fn from(value: u8) -> Self {
        Self 
        { 
            priority: u & (1 << 7) != 0,
            yflip: u & (1 << 6) != 0,
            xflip: u & (1 << 5) != 0,
            palette_num_0: u as usize & (1 << 4),
            bank: u & (1 << 3) != 0,
            palette_num_1: u as usize & 0x07, 
        }
    }
}

pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;

pub struct Gpu {
    pub data: [[[u8; 3]; SCREEN_W]; SCREEN_H],
    pub intf: Rc<RefCell<Intf>>,
    pub term: terms,
    pub h_blank: bool,
    pub v_blank: bool,

    lcdc: Lcdc,
    stat:Stat,

    sy: u8,
    sx: u8,

    wy: u8,
    wx: u8,

    ly: u8,
    lc: u8,

    bgp: u8,
    op0: u8,
    op1: u8,

    chgpi: Bgpi,

    chgpd: [[[u8; 3]; 4]; 8],

    cobpi: Bgpi,
    cobpd: [[[u8; 3]; 4]; 8],

    ram: [u8; 0x4000],
    ram_bank: usize,

    oam: [u8; 0xA0],

    prio: [(bool, usize); SCREEN_W],

    dots: u32,
}

impl Gpu {
    pub fn power_up(term : Term, intf: Rc<RefCell<Intf>>) -> Self {
        Self { 
            data: [[[0xFFu8; 3]; SCREEN_W]; SCREEN_H],
            intf,
            term,
            h_blank: false,
            v_blank: false,
            lcdc: Lcdc::power_up(),
            stat: Stat::power_up(),
            sy: 0x00,
            sx: 0x00,
            wy: 0x00,
            wx: 0x00,
            ly: 0x00,
            lc: 0x00,
            bgp: 0x00,
            op0: 0x00,
            op1: 0x01,
            chgpi: Bgpi::power_up(),
            chgpd: [[[0u8; 3]; 4]; 8],
            cobpi: Bgpi::power_up(),
            cobpd: [[[0u8; 3]; 4]; 8],
            ram: [0x00; 0x4000], 
            ram_bank: 0x00,
            oam: [0x00; 0xA0],
            prio: [(true, 0); SCREEN_W],
            dots: 0,
        }
    }

    fn get_ram0(&self, a: u16) -> u8 {
        self.ram[a as usize - 0x8000]
    }

    fn get_ram1(&self, a: u16) -> u8 {
        self.ram[a as usize - 0x6000]
    }

    fn get_gray_shaders(v: u8, i: usize) -> GrayShades {
        match v >> (2 * i) & 0x03 {
            0x00 => GrayShades::White,
            0x01 => GrayShades::Light,
            0x02 => GrayShades::Dark,
            _ => GrayShades::Black,
        }
    }
    
    fn set_gre(&mut self, x: usize, g: u8) {
        self.data[self.ly as usize][x] = [g, g, g];
    }

    fn set_rgb(&mut self, x: usize, r: u8, g: u8, b: u8) {
        assert!(r <= 0x1F);
        assert!(g <= 0x1F);
        assert!(b <= 0x1F);
        let r = u16::from(r);
        let g = u16::from(g);
        let b = u16::from(b);
        let lr = ((r * 13 + g * 2 + b) >> 1) as u8;
        let lg = ((g * 3 + b) << 1) as u8;
        let lb = ((r * 3 + g * 2 + b * 11) >> 1) as u8;
        self.data[self.ly as usize][x] = [lr, lg, lb];
    }

    pub fn next(&mut self, cycles: u32) {
        if !self.lcdc.bit7() {
            return;
        }
        self.h_blank = false;

        if cycles == 0 {
            return;
        }
        let c = (cycles - 1) / 80 + 1;
        for i in 0..c {
            if i == (c - 1){
                self.dots += cycles % 80
            } else {
                self.dots += 80
            }
            let d = self.dots;
            self.dots %= 456;
            if d != self.dots {
                self.ly = (self.ly + 1) % 154;
                if self.stat.ly_interrupt && self.ly == self.lc {
                    self.intf.borrow_mut().hi(Flag::LCDStat);
                }
            }
            if self.ly >= 144 {
                if self.stat.mode == 1 {
                    continue;
                }
                self.stat.mode = 1;
                self.v_blank = true;
                self.intf.borrow_mut().hi(Flag::VBlank);
                if self.stat.m1_interrupt {
                    self.intf.borrow_mut().hi(Flag::LCDStat);
                }
            }else if self.dots <= 80 {
                    if self.stat.mode == 2 {
                        continue;
                    }
                    self.stat.mode = 2;
                    if self.stat.m2_interrupt {
                        self.intf.borrow_mut().hi(Flags::LCDStat);
                    }
                } else if self.dots <= (80 + 172) {
                    self.stat.mode = 3;
                } else {
                    if self.stat.mode == 0 {
                        continue;
                    }
                    self.stat.mode = 0;
                    self.h_blank = true;
                    if self.stat.m0_interrupt {
                        self.intf.borrow_mut().hi(Flags::LCDStat);
                    }
                    if self.term == Term::GBC || self.lcdc.bit0() {
                        self.draw_bg();
                    }
                    if self.lcdc.bit1() {
                        self.draw_sprites();
                    }
                }
            }
        }

        fn draw_bg(&mut self) {
            todo!()
        }
        
        fn draw_sprites(&mut self) {
            todo!()
        }
}