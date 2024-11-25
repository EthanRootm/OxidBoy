
use std::fs;

use GBem::gpu::{SCREEN_H,SCREEN_W};
use GBem::motherboard::MotherBoard;
use argparse::{ArgumentParser, Store};


pub fn main() {
    /*let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    */
    let mut rom = String::from("");
    {
    let mut arg = ArgumentParser::new();
    arg.set_description("Game Boy Emulator using SDL2 in rust");
    arg.refer(&mut rom).add_argument("rom", argparse::Store, "Gameboy file to emulate");
    arg.parse_args_or_exit();
    }

    let mut mbrd = MotherBoard::power_up(rom);
    let name = mbrd.mmu.borrow().cartridge.title();

    let mut title = String::from("Game Boy Emulator - ");
    title.push_str(&name);

    let mut window = minifb::Window::new(format!("Game Boy Emulator - {}", name).as_str()
    , SCREEN_W
    , SCREEN_H
    , minifb::WindowOptions::default()).unwrap();

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
    
    /*let window = video_subsystem
        .window(title.as_str(), SCREEN_W as u32, SCREEN_H as u32)
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let mut buffer = vec![0x00; SCREEN_W * SCREEN_H];

    'main: loop {
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
            
            
            sdl_texture
        }

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                _ => {}
            }
        }
    } */
}