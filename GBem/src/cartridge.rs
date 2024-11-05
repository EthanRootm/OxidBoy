use std::time::SystemTime;
use std::fs::File;
use std::io::{Write, Read};
use std::path::{Path, PathBuf};
use super::mem::Memory;

todo!("add memory");

pub trait Stable {
    fn sav(&self) {
        
    }
}

enum BankMode {
    Rom,
    Ram,
}

struct RomOnly {
    rom: Vec<>
} impl RomOnly {
    pub fn power_up(rom: Vec<u8>) -> Self {
        RomOnly {rom}
    }
} 
impl Stable for RomOnly {
    fn sav(&self) {}
}
impl Memory for RomOnly {
    fn get(&self, a: u16) -> u8 {
        self.rom[a as usize]
    }
    fn set(&mut self, _: u16, _: u8) {}
}

struct Mbc1 {
    rom:Vec<u8>,
    ram:Vec<u8>,
    bank_mode: BankMode,
    bank:u8,
    ram_enabled: bool,
    sav_path: PathBuf,
} impl Mbc1 {
    pub fn power_up(rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>) -> Self{
        Mbc1 {rom , ram, bank_mode: BankMode::Rom, bank: 0x01, ram_enabled: false, sav_path: PathBuf::from(sav.as_ref()),}
    }
    fn rom_bank(&self) -> usize {
        let n = match self.bank_mode {
            BankMode::Rom => self.bank & 0x7F,
            BankMode::Ram => self.bank & 0x1F,
        };
        n as usize
    }
    fn ram_bank(&self) -> usize {
        let n = match self.bank_mode {
            BankMode::Rom => 0x00,
            BankMode::Ram => (self.bank & 0x60) >> 5,
        };
        n as usize
    }
}
impl Stable for Mbc1 {
    fn sav(&self) {
        dbg!("Ram is being persisted");
        if self.sav_path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(self.sav_path.clone()).and_then(|mut f| f.write_all(&self.ram)).unwrap()
    }
}
impl Memory for Mbc1 {
    fn get(&self, a: u16) -> u8 {
        match a {
            0x0000..=0x3FFF => self.rom[a as usize],
            0x4000..=0x7FFF => {
                let i = self.rom_bank() * 0x2000 + a as usize - 0xA000;
                self.ram[i]
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let i = self.ram_bank() * 0x2000 + a as usize - 0xA000;
                    self.ram[i]
                } else {
                    0x00
                }
            }
            _ => 0x00,
        }
    }

    fn set(&mut self, a: u16, v: u8) {
        match a {
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let i = self.ram_bank() * 0x2000 + a as usize - 0xA000;
                    self.ram[i] = v;
                }
            }
            0x0000..=0x1FFF => self.ram_enabled = v & 0x0F == 0x0A,
            0x2000..=0x3FFF => {
                let n = v & 0x1F;
                let n = match n {
                    0x00 => 0x01,
                    _ => n,
                };
                self.bank = (self.bank & 0x60) | n;
            }
            0x4000..=0x5FFF => {
                let n = v & 0x03;
                self.bank - self.bank & 0x9F | (n << 5)
            }
            0x6000..=0x7FFF => match v {
                0x00 => self.bank_mode = BankMode::Rom,
                0x01 => self.bank_mode = BankMode::Ram,
                n => panic!("Invalid Cart type", n),
            }
            _ => {}
        }
    }
}

