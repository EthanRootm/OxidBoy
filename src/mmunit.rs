//use super::apu::Apu;
use super::cartridge::{self, Cartridge};
use super::terms::Term;
use super::gpu::{Gpu, Hdma, HdmaMode};
use super::intf::Intf;
//use super::joypad::Joypad;
use super::linkcable::Serial;
use super::mem::Memory;
use super::timer::Timer;
use std::cell::{Ref, RefCell};
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