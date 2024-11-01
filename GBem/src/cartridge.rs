use std::time::SystemTime;
use std::{fs::File, io::Read};
use std::path::{Path, PathBuf};

todo!("add memory and sav functionality");

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

struct Mbc1 {
    rom:Vec<u8>,
    ram:Vec<u8>,
    bank_mode: BankMode,
    bank:u8,
    ram_enabled: bool,
    save_path: PathBuf,
} impl Mbc1 {
    pub fn power_up(rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>) -> Self{
        Mbc1 {rom , ram, bank_mode: BankMode::Rom, bank: 0x01, ram_enabled: false, save_path: PathBuf::from(sav.as_ref()),}
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

struct Mbc2 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: usize,
    ram_enable: bool,
    save_path: PathBuf,
} impl Mbc2 {
    pub fn power_up(rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>) -> Self {
        Self {rom, ram, rom_bank: 1, ram_enable: false, save_path: PathBuf::from(sav.as_ref())}
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
todo!("memory");

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
struct HuC1 {
    cart: Mbc1,
} impl HuC1 {
    pub fn power_up(rom: Vec<u8>, ram: Vec<u8>, sav: impl AsRef<Path>) -> Self {
        Self {cart: Mbc1::power_up(rom, ram, sav)}
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
        0x00 => Box::new(RomOnly::power_up(rom))
    };
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