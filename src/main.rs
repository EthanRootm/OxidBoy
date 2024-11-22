use std::fs;

use GBem::gpu::{SCREEN_H,SCREEN_W};
use GBem::motherboard::MotherBoard;
use argparse::{ArgumentParser, Store};

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Rect};

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let mut rom = "world".to_string();
    {
    let mut arg = ArgumentParser::new();
    arg.refer(&mut rom).add_argument("rom", argparse::Store, "Gameboy file to emulate");
    arg.parse_args_or_exit();
    }
    print!("{}", rom);

    let mbrd = MotherBoard::power_up(rom);
    let name = mbrd.mmu.borrow().cartridge.title();

    let mut title = String::from("Game Boy Emulator - ");
    title.push_str(&name);
    
    let window = video_subsystem
        .window(title.as_str(), SCREEN_W as u32, SCREEN_H as u32)
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    'main: loop {
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
    }

    Ok(())
}