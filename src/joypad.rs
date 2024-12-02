use super::intf::{Flags, Intf};
use super::mem::Memory;
use std::cell::RefCell;
use std::rc::Rc;

#[rustfmt::skip]
#[derive(Clone)]
pub enum Key {
    Right = 0b0000_0001,
    Left = 0b0000_0010,
    Up = 0b0000_0100,
    Down = 0b0000_1000,
    A = 0b0001_0000,
    B = 0b0010_0000,
    Select = 0b0100_0000,
    Start = 0b1000_0000,
}

pub struct Joypad {
    intf: Rc<RefCell<Intf>>,
    matrix: u8,
    select: u8,
}

impl Joypad {
    pub fn power_up(intf: Rc<RefCell<Intf>>) -> Self {
      Self { intf, matrix: 0xFF, select: 0x00 }  
    }
}

impl Joypad {
    pub fn keyup(&mut self, key: Key) {
        self.matrix |= key as u8;
    }

    pub fn keydown(&mut self, key: Key) {
       self.matrix &= !(key as u8);
       self.intf.borrow_mut().hi(Flags::Joypad); 
    }
}

impl Memory for Joypad {
    fn get(&self, a: u16) -> u8 {
        assert_eq!(a, 0xFF00);
        if (self.select & 0b0001_0000) == 0x00 {
            return self.select | (self.matrix & 0x0F);
        }
        if (self.select & 0b0010_0000) == 0x00 {
            return self.select | (self.matrix >> 4);
        }
        self.select
    }

    fn set(&mut self, a: u16, v: u8) {
        assert_eq!(a, 0xFF00);
        self.select = v;
    }
}