<<<<<<< Updated upstream
// Note: Game BoyTM, Game Boy PocketTM, Super Game BoyTM and Game Boy ColorTM are registered trademarks of
// Nintendo CO., LTD. © 1989 to 1999 by Nintendo CO., LTD.
use GBem::gpu::{SCREEN_H, SCREEN_W};
use GBem::motherboard::MotherBoard;
use GBem::apu::Apu;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::Sample;
=======
use std::path::Path;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::surface::Surface;

use OxidBoy::gpu::{SCREEN_H, SCREEN_W};
use OxidBoy::motherboard::MotherBoard;
use OxidBoy::apu::Apu;
use cpal::Sample;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use sdl2::pixels::PixelFormatEnum;
use OxidBoy::sdl2::{load_font, update_with_buffer};
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes

fn main() {

    let mut rom = String::from("");
    let mut c_scale = 2;
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Gameboy emulator");
        ap.refer(&mut c_scale).add_option(
            &["-x", "--scale-factor"],
            argparse::Store,
            "Scale the video by a factor of 1, 2, 4, or 8",
        );
        ap.refer(&mut rom).add_argument("rom", argparse::Store, "Rom name");
        ap.parse_args_or_exit();
    }

<<<<<<< Updated upstream
    let mut mbrd = MotherBoard::power_up(rom);
    let rom_name = mbrd.mmu.borrow().cartridge.title();
=======
    // Powers up the MotherBoard
    let mut motherboard = MotherBoard::power_up(rom);
    let rom_name = motherboard.mmu.borrow().cartridge.title();

    // Creates sdl2 dependencies and unwraps them
    let sdl_context = sdl2::init()?;
    let ttf_context = sdl2::ttf::init(). map_err(|e| e.to_string())?;
    let font_path: &Path = Path::new(&"./assets/font/Font.ttf");
    let font = load_font(&ttf_context, font_path);
    let video = sdl_context.video()?;
    let mut window = video.window(format!("OxidBoy - {}", rom_name).as_str(), (SCREEN_W as u32) * _scale, (SCREEN_H as u32) * _scale)
    .position_centered()
    .build()
    .map_err(|e| e.to_string())?;

    let icon = Surface::load_bmp(Path::new("./assets/OBicon.bmp")).map_err(|e| e.to_string())?;
    window.set_icon(icon);

    let mut canvas = window.into_canvas()
    .present_vsync()
    .build()
    .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::ARGB8888, SCREEN_W as u32, SCREEN_H as u32)
    .map_err(|e| e.to_string())?;
>>>>>>> Stashed changes

    let mut option = minifb::WindowOptions::default();
    option.resize = true;
    option.scale = minifb::Scale::X2;
    let mut window =
        minifb::Window::new(format!("Gameboy - {}", rom_name).as_str(), SCREEN_W, SCREEN_H, option).unwrap();
    let mut window_buffer = vec![0x00; SCREEN_W * SCREEN_H];
    window.update_with_buffer(window_buffer.as_slice(), SCREEN_W, SCREEN_H).unwrap();

    // Initialize audio related. It is necessary to ensure that the stream object remains alive.
    let stream: cpal::Stream;
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();
        let sample_format = config.sample_format();
        let config: cpal::StreamConfig = config.into();

        let apu = Apu::power_up(config.sample_rate.0);
        let apu_data = apu.buffer.clone();
        mbrd.mmu.borrow_mut().apu = apu;

        stream = match sample_format {
            cpal::SampleFormat::F32 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                        let len = std::cmp::min(data.len() / 2, apu_data.lock().unwrap().len());
                        for (i, (data_l, data_r)) in apu_data.lock().unwrap().drain(..len).enumerate() {
                            data[i * 2 + 0] = data_l;
                            data[i * 2 + 1] = data_r;
                        }
                    },
                    move |err| println!("{}", err),
                    None,
                )
                .unwrap(),
            cpal::SampleFormat::F64 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f64], _: &cpal::OutputCallbackInfo| {
                        let len = std::cmp::min(data.len() / 2, apu_data.lock().unwrap().len());
                        for (i, (data_l, data_r)) in apu_data.lock().unwrap().drain(..len).enumerate() {
                            data[i * 2 + 0] = data_l.to_sample::<f64>();
                            data[i * 2 + 1] = data_r.to_sample::<f64>();
                        }
                    },
                    move |err| println!("{}", err),
                    None,
                )
                .unwrap(),
            _ => panic!("unreachable"),
        };
        stream.play().unwrap();
    let _ = stream;