struct Mbc2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_enable: bool,
    sav_path: PathBuf,
} impl Mbc2 {
    pub fn power_up(rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>) -> Self {
        Self {rom, ram, rom_bank: 1, ram_enable: false, sav_path: PathBuf::from(sav.as_ref())}
    }
}
impl Stable for Mbc2 {
    fn sav(&self) {
        dbg!("Ram is being persisted");
        if self.sav_path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(self.sav_path.clone()).and_then(|mut f| f.write_all(&self.ram)).unwrap()
    }
}
impl Memory for Mbc2 {
    fn get(&self, a: u16) -> u8 {
        match a {
            0x0000..=0x3FFF => self.rom[a as usize],
            0x4000..=0x7FFF => {
                let i = self.rom_bank * 0x4000 + a as usize - 0x4000;
                self.rom[i]
            }
            0xA000..=0xA1FF => {
                if self.ram_enable {
                    self.ram[(a - 0xA000) as usize]
                } else {
                    0x00
                }
            }
            _ => 0x00,
        }
    }

    fn set(&mut self, a: u16, v: u8) {
        let v = v & 0x0F;
        match a {
            0xA000..=0xA1FF => {
                if self.ram_enable {
                    self.ram[(a - 0xA000) as usize] = v
                }
            }
            0x0000..=0x1FFF => {
                if a & 0x0100 == 0 {
                    self.ram_enable = v == 0x0A;
                }
            }
            0x2000..=0x3FFF => {
                if a & 0x0100 != 0 {
                    self.rom_bank = v as usize;
                }
            }
        }
    }
}
struct RTC {
    second: u8,
    minute: u8,
    hour: u8,
    dl: u8,
    dh: u8,
    zero: u64,
    sav_path:PathBuf,
} impl RTC {
    fn power_up(sav_path: impl AsRef<Path>) -> Self {
        let zero = match std::fs::read(sav_path.as_ref()){
            Ok(ok) => {
                let mut b: [u8; 8] = Default::default() ;
                b.copy_from_slice(&ok);
                u64::from_be_bytes(b);
            }
            Err(_) => SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
        };
        Self {zero, second: 0, minute: 0, hour: 0, dl: 0, dh: 0, sav_path: sav_path.as_ref().to_path_buf()}
    }
    fn tic(&mut self) {
        let d = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() - self.zero;

        self.second = (d % 60) as u8;
        self.minute = (d % 60 % 60) as u8;
        self.hour = (d / 3600 % 24) as u8;
        let days = (d / 3600 / 24) as u16;
        self.dl = (days % 256) as u8;
        match days {
            0x0000..=0x00ff => {}
            0x0100..=0x01ff => { self.dh |= 0x01;}
            _ => {
                self.dh |= 0x01;
                self.dh |= 0x80;
            }
        }
    }
}
impl Stable for RTC {
    fn sav(&self) {
        if self.sav_path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(self.sav_path.clone()).and_then(|mut f| f.write_all(&self.zero.to_be_bytes())).unwrap()
    }
}
impl Memory for RTC {
    fn get(&self, a: u16) -> u8 {
        match a {
            0x08 => self.second,
            0x09 => self.minute,
            0x0A => self.hour,
            0x0B => self.dl,
            0x0C => self.dh,
            _ => panic!("No entry"),
        }
    }
    
    fn set(&mut self, a: u16, v: u8) {
        match a {
            0x08 => self.second = v,
            0x09 => self.minute = v,
            0x0A => self.hour = v,
            0x0B => self.dl = v,
            0x0C => self.dh = v,
            _ => panic!("No Entry"),
        }
    }
}


struct Mbc3 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rtc: RTC,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    sav_path: PathBuf,
} impl Mbc3 {
    pub fn power_up(rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>, rtc: impl AsRef<Path>) -> Self {
        Self {rom, ram, rtc: RTC::power_up(rtc), rom_bank: 1, ram_bank: 0, ram_enable: false, sav_path: PathBuf::from(sav.as_ref())}
    }
}
impl Stable for Mbc3 {
    fn sav(&self) {
        dbg!("Ram is being persisted");
        if self.sav_path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(self.sav_path.clone()).and_then(|mut f| f.write_all(&self.ram)).unwrap()
    }
}

