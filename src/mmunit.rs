//use super::apu::Apu;
use super::cartridge::{self, Cartridge};
use super::terms::Term;
use super::gpu::{Gpu, Hdma, HdmaMode};
use super::intf::Intf;
//use super::joypad::Joypad;
use super::linkcable::Serial;
use super::mem::Memory;
use super::timer::Timer;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

#[derive(Clone,Copy, PartialEq, Eq)]
pub enum Speed {
    Normal = 0x01,
    Double = 0x02,
}

pub struct Mmunit {
    pub cartridge: Box<dyn Cartridge>,
    pub gpu: Gpu,
    pub serial: Serial,
    pub shift: bool,
    pub speed: Speed,
    pub term: Term,
    pub time: Timer,
    inte: u8,
    intf: Rc<RefCell<Intf>>,
    hdma: Hdma,
    hram: [u8; 0x7F],
    wram: [u8; 0x8000],
    wram_bank: usize,
}

impl Mmunit {
    pub fn power_up(path: impl AsRef<Path>) -> Self {
        let cart = cartridge::power_up(path);
        let term = match cart.get(0x0143) & 0x80 {
            0x80 => Term::GBC,
            _ => Term::GB,
        };
        let intf = Rc::new(RefCell::new(Intf::power_up()));
        let mut r = Self { 
            cartridge: cart,
            gpu: Gpu::power_up(term, intf.clone()),
            serial: Serial::power_up(intf.clone()),
            shift: false,
            speed: Speed::Normal,
            term,
            time: Timer::power_up(intf.clone()),
            inte: 0x00,
            intf: intf.clone(),
            hdma: Hdma::power_up(),
            hram: [0x00; 0x7F],
            wram: [0x00; 0x8000],
            wram_bank: 0x01,
        };
        r.set(0xFF05, 0x00);
        r.set(0xFF06, 0x00);
        r.set(0xFF07, 0x00);
        r.set(0xFF10, 0x80);
        r.set(0xFF11, 0xBF);
        r.set(0xFF12, 0xF3);
        r.set(0xFF14, 0xBF);
        r.set(0xFF16, 0x3F);
        r.set(0xFF17, 0x00);
        r.set(0xFF19, 0xBF);
        r.set(0xFF1A, 0x7F);
        r.set(0xFF1B, 0xFF);
        r.set(0xFF1C, 0x9F);
        r.set(0xFF1E, 0xFF);
        r.set(0xFF20, 0xFF);
        r.set(0xFF21, 0x00);
        r.set(0xFF22, 0x00);
        r.set(0xFF23, 0xBF);
        r.set(0xFF24, 0x77);
        r.set(0xFF25, 0xF3);
        r.set(0xFF26, 0xF1);
        r.set(0xFF40, 0x91);
        r.set(0xFF42, 0x00);
        r.set(0xFF43, 0x00);
        r.set(0xFF45, 0x00);
        r.set(0xFF47, 0xFC);
        r.set(0xFF48, 0xFF);
        r.set(0xFF49, 0xFF);
        r.set(0xFF4A, 0x00);
        r.set(0xFF4B, 0x00);
        r
    }
}

impl Mmunit {
    pub fn next(&mut self, cycles: u32) -> u32 {
        let cpu_divider = self.speed as u32;
        let vram_cycles = self.run_dma();
        let gpu_cycles = cycles /cpu_divider + vram_cycles;
        let cpu_cycles = cycles + vram_cycles * cpu_divider;
        self.time.next(cpu_cycles);
        self.gpu.next(gpu_cycles);
        gpu_cycles
    }

    pub fn switch_speed(&mut self) {
        if self.shift {
            if self.speed == Speed::Double {
                self.speed = Speed::Normal;
            } else {
                self.speed = Speed::Double;
            }
        }
        self.shift = false;
    }

    fn run_dma(&mut self) -> u32 {
        if !self.hdma.active { return 0; }
        match self.hdma.mode {
            HdmaMode::Gdma => {
                let len = u32::from(self.hdma.remain) + 1;
                for _ in 0..len {
                    self.run_dma_hrampart();
                }
                self.hdma.active = false;
                len * 8
            }
            HdmaMode::Hdma => {
                if !self.gpu.h_blank { return 0; }
                self.run_dma_hrampart();
                if self.hdma.remain == 0x7F {
                    self.hdma.active = false;
                }
                8
            }
        }
    }

