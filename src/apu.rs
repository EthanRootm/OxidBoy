use super::clock::Clock;
use super::cpu;
use super::mem::Memory;
use blip_buf::BlipBuf;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

#[derive(Clone, PartialEq, Eq)]
enum Channel {
    Square1,
    Square2,
    Wave,
    Noise,
    Mixer,
}

struct Register {
    channel: Channel,
    nrx0: u8,
    nrx1: u8,
    nrx2: u8,
    nrx3: u8,
    nrx4: u8,
}

impl Register {
    fn get_sweep_period(&self) -> u8 {
        assert!(self.channel == Channel::Square1);
        (self.nrx0 >> 4) & 0x07
    }

    fn get_negate(&self) -> bool {
        assert!(self.channel == Channel::Square1);
        self.nrx0 & 0x08 != 0x00
    }

    fn get_shift(&self) -> u8 {
        assert!(self.channel == Channel::Square1);
        self.nrx0 & 0x07
    }

    fn get_dac_power(&self) -> bool {
        assert!(self.channel == Channel::Wave);
        self.nrx0 & 0x80 != 0x00
    }

    fn get_duty(&self) -> u8 {
        assert!(self.channel == Channel::Square1 || self.channel == Channel::Square2);
        self.nrx1 >> 6
    }

    fn get_length_load(&self) -> u16 {
        if self.channel == Channel::Wave {
            (1 << 8) - u16::from(self.nrx1)
        } else {
            (1 << 6) - u16::from(self.nrx1 & 0x3F)
        }
    }

    fn get_starting_volume(&self) -> u8 {
        assert!(self.channel != Channel::Wave);
        self.nrx2 >> 4
    }

    fn get_volume_code(&self) -> u8 {
        assert!(self.channel == Channel::Wave);
        (self.nrx2 >> 5) & 0x03
    }

    fn get_envelope_add_mode(&self) -> bool {
        assert!(self.channel != Channel::Wave);
        self.nrx2 & 0x08 != 0x00
    }

    fn get_period(&self) -> u8 {
        assert!(self.channel != Channel::Wave);
        self.nrx2 & 0x07
    }

    fn get_frequency(&self) -> u16 {
        assert!(self.channel != Channel::Noise);
        u16::from(self.nrx4 & 0x07) << 8 | u16::from(self.nrx3)
    }
    
    fn set_frequency(&mut self, f: u16) {
        assert!(self.channel != Channel::Noise);
        let h = ((f >> 8) & 0x07) as u8;
        let l = f as u8;
        self.nrx4 = (self.nrx4 & 0xf8) | h;
        self.nrx3 = l;
    }
    
    fn get_clock_shift(&self) -> u8 {
        assert!(self.channel == Channel::Noise);
        self.nrx3 >> 4
    }

    fn get_width_mode(&self) -> bool {
        assert!(self.channel == Channel::Noise);
        self.nrx3 & 0x08 != 0x00
    }

    fn get_dividor(&self) -> u8 {
        assert!(self.channel == Channel::Noise);
        self.nrx3 & 0x07
    }

    fn get_trigger(&self) -> bool {
        self.nrx4 & 0x80 != 0x00
    }

    fn set_trigger(&mut self, b: bool) {
        if b {
            self.nrx4 |= 0x80;
        } else {
            self.nrx4 &= 0x7f;
        };
    }

    fn get_length_enable(&self) -> bool {
        self.nrx4 & 0x40 != 0x00
    }

    fn get_l(&self) -> u8 {
        assert!(self.channel == Channel::Mixer);
        (self.nrx0 >> 4) & 0x07
    }

    fn get_r(&self) -> u8 {
        assert!(self.channel == Channel::Mixer);
        self.nrx0 & 0x07
    }

    fn get_power(&self) -> bool {
        assert!(self.channel == Channel::Mixer);
        self.nrx2 & 0x80 != 0x00
    }
}

impl Register {
    fn power_up(channel: Channel) -> Self {
        let nrx1 = match channel {
            Channel::Square1 | Channel::Square2 => 0x40,
            _ => 0x00,
        };
        Self { channel, nrx0: 0x00, nrx1, nrx2: 0x00, nrx3: 0x00, nrx4: 0x00 }
    }
}

struct FrameSequencer {
    step: u8
}

impl FrameSequencer {
    fn power_up() -> Self {
        Self { step: 0x00 }
    }

    fn next(&mut self) -> u8 {
        self.step += 1;
        self.step %= 8;
        self.step
    }
}

struct LengthCounter{
    reg: Rc<RefCell<Register>>,
    n: u16,
}

impl LengthCounter {
    fn power_up(reg: Rc<RefCell<Register>>) -> Self {
        Self { reg, n: 0x0000 }
    }

    fn next(&mut self) {
        if self.reg.borrow().get_length_enable() && self.n != 0 {
            self.n -= 1;
            if self.n == 0 {
                self.reg.borrow_mut().set_trigger(false);
            }
        }
    }

    fn reload(&mut self) {
        if self.n == 0x0000 {
            self.n = if self.reg.borrow().channel == Channel::Wave { 1 << 8 } else { 1 << 6 };
        }
    }
}

struct VolumeEnvelope {
    reg: Rc<RefCell<Register>>,
    timer: Clock,
    volume: u8,
}