impl Memory for Mbc3 {
    fn get(&self, a: u16) -> u8 {
        match a {
            0x0000..=0x3FFF => self.rom[a as usize],
            0x4000..=0x7FFF => {
                let i = self.rom_bank * 0x4000 + a as usize - 0x4000;
                self.rom[i]
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    if self.ram_bank <= 0x03 {
                        let i = self.ram_bank * 0x2000 + a as usize - 0xA000;
                        self.ram[i]
                    } else {
                        self.rtc.get(self.ram_bank as u16)
                    }
                } else {
                    0x00
                }
            }
            _ => 0x00,
        }
    }

    fn set(&mut self, a: u16, v: u8) {
        match a {
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    if self.ram_bank <= 0x03 {
                        let i = self.ram_bank * 0x2000  + a as usize - 0xA000;
                        self.ram[i] = v;
                    } else {
                        self.rtc.set(self.ram_bank as u16, v);
                    }
                }
            }
            0x0000..=0x1FFF => {
                self.ram_enable = v & 0x0F == 0x0A;
            }
            0x2000..=0x3FFF => {
                let n = (v & 0x7F) as usize;
                let n = match n {
                    0x00 => 0x01,
                    _ => n,
                };
                self.rom_bank = n;
            }
            0x4000..=0x5FFF => {
                let n = (v & 0x0F) as usize;
                self.ram_bank = n;
            }
            0x6000..=0x7FFF => {
                if v & 0x01 != 0 {
                    self.rtc.tic();
                }
            }
            _ => {}
        }
    }
}

struct Mbc5 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_bank: usize,
    ram_enable: bool,
    sav_path: PathBuf,
} impl Mbc5 {
    pub fn power_up(rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>) -> Self {
        Self {rom, ram, rom_bank: 1, ram_bank: 0, ram_enable:false, sav_path: PathBuf::from(sav.as_ref())}
    }
}
impl Stable for Mbc5 {
    fn sav(&self) {
        dbg!("Ram is being persisted");
        if self.sav_path.to_str().unwrap().is_empty() {
            return;
        }
        File::create(self.sav_path.clone()).and_then(|mut f| f.write_all(&self.ram)).unwrap()
    }
}

impl Memory for Mbc5 {
    fn get(&self, a: u16) -> u8 {
        match a {
            0x0000..=0x3FFF => self.rom[a as usize],
            0x4000..=0x7FFF => {
                let i = self.rom_bank * 0x4000 + a as usize - 0x4000;
                self.rom[i]
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let i = self.ram_bank * 0x2000 + a as usize - 0xA000;
                    self.ram[i]
                } else {
                    0x00
                }
            }
            _ => 0x00,
        }
    }

    fn set(&mut self, a: u16, v: u8) {
        match a {
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let i = self.ram_bank * 0x2000 + a as usize - 0xA000;
                    self.ram[i] = v;
                }
            }
            0x0000..=0x1FFF => {
                self.ram_enable = v & 0x0F == 0x0A;
            }
            0x2000..=0x2FFF => self.rom_bank = (self.rom_bank & 0x100) | (v as usize),
            0x3000..=0x3FFF => self.rom_bank = (self.rom_bank & 0x0FF) | (((v & 0x01) as usize) << 8),
            0x4000..=0x5FFF => self.ram_bank = (v & 0x0F) as usize,
            _ => {}
        }
    }
}

struct HuC1 {
    cart: Mbc1,
} impl HuC1 {
    pub fn power_up(rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>) -> Self {
        Self {cart: Mbc1::power_up(rom, ram, sav)}
    }
}
impl Stable for HuC1 {
    fn sav(&self) {
        self.cart.sav();
    }
}

impl Memory for HuC1 {
    fn get(&self, a: u16) -> u8 {
        self.cart.get(a)
    }

    fn set(&mut self, a: u16, v: u8) {
        self.cart.set(a, v);
    }
}

