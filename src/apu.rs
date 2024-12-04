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
            0xFF10 | 0xFF15 => self.reg.borrow_mut().nrx0 = v,
            0xFF11 | 0xFF16 => {
                self.reg.borrow_mut().nrx1 = v;
                self.lc.n = self.reg.borrow().get_length_load();
            }
            0xFF12 | 0xFF17 => self.reg.borrow_mut().nrx2 = v,
            0xFF13 | 0xFF18 => {
                self.reg.borrow_mut().nrx3 = v;
                self.timer.period = period(self.reg.clone());
            }
            0xFF14 | 0xFF19 => {
                self.reg.borrow_mut().nrx4 = v;
                self.timer.period = period(self.reg.clone());
                if self.reg.borrow().get_trigger() {
                    self.lc.reload();
                    self.ve.reload();
                    if self.reg.borrow().channel == Channel::Square1 {
                        self.fs.reload();
                    }
                }
            }
            _ => unreachable!()
        }
    }
}

struct ChannelWave {
    reg: Rc<RefCell<Register>>,
    timer: Clock,
    lc: LengthCounter,
    blip: Blip,
    waveram: [u8; 16],
    waveidx: usize,
}

impl ChannelWave {
    fn power_up(blip: BlipBuf) -> ChannelWave {
        let reg = Rc::new(RefCell::new(Register::power_up(Channel::Wave)));
        ChannelWave { reg: reg.clone(), timer: Clock::power_up(8192), lc: LengthCounter::power_up(reg.clone()), blip: Blip::power_up(blip), waveram: [0x00; 16], waveidx: 0x00 }
    }

    fn next(&mut self, cycles: u32) {
        let s = match self.reg.borrow().get_volume_code() {
            0 => 4,
            1 => 0,
            2 => 1,
            3 => 2,
            _ => unreachable!(),
        };
        for _ in 0..self.timer.next(cycles) {
            let sample = if self.waveidx & 0x01 == 0x00 {
                self.waveram[self.waveidx / 2] & 0x0F
            } else {
                self.waveram[self.waveidx / 2] >> 4
            };
            let amplitude = if !self.reg.borrow().get_trigger() || !self.reg.borrow().get_dac_power() {
                0x00
            } else {
                i32::from(sample >> s)
            };
            self.blip.set(self.blip.from.wrapping_add(self.timer.period), amplitude);
            self.waveidx = (self.waveidx + 1) % 32;
        }
    }
}

impl Memory for ChannelWave {
    fn get(&self, a: u16) -> u8 {
        match a {
            0xFF1A => self.reg.borrow().nrx0,
            0xFF1B => self.reg.borrow().nrx1,
            0xFF1C => self.reg.borrow().nrx2,
            0xFF1D => self.reg.borrow().nrx3,
            0xFF1E => self.reg.borrow().nrx4,
            0xFF30..=0xFF3F => self.waveram[a as usize - 0xFF30],
            _ => unreachable!()
        }    
    }

    fn set(&mut self, a: u16, v: u8) {
        match a {
            0xFF1A => self.reg.borrow_mut().nrx0 = v,
            0xFF1B => {
                self.reg.borrow_mut().nrx1 = v;
                self.lc.n = self.reg.borrow().get_length_load();
            }
            0xFF1C => self.reg.borrow_mut().nrx2 = v,
            0xFF1D => {
                self.reg.borrow_mut().nrx3 = v;
                self.timer.period = period(self.reg.clone());
            }
            0xFF1E => {
                self.reg.borrow_mut().nrx4 = v;
                self.timer.period = period(self.reg.clone());
                if self.reg.borrow().get_trigger() {
                    self.lc.reload();
                    self.waveidx = 0x00;
                }
            }
            0xFF30..=0xFF3F => self.waveram[a as usize - 0xFF30] = v,
            _ => unreachable!(),
        }
    }
}

struct Lfsr {
    reg: Rc<RefCell<Register>>,
    n: u16,
}

impl Lfsr {
    fn power_up(reg: Rc<RefCell<Register>>) -> Self {
        Self { reg, n: 0x0001 }
    }

    fn next(&mut self) -> bool {
        let s = if self.reg.borrow().get_width_mode() { 0x06 } else { 0x0E };
        let src = self.n;
        self.n <<= 1;
        let bit = ((src >> s) ^ (self.n >>s)) & 0x0001;
        self.n |= bit;
        (src >> s) & 0x0001 != 0x0000
    }