    fn run_dma_hrampart(&mut self) {
        let mmu_src = self.hdma.src;
        for i in 0..0x10 {
            let b: u8 = self.get(mmu_src + i);
            self.gpu.set(self.hdma.dst + i, b);
        }
        self.hdma.src += 0x10;
        self.hdma.dst += 0x10;
        if self.hdma.remain == 0 {
            self.hdma.remain = 0x7F;
        } else {
            self.hdma.remain -= 1;
        }
    }
}

impl Memory for Mmunit {
    fn get(&self, a: u16) -> u8 {
        match a {
            0x0000..=0x7FFF => self.cartridge.get(a),
            0x8000..=0x9FFF => self.gpu.get(a),
            0xA000..=0xBFFF => self.cartridge.get(a),
            0xC000..=0xCFFF => self.wram[a as usize - 0xC000],
            0xD000..=0xDFFF => self.wram[a as usize - 0xD000 + 0x1000 * self.wram_bank],
            0xE000..=0xEFFF => self.wram[a as usize - 0xE000],
            0xF000..=0xFDFF => self.wram[a as usize - 0xF000 + 0x1000 * self.wram_bank],
            0xFE00..=0xFE9F => self.gpu.get(a),
            0xFEA0..=0xFEFF => 0x00,
            0xFF00 => todo!(),
            0xFF01..=0xFF02 => self.serial.get(a),
            0xFF04..=0xFF07 => self.time.get(a),
            0xFF0F => self.intf.borrow().data,
            0xFF10..=0xFF3F => todo!(),
            0xFF4D => {
                let a = if self.speed == Speed::Double { 0x80 } else { 0x00 };
                let b = if self.shift { 0x01 } else { 0x00 };
                a | b
            }
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF4F => self.gpu.get(a),
            0xFF51..=0xFF55 => self.hdma.get(a),
            0xFF68..=0xFF6B => self.gpu.get(a),
            0xFF70 => self.wram_bank as u8,
            0xFF80..=0xFFFE => self.hram[a as usize - 0xFF80],
            0xFFFF => self.inte,
            _ => 0x00
        }
    }

    fn set(&mut self, a: u16, v: u8) {
        match a {
            0x0000..=0x7FFF => self.cartridge.set(a, v),
            0x8000..=0x9FFF => self.gpu.set(a, v),
            0xA000..=0xBFFF => self.cartridge.set(a, v),
            0xC000..=0xCFFF => self.wram[a as usize - 0xC000] = v,
            0xD000..=0xDFFF => self.wram[a as usize - 0xD000 + 0x1000 * self.wram_bank] = v,
            0xE000..=0xEFFF => self.wram[a as usize - 0xE000] = v,
            0xF000..=0xFDFF => self.wram[a as usize - 0xF000 + 0x1000 * self.wram_bank] = v,
            0xFE00..=0xFE9F => self.gpu.set(a, v),
            0xFEA0..=0xFEFF => {}
            0xFF00 => todo!(),
            0xFF01..=0xFF02 => self.serial.set(a, v),
            0xFF04..=0xFF07 => self.time.set(a, v),
            0xFF10..=0xFF3F => todo!(),
            0xFF46 => {
                assert!(v <= 0xF1);
                let base = u16::from(v) << 8;
                for i in 0..0xA0 {
                    let b = self.get(base + i);
                    self.set(0xFE00 + i, b);
                }
            }
            0xFF4D => self.shift = (v & 0x01) == 0x01,
            0xFF40..=0xFF45 | 0xFF47..=0xFF4B | 0xFF4F => self.gpu.set(a, v),
            0xFF51..=0xFF55 => self.hdma.set(a, v),
            0xFF68..=0xFF6B => self.gpu.set(a, v),
            0xFF70 => {
                self.wram_bank = match v & 0x7 {
                    0 => 1,
                    n => n as usize,
                };
            }
            0xFF80..=0xFFFE => self.hram[a as usize - 0xFF80] = v,
            0xFFFF => self.inte = v,
            _ => {}
        }
    }
}