<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
    loop {
        // Stop the program, if the GUI is closed by the user
        if !window.is_open() {
            break;
        }

        // Execute an instruction
        mbrd.next();
=======
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
=======
>>>>>>> Stashed changes
    //Change these Controls to what you want
    // TODO make this possible in the application
    let keymap = vec![
            (sdl2::keyboard::Keycode::Right, OxidBoy::joypad::Key::Right),
            (sdl2::keyboard::Keycode::Up, OxidBoy::joypad::Key::Up),
            (sdl2::keyboard::Keycode::Left, OxidBoy::joypad::Key::Left),
            (sdl2::keyboard::Keycode::Down, OxidBoy::joypad::Key::Down),
            (sdl2::keyboard::Keycode::Z, OxidBoy::joypad::Key::A),
            (sdl2::keyboard::Keycode::X, OxidBoy::joypad::Key::B),
            (sdl2::keyboard::Keycode::C, OxidBoy::joypad::Key::Select),
            (sdl2::keyboard::Keycode::V, OxidBoy::joypad::Key::Start),
        ];
    // Intialize the event punp for receiving input
    let mut event_pump = sdl_context.event_pump()?;
    let mut pause = false;
    'running: loop 
    {
        // Execute next instruction
        motherboard.next();
>>>>>>> Stashed changes

        // Update the window
        if mbrd.check_reset_gpu() {
            let mut i: usize = 0;
            for l in mbrd.mmu.borrow().gpu.data.iter() {
                for w in l.iter() {
                    let b = u32::from(w[0]) << 16;
                    let g = u32::from(w[1]) << 8;
                    let r = u32::from(w[2]);
                    let a = 0xff00_0000;

                    window_buffer[i] = a | b | g | r;
                    i += 1;
                }
            }
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
<<<<<<< Updated upstream
            window.update_with_buffer(window_buffer.as_slice(), SCREEN_W, SCREEN_H).unwrap();
=======
            let _ = update_with_buffer(&mut canvas, &mut texture, &window_buffer, SCREEN_W, pause, &texture_creator, _scale, &ttf_context, font_path);
>>>>>>> Stashed changes
=======
            let _ = update_with_buffer(&mut canvas, &mut texture, &window_buffer, SCREEN_W, pause, &texture_creator, _scale, &ttf_context, font_path);
>>>>>>> Stashed changes
=======
            let _ = update_with_buffer(&mut canvas, &mut texture, &window_buffer, SCREEN_W, pause, &texture_creator, _scale, &ttf_context, font_path);
>>>>>>> Stashed changes
=======
            let _ = update_with_buffer(&mut canvas, &mut texture, &window_buffer, SCREEN_W, pause, &texture_creator, _scale, &ttf_context, font_path);
>>>>>>> Stashed changes
=======
            let _ = update_with_buffer(&mut canvas, &mut texture, &window_buffer, SCREEN_W, pause, &texture_creator, _scale, &ttf_context, font_path);
>>>>>>> Stashed changes
        }
        

        if !mbrd.cpu.flip() {
            continue;
        }

        // Handling keyboard events
<<<<<<< Updated upstream
        if window.is_key_down(minifb::Key::Escape) {
            break;
        }
        let keys = vec![
            (minifb::Key::D, GBem::joypad::Key::Right),
            (minifb::Key::W, GBem::joypad::Key::Up),
            (minifb::Key::A, GBem::joypad::Key::Left),
            (minifb::Key::S, GBem::joypad::Key::Down),
            (minifb::Key::Right, GBem::joypad::Key::A),
            (minifb::Key::Left, GBem::joypad::Key::B),
            (minifb::Key::Space, GBem::joypad::Key::Select),
            (minifb::Key::Enter, GBem::joypad::Key::Start),
        ];
        for (rk, vk) in &keys {
            if window.is_key_down(*rk) {
                mbrd.mmu.borrow_mut().joypad.keydown(vk.clone());
            } else {
                mbrd.mmu.borrow_mut().joypad.keyup(vk.clone());
=======
        for event in event_pump.poll_iter() {
            match event {
                // Breaks loop if escape is pressed or program is exited
                Event::Quit { .. } => break 'running,
                // Uses keymap to use inputed key as a GB Button and set it in motherboard
                Event::KeyDown { keycode: Some(key), .. } => {
                    if let Some((_, gbkey)) = keymap.iter().find(|(k, _)| *k == key) {
                        motherboard.mmu.borrow_mut().joypad.keydown(gbkey.clone());
                    }
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    if let Some((_, gbkey)) = keymap.iter().find(|(k, _)| *k == key) {
                        motherboard.mmu.borrow_mut().joypad.keyup(gbkey.clone());
                    }
                }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    pause = !pause; 
                }
                _ => {}
>>>>>>> Stashed changes
            }
        }
        if event_pump.keyboard_state().is_scancode_pressed(sdl2::keyboard::Scancode::LCtrl) && event_pump.keyboard_state().is_scancode_pressed(sdl2::keyboard::Scancode::P) {
            pause = true;
        }
    }

    mbrd.mmu.borrow_mut().cartridge.sav();
}
