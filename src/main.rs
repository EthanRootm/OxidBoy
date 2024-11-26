
use std::fs;

use GBem::gpu::{SCREEN_H,SCREEN_W};
use GBem::motherboard::MotherBoard;
use argparse::{ArgumentParser, Store};


pub fn main() {
    let mut rom = String::from("");
    {
    let mut arg = ArgumentParser::new();
    arg.set_description("Game Boy Emulator in rust");
    arg.refer(&mut rom).add_argument("rom", argparse::Store, "Gameboy file to emulate");
    arg.parse_args_or_exit();
    }

    let mut mbrd = MotherBoard::power_up(rom);
    let name = mbrd.mmu.borrow().cartridge.title();
    let mut option = minifb::WindowOptions::default();
    option.resize = true;
    option.scale = minifb::Scale::X2;

    let mut window = minifb::Window::new(format!("Game Boy Emulator - {}", name).as_str()
    , SCREEN_W
    , SCREEN_H
    , option).unwrap();

    let mut buffer = vec![0x00; SCREEN_W * SCREEN_H];
    window.update_with_buffer(buffer.as_slice(), SCREEN_W, SCREEN_H).unwrap();

    loop {
        if !window.is_open() {
            break;
        }

        mbrd.next();


        if mbrd.check_reset_gpu() {
            let mut i: usize = 0;
            for l in mbrd.mmu.borrow().gpu.data.iter(){
                for w in l.iter() {
                    let b = u32::from(w[0]) << 16;
                    let g = u32::from(w[1]) << 8;
                    let r = u32::from(w[2]);
                    let a = 0xFF00_0000;
    
                    buffer[i] = a | b | g | r;
                    i += 1;
                }
            }
            window.update_with_buffer(buffer.as_slice(), SCREEN_W, SCREEN_H).unwrap();
        }   
        if !mbrd.cpu.flip() {
            continue;
        }

    }
    mbrd.mmu.borrow_mut().cartridge.sav();
}