    fn reload(&mut self) {
        self.n = 0x0001
    }
}

struct ChannelNoise {
    reg: Rc<RefCell<Register>>,
    timer: Clock,
    lc: LengthCounter,
    ve: VolumeEnvelope,
    lfsr: Lfsr,
    blip: Blip,
}

impl ChannelNoise {
    fn power_up(blip: BlipBuf) -> ChannelNoise {
        let reg = Rc::new(RefCell::new(Register::power_up(Channel::Noise)));
        ChannelNoise { reg: reg.clone(), timer: Clock::power_up(4096), lc: LengthCounter::power_up(reg.clone()), 
        ve: VolumeEnvelope::power_up(reg.clone()), lfsr: Lfsr::power_up(reg.clone()), blip: Blip::power_up(blip) }
    }

    fn next(&mut self, cycles: u32) {
        for _ in 0..self.timer.next(cycles) {
            let amplitude = if !self.reg.borrow().get_trigger() || self.ve.volume == 0 {
                0x00
            } else if self.lfsr.next() {
                i32::from(self.ve.volume)
            } else {
                i32::from(self.ve.volume) * -1
            };
            self.blip.set(self.blip.from.wrapping_add(self.timer.period), amplitude);
        }
    }
}

impl Memory for ChannelNoise {
    fn get(&self, a: u16) -> u8 {
        match a {
            0xFF1F => self.reg.borrow().nrx0,
            0xFF20 => self.reg.borrow().nrx1,
            0xFF21 => self.reg.borrow().nrx2,
            0xFF22 => self.reg.borrow().nrx3,
            0xFF23 => self.reg.borrow().nrx4,
            _ => unreachable!(),
        }
    }

    fn set(&mut self, a: u16, v: u8) {
        match a {
            0xFF1F => self.reg.borrow_mut().nrx0 = v,
            0xFF20 => {
                self.reg.borrow_mut().nrx1 = v;
                self.lc.n = self.reg.borrow().get_length_load();
            }
            0xFF21 => self.reg.borrow_mut().nrx2 = v,
            0xFF22 => {
                self.reg.borrow_mut().nrx3 = v;
                self.timer.period = period(self.reg.clone());
            }
            0xFF23 => {
                self.reg.borrow_mut().nrx4 = v;
                if self.reg.borrow().get_trigger() {
                    self.lc.reload();
                    self.ve.reload();
                    self.lfsr.reload();
                }
            }
            _ => unreachable!(),
        }
    }
}

pub struct Apu {
    pub buffer: Arc<Mutex<Vec<(f32, f32)>>>,
    reg: Register,
    timer: Clock,
    fs: FrameSequencer,
    channel1: ChannelSquare,
    channel2: ChannelSquare,
    channel3: ChannelWave,
    channel4: ChannelNoise,
    sample_rate: u32,
}

impl Apu {
    pub fn power_up(sample: u32) -> Self {
        let blipbuf1 = create_blipbuf(sample);
        let blipbuf2 = create_blipbuf(sample);
        let blipbuf3 = create_blipbuf(sample);
        let blipbuf4 = create_blipbuf(sample);
        Self { buffer: Arc::new(Mutex::new(Vec::new())), reg: Register::power_up(Channel::Mixer), timer: Clock::power_up(cpu::CLOCK_FREQUENCY / 512), 
        fs: FrameSequencer::power_up(), channel1: ChannelSquare::power_up(blipbuf1, Channel::Square1),
        channel2: ChannelSquare::power_up(blipbuf2, Channel::Square2), 
        channel3: ChannelWave::power_up(blipbuf3), channel4: ChannelNoise::power_up(blipbuf4), sample_rate: sample }
    }

    fn play(&mut self, l: &[f32], r: &[f32]) {
        assert_eq!(l.len(), r.len());
        let mut buffer = self.buffer.lock().unwrap();
        for (l, r) in l.iter().zip(r) {
            if buffer.len() > self.sample_rate as usize {
                return;
            }
            buffer.push((*l, *r));
        }
    }