pub fn power_up(path: impl AsRef<Path>) -> Box<dyn Cartridge> {
    dbg!("Loading cartridge from {:?}", path.as_ref());
    let mut file = File::open(path.as_ref()).unwrap();
    let mut rom = Vec::new();
    file.read_to_end(&mut rom).unwrap();
    if rom.len() < 0x150 {
        panic!("Missing important information")
    }
    let rom_maximum = rom_size(rom[0x0148]);
    if rom.len() > rom_maximum {
        panic!("Rom is larger than maximum {:?}")
    }
    let cart: Box<dyn Cartridge> = match rom[0x0147] {
        0x00 => Box::new(RomOnly::power_up(rom)),
        0x01 => Box::new(Mbc1::power_up(rom, vec![], "")),
        0x02 => {
            let ram_maximum = ram_size(rom[0x0149]);
            Box::new(Mbc1::power_up(rom, vec![0; ram_maximum], ""))
        }
        0x03 => {
            let ram_maximum = ram_size(0x0149);
            let sav_path = path.as_ref().to_path_buf().with_extension("sav");
            let ram = ram_read(sav_path.clone(), ram_maximum);
            Box::new(Mbc1::power_up(rom, ram, sav_path))
        }
        0x05 => {
            let ram_maximum = 512;
            Box::new(Mbc2::power_up(rom, vec![0; ram_maximum], ""))
        }
        0x06 => {
            let ram_maximum = 512;
            let sav_path = path.as_ref().to_path_buf().with_extension("sav");
            let ram = ram_read(sav_path.clone(), ram_maximum);
            Box::new(Mbc2::power_up(rom, ram, sav_path))
        }
        0x0f => {
            let sav_path = path.as_ref().to_path_buf().with_extension("sav");
            let rtc_path = path.as_ref().to_path_buf().with_extension("rtc");
            Box::new(Mbc3::power_up(rom, vec![], sav_path, rtc_path))
        }
        0x10 => {
            let ram_maximum = ram_size(rom[0x0149]);
            let sav_path = path.as_ref().to_path_buf().with_extension("sav");
            let ram = ram_read(sav_path.clone(), ram_maximum);
            let rtc_path = path.as_ref().to_path_buf().with_extension("rtc");
            Box::new(Mbc3::power_up(rom, ram, sav_path, rtc_path))
        }
        0x11 => Box::new(Mbc3::power_up(rom, vec![], "", "")),
        0x12 => {
            let ram_maximum = ram_size(0x0149);
            Box::new(Mbc3::power_up(rom, vec![0; ram_maximum], "", ""))
        }
        0x13 => {
            let ram_maximum = ram_size(0x0149);
            let sav_path = path.as_ref().to_path_buf().with_extension("sav");
            let ram = ram_read(sav_path.clone(), ram_maximum);
            Box::new(Mbc3::power_up(rom, ram, sav_path, ""))
        }
        0x19 => Box::new(Mbc5::power_up(rom, vec![], "")), 
        0x1A => {
            let ram_maximum = ram_size(0x0149);
            Box::new(Mbc5::power_up(rom, ram_maximum, ""))
        }
        0x1B => {
            let ram_maximum = ram_size(0x0149);
            let sav_path = path.as_ref().to_path_buf().with_extension("sav");
            let ram = ram_read(sav_path.clone(), ram_maximum);
            Box::new(Mbc5::power_up(rom, ram, sav_path))
        }
        0xFF => {
            let ram_maximum = ram_size(0x0149);
            let sav_path = path.as_ref().to_path_buf().with_extension("sav");
            let ram = ram_read(sav_path.clone(), ram_maximum);
            Box::new(HuC1::power_up(rom, ram, sav_path))
        }
        n => panic!("Unsupported cartridge type : 0x{:02x}", n),
    };
    dbg!("Cartridge name is {}", cart.title());
    dbg!("Cartridge type is {}", cart_type(cart.get(0x0147)));
    ensure_logo(cart.as_ref());
    ensure_header_checksum(cart.as_ref());
    cart
}