impl VolumeEnvelope {
    fn power_up(reg: Rc<RefCell<Register>>) -> Self {
        Self { reg, timer: Clock::power_up(8), volume: 0x00 }
    }

    fn next(&mut self) {
        if self.reg.borrow().get_period() == 0 {
            return;
        }
        if self.timer.next(1) == 0x00 {
            return;
        };
        let v = if self.reg.borrow().get_envelope_add_mode() {
            self.volume.wrapping_add(1)
        } else {
            self.volume.wrapping_sub(1)
        };
        if v <= 15 {
            self.volume = v;
        }
    }

    fn reload(&mut self) {
        let p = self.reg.borrow().get_period();
        self.timer.period = if p == 0 { 8 } else { u32::from(p) };
        self.volume = self.reg.borrow().get_starting_volume();
    }
}

struct FrequencySweep {
    reg: Rc<RefCell<Register>>,
    timer: Clock,
    enable: bool,
    shadow: u16,
    newfeq: u16,
}

impl FrequencySweep {
    fn power_up(reg: Rc<RefCell<Register>>) -> Self {
        Self { reg, timer: Clock::power_up(8), enable: false, shadow: 0x0000, newfeq: 0x0000 }
    }

    fn next(&mut self) {
        if !self.enable || self.reg.borrow().get_sweep_period() == 0 {
            return;
        }
        if self.timer.next(1) == 0x00 {
            return;
        }
        self.frequency_calc();
        self.overflow_check();

        if self.newfeq < 2048 && self.reg.borrow().get_shift() != 0 {
            self.reg.borrow_mut().set_frequency(self.newfeq);
            self.shadow = self.newfeq;
            self.frequency_calc();
            self.overflow_check();
        }
    }

    fn frequency_calc(&mut self) {
        let offset = self.shadow >> self.reg.borrow().get_shift();
        if self.reg.borrow().get_negate() {
            self.newfeq = self.shadow.wrapping_sub(offset);
        } else {
            self.newfeq = self.shadow.wrapping_add(offset);
        }
    }

    fn overflow_check(&mut self) {
        if self.newfeq >= 2048 {
            self.reg.borrow_mut().set_trigger(false);
        }
    }

    fn reload(&mut self) {
        self.shadow = self.reg.borrow().get_frequency();
        let p = self.reg.borrow().get_sweep_period();
        self.timer.period = if p == 0 { 8 } else { u32::from(p) };
        self.enable = p != 0x00 || self.reg.borrow().get_shift() != 0x00;
        if self.reg.borrow().get_shift() != 0x00 {
            self.frequency_calc();
            self.overflow_check();
        }
    }
}

struct Blip {
    data: BlipBuf,
    from: u32,
    ampl: i32,
}

impl Blip {
    fn power_up(data: BlipBuf) -> Self {
        Self { data, from: 0x0000_0000, ampl: 0x0000_0000 }
    }

    fn set(&mut self, time: u32, ampl: i32) {
        self.from = time;
        let d = ampl - self.ampl;
        self.ampl = ampl;
        self.data.add_delta(time, d);
    }
}

struct ChannelSquare {
    reg: Rc<RefCell<Register>>,
    timer: Clock,
    lc: LengthCounter,
    ve: VolumeEnvelope,
    fs: FrequencySweep,
    blip: Blip,
    idx: u8,
}

impl ChannelSquare {
    fn power_up(blip: BlipBuf, mode: Channel) -> ChannelSquare {
        let reg = Rc::new(RefCell::new(Register::power_up(mode.clone())));
        ChannelSquare { reg: reg.clone(), timer: Clock::power_up(8192), lc: LengthCounter::power_up(reg.clone()), ve: VolumeEnvelope::power_up(reg.clone()),
            fs: FrequencySweep::power_up(reg.clone()), blip: Blip::power_up(blip), idx: 1, }
    }

    fn next(&mut self, cycles: u32) {
        let pat = match self.reg.borrow().get_duty() {
            0 => 0b0000_0001,
            1 => 0b1000_0001,
            2 => 0b1000_0111,
            3 => 0b0111_1110,
            _ => unreachable!(),
        };
        let vol = i32::from(self.ve.volume);
        for _ in 0..self.timer.next(cycles) {
            let ampl = if !self.reg.borrow().get_trigger() || self.ve.volume == 0 {
                0x00
            }else if (pat >> self.idx) & 0x01 != 0x00 {
                vol
            } else {
                vol * -1
            };
            self.blip.set(self.blip.from.wrapping_add(self.timer.period), ampl);
            self.idx = (self.idx + 1) % 8;
        }
    }
}

impl Memory for ChannelSquare {
    fn get(&self, a: u16) -> u8 {
        match a {
            0xFF10 | 0xFF15 => self.reg.borrow().nrx0,
            0xFF11 | 0xFF16 => self.reg.borrow().nrx1,
            0xFF12 | 0xFF17 => self.reg.borrow().nrx2,
            0xFF13 | 0xFF18 => self.reg.borrow().nrx3,
            0xFF14 | 0xFF19 => self.reg.borrow().nrx4,
            _ => unreachable!()    
        }
    }

    fn set(&mut self, a: u16, v: u8) {
        match a {
            
        }
    }
}