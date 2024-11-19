use super::intf::Intf;
use std::cell::RefCell;
use  std::rc::Rc;

pub struct Serial {
    intf: Rc<RefCell<Intf>>,
    data: u8,
    control: u8,
}

impl Serial {
    pub fn power_up(intf: Rc<RefCell<Intf>>) -> Self {
        Self { intf: intf, data: 0x00, control: 0x00 }
    }

    pub fn get(&self, a: u16) -> u8 {
        match a {
            0xFF01 => self.data,
            0xFF02 => self.control,
            _ => panic!("Not supported data")
        }
    }
    pub fn set(&mut self, a: u16, v: u8) {
        match a {
            0xFF01 => self.data = v,
            0xFF02 => self.control = v,
            _ => panic!("Not supported data")
        };
    }
}