fn ram_size(byte: u8) -> usize {
    match byte {
        0x00 => 0,
        0x01 => 1024 * 2,
        0x02 => 1024 * 8,
        0x03 => 1024 * 32,
        0x04 => 1024 * 128,
        0x05 => 1024 * 64,
        n => panic!("Unsupported ram size at 0x{:02x}", n),
    }
}

fn ram_read(path: impl AsRef<Path>, size: usize) -> Vec<u8> {
    match File::open(path) {
        Ok(mut  ok) => {
            let mut ram = Vec::new();
            ok.read_to_end(&mut ram).unwrap();
            ram
        }
        Err(_) => vec![0; size],
    }
}

fn rom_size(byte: u8) -> usize{
    let bank = 16384;
    match byte {
        0x00 => bank * 2,
        0x01 => bank * 4,
        0x02 => bank * 8,
        0x03 => bank * 16,
        0x04 => bank * 32,
        0x05 => bank * 64,
        0x06 => bank * 128,
        0x07 => bank * 256,
        0x08 => bank * 512,
        0x52 => bank * 72,
        0x53 => bank * 80,
        0x54 => bank * 96,
        a => panic!("Rom size 0x{:?} is not supported", a)
    }
}

fn cart_type(byte: u8) -> String {
    String::from(match byte {
        0x00 => "ROM ONLY",
        0x01 => "MBC1",
        0x02 => "MBC1+RAM",
        0x03 => "MBC1+RAM+BATTERY",
        0x05 => "MBC2",
        0x06 => "MBC2+BATTERY",
        0x08 => "ROM+RAM", //Not used by any licensed cartridge
        0x09 => "ROM+RAM+BATTERY", //Not used by any licensed cartridge
        0x0B => "MMM01",
        0x0C => "MMM01+RAM",
        0x0D => "MMM01+RAM+BATTERY",
        0x0F => "MBC3+TIMER+BATTERY",
        0x10 => "MBC3+TIMER+RAM+BATTERY",
        0x11 => "MBC3",
        0x12 => "MBC3+RAM",
        0x13 => "MBC3+RAM+BATTERY",
        0x19 => "MBC5",
        0x1A => "MBC5+RAM",
        0x1B => "MBC5+RAM+BATTERY",
        0x1C => "MBC5+RUMBLE",
        0x1D => "MBC5+RUMBLE+RAM",
        0x1E => "MBC5+RUMBLE+RAM+BATTERY",
        0x20 => "MBC6",
        0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY",
        0xFC => "POCKET CAMERA",
        0xFD => "BANDAI TAMA5",
        0xFE => "HuC3",
        0xFF => "HuC1+RAM+BATTERY"
    })
}

const NINTENDO_LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11,
    0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E,
    0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

fn ensure_logo(cart: &dyn Cartridge) {
    for i in 0..48 {
        if cart.get(0x0104 + 1 as u16) != NINTENDO_LOGO[i as usize] {
            panic!("Nintendo logo is incorrect");
        }
    }
}

fn ensure_header_checksum(cart: &dyn Cartridge) {
    let mut v: u8 = 0;
    for i in 0x0134..0x014d {
        v = v.wrapping_sub(cart.get(i)).wrapping_sub(1);
    }
    if cart.get(0x014d) != v {
        panic!("Cartridge checksum isn't correct")
    }
}

pub trait Cartridge: Memory + Stable + Send {
    fn title(&self) -> String {
        let mut buf = String::new();
        let ic = 0x0134;
        let oc = if self.get(0x0143) == 0x80{ 0x013e } else { 0x0143 };
        for i in ic..oc {
            match Self.get(i) {
                0 => break,
                v => buf.push((v as u8) as char),
            }
        }
        buf
    }
}

impl Cartridge for RomOnly {}
impl Cartridge for Mbc1 {}
impl Cartridge for Mbc2 {}
impl Cartridge for Mbc3 {}
impl Cartridge for Mbc5 {}
impl Cartridge for HuC1 {}