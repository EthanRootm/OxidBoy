use super::clock::Clock;
use super::intf::{Flags, Intf};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
struct Register {
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8
}

pub struct Timer {
    intf: Rc<RefCell<Intf>>,
    reg: Register,
    div_clock: Clock,
    tma_clock: Clock,
}

impl Timer {
    pub fn power_up(intf: Rc<RefCell<Intf>>) -> Self {
        Timer { intf, reg: Register::default(), div_clock: Clock::power_up(256), tma_clock: Clock::power_up(1024) }
    }

    pub fn get(&self, a: u16) -> u8 {
        match a {
            0xFF04 => self.reg.div,
            0xFF05 => self.reg.tima,
            0xFF06 => self.reg.tma,
            0xFF07 => self.reg.tac,
            _ => panic!("Unsupported address"),
        }
    }
    
    pub fn set(&mut self, a: u16, v: u8) {
        match a {
            0xFF04 => {
                self.reg.div = 0x00;
                self.div_clock.n = 0x00;
            }
            0xFF05 => self.reg.tima = v,
            0xFF06 => self.reg.tma = v,
            0xFF07 => {
                if (self.reg.tac & 0x03) != (v & 0x03) {
                    self.tma_clock.n = 0x00;
                    self.tma_clock.period = match v & 0x03 {
                        0x00 => 1024,
                        0x01 => 16,
                        0x02 => 64,
                        0x03 => 256,
                        _ => panic!(""),
                    };
                    self.reg.tima = self.reg.tma;
                }
                self.reg.tac = v;
            }
            _ => panic!("Unsupported address"),
        }
    }

    pub fn next(&mut self, cycle: u32) {
        self.reg.div = self.reg.div.wrapping_add(self.div_clock.next(cycle) as u8);


        if (self.reg.tac & 0x04) != 0x00 {
            let n = self.tma_clock.next(cycle);
            for _ in 0..n {
                self.reg.tima = self.reg.tima.wrapping_add(1);
                if self.reg.tima == 0x00 {
                    self.reg.tima = self.reg.tma;
                    self.intf.borrow_mut().hi(Flags::Timer);
                }
            }
        }
    }
}