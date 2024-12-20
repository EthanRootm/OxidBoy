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
use OxidBoy::sdl2::update_with_buffer;


fn main() -> Result<(), String> {

    let mut rom = String::from("");
    let mut _scale = 2;
    // Sets up argument parser to get rom location
    {
        let mut ap = argparse::ArgumentParser::new();
        ap.set_description("Gameboy emulator");
        ap.refer(&mut _scale).add_option(
            &["-s", "--scale"],
            argparse::Store,
            "Scale the Window",
        );
        ap.refer(&mut rom).add_argument("rom", argparse::Store, "Rom name");
        ap.parse_args_or_exit();
    }

    // Powers up the MotherBoard
    let mut motherboard = MotherBoard::power_up(rom);
    let rom_name = motherboard.mmu.borrow().cartridge.title();

    // Creates sdl2 dependencies and unwraps them
    let sdl_context = sdl2::init()?;
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

    let mut window_buffer = vec![0x00; SCREEN_W * SCREEN_H];


    // Initialize audio related. It is necessary to ensure that the stream object remains alive.
    let stream: cpal::Stream;
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config = device.default_output_config().unwrap();
        let sample_format = config.sample_format();
        let config: cpal::StreamConfig = config.into();

        let apu = Apu::power_up(config.sample_rate.0);
        let apu_data = apu.buffer.clone();
        motherboard.mmu.borrow_mut().apu = apu;

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

    let keymap = vec![
            (sdl2::keyboard::Keycode::Right, OxidBoy::joypad::Key::Right),
            (sdl2::keyboard::Keycode::UP, OxidBoy::joypad::Key::Up),
            (sdl2::keyboard::Keycode::Left, OxidBoy::joypad::Key::Left),
            (sdl2::keyboard::Keycode::Down, OxidBoy::joypad::Key::Down),
            (sdl2::keyboard::Keycode::Z, OxidBoy::joypad::Key::A),
            (sdl2::keyboard::Keycode::X, OxidBoy::joypad::Key::B),
            (sdl2::keyboard::Keycode::C, OxidBoy::joypad::Key::Select),
            (sdl2::keyboard::Keycode::V, OxidBoy::joypad::Key::Start),
        ];
    // Intialize the event punp for receiving input
    let mut event_pump = sdl_context.event_pump()?;
    'running: loop 
    {
        // Execute next instruction
        motherboard.next();

        // Update the window
        if motherboard.check_reset_gpu() {
            let mut i: usize = 0;
            for l in motherboard.mmu.borrow().gpu.data.iter() {
                for w in l.iter() {
                    let b = u32::from(w[0]) << 16;
                    let g = u32::from(w[1]) << 8;
                    let r = u32::from(w[2]);
                    let a = 0xff00_0000;

                    window_buffer[i] = a | r | g | b ;
                    i += 1;
                }
            }
            let _ = update_with_buffer(&mut canvas, &mut texture, &window_buffer, SCREEN_W);
        }
        

        if !motherboard.cpu.flip() {
            continue;
        }

        // Handling keyboard events
        for event in event_pump.poll_iter() {
            match event {
                // Breaks loop if escape is pressed or program is exited
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
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
                _ => {}
            }
        }
    }
    // Save all data on application end
    motherboard.mmu.borrow_mut().cartridge.sav();
    Ok(())
}