    pub fn next(&mut self, cycles: u32) {
        if !self.reg.get_power() {
            return;
        }

        for _ in 0..self.timer.next(cycles) {
            self.channel1.next(self.timer.period);
            self.channel2.next(self.timer.period);
            self.channel3.next(self.timer.period);
            self.channel4.next(self.timer.period);

            let step = self.fs.next();
            if step == 0 || step == 2 || step == 4 || step == 6 {
                self.channel1.lc.next();
                self.channel2.lc.next();
                self.channel3.lc.next();
                self.channel4.lc.next();
            }
            if step == 7 {
                self.channel1.ve.next();
                self.channel2.ve.next();
                self.channel4.ve.next();
            }
            if step == 2 || step == 6 {
                self.channel1.fs.next();
                self.channel1.timer.period = period(self.channel1.reg.clone());
            }
            self.channel1.blip.data.end_frame(self.timer.period);
            self.channel2.blip.data.end_frame(self.timer.period);
            self.channel3.blip.data.end_frame(self.timer.period);
            self.channel4.blip.data.end_frame(self.timer.period);
            
            self.channel1.blip.from = self.channel1.blip.from.wrapping_sub(self.timer.period);
            self.channel2.blip.from = self.channel2.blip.from.wrapping_sub(self.timer.period);
            self.channel3.blip.from = self.channel3.blip.from.wrapping_sub(self.timer.period);
            self.channel4.blip.from = self.channel4.blip.from.wrapping_sub(self.timer.period);
            self.mix();
        }
    }

    fn mix(&mut self) {
        let sc1 = self.channel1.blip.data.samples_avail();
        let sc2 = self.channel2.blip.data.samples_avail();
        let sc3 = self.channel3.blip.data.samples_avail();
        let sc4 = self.channel4.blip.data.samples_avail();
        assert_eq!(sc1, sc2);
        assert_eq!(sc2, sc3);
        assert_eq!(sc3, sc4);

        let sample_count = sc1 as usize;
        let mut sum = 0;

        let l_volume = (f32::from(self.reg.get_l()) / 7.0) * (1.0 / 15.0) * 0.25;
        let r_volume = (f32::from(self.reg.get_r()) / 7.0) * (1.0 / 15.0) * 0.25;

        while sum < sample_count {
            let buf_l = &mut [0f32; 2048];
            let buf_r = &mut [0f32; 2048];
            let buf = &mut [0i16; 2048];

            let count1 = self.channel1.blip.data.read_samples(buf, false);
            for (i, v) in buf[..count1].iter().enumerate() {
                if self.reg.nrx1 & 0x01 == 0x01 {
                    buf_l[i] += f32::from(*v) * l_volume;
                }
                if self.reg.nrx1 & 0x10 == 0x10 {
                    buf_r[i] += f32::from(*v) * r_volume;
                }
            }

            let count2 = self.channel2.blip.data.read_samples(buf, false);
            for (i, v) in buf[..count2].iter().enumerate() {
                if self.reg.nrx1 & 0x02 == 0x02 {
                    buf_l[i] += f32::from(*v) * l_volume;
                }
                if self.reg.nrx1 & 0x20 == 0x20 {
                    buf_r[i] += f32::from(*v) * r_volume;
                }
            }

            let count3 = self.channel3.blip.data.read_samples(buf, false);
            for (i, v) in buf[..count3].iter().enumerate() {
                if self.reg.nrx1 & 0x04 == 0x04 {
                    buf_l[i] += f32::from(*v) * l_volume;
                }
                if self.reg.nrx1 & 0x40 == 0x40 {
                    buf_r[i] += f32::from(*v) * r_volume;
                }
            }

            let count4 = self.channel4.blip.data.read_samples(buf, false);
            for (i, v) in buf[..count4].iter().enumerate() {
                if self.reg.nrx1 & 0x08 == 0x08 {
                    buf_l[i] += f32::from(*v) * l_volume;
                }
                if self.reg.nrx1 & 0x80 == 0x80 {
                    buf_r[i] += f32::from(*v) * r_volume;
                }
            }
            assert_eq!(count1, count2);
            assert_eq!(count2, count3);
            assert_eq!(count3, count4);

            self.play(&buf_l[..count1], &buf_r[..count1]);
            sum += count1;
        }
    }
}

const RD_MASK: [u8; 48] = [
    0x80, 0x3f, 0x00, 0xff, 0xbf, 0xff, 0x3f, 0x00, 0xff, 0xbf, 0x7f, 0xff, 0x9f, 0xff, 0xbf, 0xff, 0xff, 0x00, 0x00,
    0xbf, 0x00, 0x00, 0x70, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

impl Memory for Apu {
    
}



fn create_blipbuf(sample: u32) -> BlipBuf {
    
}

fn period(reg: Rc<RefCell<Register>>) -> u32 {
    
}