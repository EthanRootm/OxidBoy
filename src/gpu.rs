use super::terms::Term;
use super::intf::{Flags, Intf};
use super::mem::Memory;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(PartialEq, Eq)]
pub enum HdmaMode {
    Gdma,
    Hdma,
}
pub struct Hdma {
    pub src: u16,
    pub dst: u16,
    pub active: bool,
    pub mode: HdmaMode,
    pub remain: u8,
}
impl Hdma {
    pub fn power_up() -> Self {
        Self { src: 0x0000, dst: 0x8000, active: false, mode: HdmaMode::Gdma, remain: 0x00 }
    }
}
impl Memory for Hdma {
    fn get(&self, a: u16) -> u8 {
        match a {
            0xFF51 => (self.src >> 8) as u8,
            0xFF52 => self.src as u8,
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
#[rustfmt::skip]
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
    fn from(u: u8) -> Self {
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
    pub term: Term,
    pub h_blank: bool,
    pub v_blank: bool,

    lcdc: Lcdc,
    stat: Stat,

    sy: u8,
    sx: u8,

    wy: u8,
    wx: u8,

    ly: u8,
    lc: u8,

    bgp: u8,
    op0: u8,
    op1: u8,

    cbgpi: Bgpi,

    cbgpd: [[[u8; 3]; 4]; 8],

    cobpi: Bgpi,
    cobpd: [[[u8; 3]; 4]; 8],

    ram: [u8; 0x4000],
    ram_bank: usize,

    oam: [u8; 0xA0],

    prio: [(bool, usize); SCREEN_W],

    dots: u32,
}

impl Gpu {
    pub fn power_up(term: Term, intf: Rc<RefCell<Intf>>) -> Self {
        Self { 
            data: [[[0xffu8; 3]; SCREEN_W]; SCREEN_H],
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
            cbgpi: Bgpi::power_up(),
            cbgpd: [[[0u8; 3]; 4]; 8],
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
        let r = u32::from(r);
        let g = u32::from(g);
        let b = u32::from(b);
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
            if i == (c - 1) {
                self.dots += cycles % 80
            } else {
                self.dots += 80
            }
            let d = self.dots;
            self.dots %= 456;
            if d != self.dots {
                self.ly = (self.ly + 1) % 154;
                if self.stat.ly_interrupt && self.ly == self.lc {
                    self.intf.borrow_mut().hi(Flags::LCDStat);
                }
            }
            if self.ly >= 144 {
                if self.stat.mode == 1 {
                    continue;
                }
                self.stat.mode = 1;
                self.v_blank = true;
                self.intf.borrow_mut().hi(Flags::Vblank);
                if self.stat.m1_interrupt {
                    self.intf.borrow_mut().hi(Flags::LCDStat);
                }
            } else if self.dots <= 80 {
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
                // Render scanline
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
        let show_window = self.lcdc.bit5() && self.wy <= self.ly;
        let tile_base = if self.lcdc.bit4() { 0x8000 } else { 0x8800 };

        let wx = self.wx.wrapping_sub(7);
        let py = if show_window { self.ly.wrapping_sub(self.wy) } else { self.sy.wrapping_add(self.ly) };
        let ty = (u16::from(py) >> 3) & 31;

        for x in 0..SCREEN_W {
            let px = if show_window && x as u8 >= wx { x as u8 - wx } else { self.sx.wrapping_add(x as u8) };
            let tx = (u16::from(px) >> 3) & 31;

            let bg_base = if show_window && x as u8 >= wx {
                if self.lcdc.bit6() {
                    0x9C00
                } else {
                    0x9800
                }
            } else if self.lcdc.bit3() {
                0x9C00
            } else {
                0x9800
            };

        let tile_addr = bg_base + ty * 32 + tx;
            let tile_number = self.get_ram0(tile_addr);
            let tile_offset = if self.lcdc.bit4() {
                i16::from(tile_number)
            } else {
                i16::from(tile_number as i8) + 128
            } as u16 * 16;
            let tile_location = tile_base + tile_offset;
            let tile_attr = Attr::from(self.get_ram1(tile_addr));

            let tile_y = if tile_attr.yflip { 7 - py % 8 } else { py % 8 };
            let tile_y_data: [u8; 2] = if self.term == Term::GBC && tile_attr.bank {
                let a = self.get_ram1(tile_location + u16::from(tile_y * 2));
                let b = self.get_ram1(tile_location + u16::from(tile_y * 2) + 1);
                [a, b]
            } else {
                let a = self.get_ram0(tile_location + u16::from(tile_y * 2));
                let b = self.get_ram0(tile_location + u16::from(tile_y * 2) + 1);
                [a, b]
            };
            let tile_x = if tile_attr.xflip { 7 - px % 8 } else { px % 8 };

            let color_l = if tile_y_data[0] & (0x80 >> tile_x) != 0 { 1 } else { 0 };
            let color_h = if tile_y_data[1] & (0x80 >> tile_x) != 0 { 2 } else { 0 };
            let color = color_h | color_l;
                

            self.prio[x] = (tile_attr.priority, color);
                

            if self.term == Term::GBC {
                let r = self.cbgpd[tile_attr.palette_num_1][color][0];
                let g = self.cbgpd[tile_attr.palette_num_1][color][1];
                let b = self.cbgpd[tile_attr.palette_num_1][color][2];
                self.set_rgb(x as usize, r, g, b);
            } else {
                let color = Self::get_gray_shaders(self.bgp, color) as u8;
                self.set_gre(x, color);
            }
        }
    }
        
    fn draw_sprites(&mut self) {
        let sprite_size = if self.lcdc.bit2() { 16 } else { 8 };
        for i in 0..40 {
            let sprite_addr = 0xFE00 + (i as u16) * 4;
            let py = self.get(sprite_addr).wrapping_sub(16);
            let px = self.get(sprite_addr + 1).wrapping_sub(8);
            let tile_number = self.get(sprite_addr + 2) & if self.lcdc.bit2() { 0xFE } else { 0xFF };
            let tile_attr = Attr::from(self.get(sprite_addr + 3));

            if py <= 0xFF - sprite_size + 1 {
                if self.ly < py || self.ly > py + sprite_size - 1 {
                    continue;
                }
            } else {
                if self.ly > py.wrapping_add(sprite_size) - 1 {
                    continue;
                }
            }
            if px >= (SCREEN_W as u8) && px <= (0xFF - 7) {
                continue;
            }

            let tile_y = if tile_attr.yflip { sprite_size - 1 - self.ly.wrapping_sub(py) } else { self.ly.wrapping_sub(py) };
            let tile_y_addr = 0x8000u16 + u16::from(tile_number) * 16 + u16::from(tile_y) * 2;
            let tile_y_data: [u8; 2] = if self.term == Term::GBC && tile_attr.bank {
                let b1 = self.get_ram1(tile_y_addr);
                let b2 = self.get_ram1(tile_y_addr + 1);
                [b1, b2]
            } else {
                let b1 = self.get_ram0(tile_y_addr);
                let b2 = self.get_ram0(tile_y_addr + 1);
                [b1, b2]
            };

            for x in 0..8 {
                if px.wrapping_add(x) >= (SCREEN_W as u8) {
                    continue;
                }
                let tile_x = if tile_attr.xflip { 7 - x } else { x };

                let color_l = if tile_y_data[0] & (0x80 >> tile_x) != 0 { 1 } else { 0 };
                let color_h = if tile_y_data[1] & (0x80 >> tile_x) != 0 { 2 } else { 0 };
                let color = color_h | color_l;
                if color == 0 {
                    continue;
                }

                let prio = self.prio[px.wrapping_add(x) as usize];
                let skip = if self.term == Term::GBC && !self.lcdc.bit0() {
                    prio.1 == 0
                } else if prio.0 {
                    prio.1 != 0
                } else {
                    tile_attr.priority && prio.1 != 0
                };
                if skip {
                    continue;
                }

                if self.term == Term::GBC {
                    let r = self.cobpd[tile_attr.palette_num_1][color][0];
                    let g = self.cobpd[tile_attr.palette_num_1][color][1];
                    let b = self.cobpd[tile_attr.palette_num_1][color][2];
                    self.set_rgb(px.wrapping_add(x) as usize, r, g, b);
                } else {
                    let color = if tile_attr.palette_num_0 == 1 {
                        Self::get_gray_shaders(self.op1, color) as u8
                    } else {
                        Self::get_gray_shaders(self.op0, color) as u8
                    };
                    self.set_gre(px.wrapping_add(x) as usize, color);
                }
            }
        }
    }
}

impl Memory for Gpu {
    fn get(&self, a: u16) -> u8 {
        match a {
            0x8000..=0x9FFF => self.ram[self.ram_bank * 0x2000 + a as usize - 0x8000],
            0xFE00..=0xFE9F => self.oam[a as usize - 0xFE00],
            0xFF40 => self.lcdc.data,
            0xFF41 => {
                let bit6 = if self.stat.ly_interrupt { 0x40 } else { 0x00 };
                let bit5 = if self.stat.m2_interrupt { 0x20 } else { 0x00 };
                let bit4 = if self.stat.m1_interrupt { 0x10 } else { 0x00 };
                let bit3 = if self.stat.m0_interrupt { 0x08 } else { 0x00 };
                let bit2 = if self.ly == self.lc { 0x04 } else { 0x00 };
                bit6 | bit5 | bit4 | bit3 | bit2 | self.stat.mode
            }
            0xFF42 => self.sy,
            0xFF43 => self.sx,
            0xFF44 => self.ly,
            0xFF45 => self.lc,
            0xFF47 => self.bgp,
            0xFF48 => self.op0,
            0xFF49 => self.op1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            0xFF4F => 0xFE | self.ram_bank as u8,
            0xFF68 => self.cbgpi.get(),
            0xFF69 => {
                let r = self.cbgpi.i as usize >> 3;
                let c = self.cbgpi.i as usize >> 1 & 0x3;
                if self.cbgpi.i & 0x01 == 0x00 {
                    let a = self.cbgpd[r][c][0];
                    let b = self.cbgpd[r][c][1] << 5;
                    a | b
                } else {
                    let a = self.cbgpd[r][c][1] >> 3;
                    let b = self.cbgpd[r][c][2] << 2;
                    a | b
                }
            }
            0xFF6A => self.cobpi.get(),
            0xFF6B => {
                let r = self.cobpi.i as usize >> 3;
                let c = self.cobpi.i as usize >> 1 & 0x3;
                if self.cobpi.i & 0x01 == 0x00  {
                    let a = self.cobpd[r][c][0];
                    let b = self.cobpd[r][c][1] << 5;
                    a | b
                } else {
                    let a = self.cobpd[r][c][1] >> 3;
                    let b = self.cobpd[r][c][2] << 2;
                    a | b
                }
            }
            _ => panic!(""),
        }
    }

    fn set(&mut self, a: u16, v: u8) {
        match a {
            0x8000..=0x9FFF => self.ram[self.ram_bank * 0x2000 + a as usize - 0x8000] = v,
            0xFE00..=0xFE9F => self.oam[a as usize - 0xFE00] = v,
            0xFF40 => {
                self.lcdc.data = v;
                if !self.lcdc.bit7() {
                    self.dots = 0;
                    self.ly = 0;
                    self.stat.mode = 0;
                    self.data = [[[0xffu8; 3]; SCREEN_W]; SCREEN_H];
                    self.v_blank = true;
                }
            }
            0xFF41 => {
                self.stat.ly_interrupt = v & 0x40 != 0x00;
                self.stat.m2_interrupt = v & 0x20 != 0x00;
                self.stat.m1_interrupt = v & 0x10 != 0x00;
                self.stat.m0_interrupt = v & 0x08 != 0x00;
            }
            0xFF42 => self.sy = v,
            0xFF43 => self.sx = v,
            0xFF44 => {}
            0xFF45 => self.lc = v,
            0xFF47 => self.bgp = v,
            0xFF48 => self.op0 = v,
            0xFF49 => self.op1 = v,
            0xFF4A => self.wy = v,
            0xFF4B => self.wx = v,
            0xFF4F => self.ram_bank = (v & 0x01) as usize,
            0xFF68 => self.cbgpi.set(v),
            0xFF69 => {
                let r = self.cbgpi.i as usize >> 3;
                let c = self.cbgpi.i as usize >> 1 & 0x03;
                if self.cbgpi.i & 0x01 == 0x00 {
                    self.cbgpd[r][c][0] = v & 0x1F;
                    self.cbgpd[r][c][1] = (self.cbgpd[r][c][1] & 0x18) | (v >> 5);
                } else {
                    self.cbgpd[r][c][1] = (self.cbgpd[r][c][1] & 0x07) | ((v & 0x03) << 3);
                    self.cbgpd[r][c][2] = (v >> 2) & 0x1F;
                }
                if self.cbgpi.auto_increment {
                    self.cbgpi.i += 0x01;
                    self.cbgpi.i &= 0x3F;
                }
            }
            0xFF6A => self.cobpi.set(v),
            0xFF6B => {
                let r = self.cobpi.i as usize >> 3;
                let c = self.cobpi.i as usize >> 1 & 0x03;
                if self.cobpi.i & 0x01 == 0x00 {
                    self.cobpd[r][c][0] = v & 0x1F;
                    self.cobpd[r][c][1] = (self.cobpd[r][c][1] & 0x18) | (v >> 5);
                } else {
                    self.cobpd[r][c][1] = (self.cobpd[r][c][1] & 0x07) | ((v & 0x03) << 3);
                    self.cobpd[r][c][2] = (v >> 2) & 0x1F;
                }
                if self.cobpi.auto_increment {
                    self.cobpi.i += 0x01;
                    self.cobpi.i &= 0x3F;
                }
            }
            _ => panic!(""),
        }
    }
}