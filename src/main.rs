use sdl2::event::Event;
use sdl2::keyboard::Keycode;
// Note: Game BoyTM, Game Boy PocketTM, Super Game BoyTM and Game Boy ColorTM are registered trademarks of
// Nintendo CO., LTD. © 1989 to 1999 by Nintendo CO., LTD.
use GBem::gpu::{SCREEN_H, SCREEN_W};
use GBem::motherboard::MotherBoard;
use GBem::apu::Apu;
use sdl2::pixels::PixelFormatEnum;
use GBem::render::update_with_buffer;


fn main() -> Result<(), String> {

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

    let mut mbrd = MotherBoard::power_up(rom);
    let rom_name = mbrd.mmu.borrow().cartridge.title();

    let sdl_context = sdl2::init()?;
    let video = sdl_context.video()?;

    let window = video.window(format!("Gameboy - {}", rom_name).as_str(), (SCREEN_W as u32) * c_scale, (SCREEN_H as u32) * c_scale)
    .position_centered()
    .build()
    .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas()
    .present_vsync()
    .build()
    .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::ABGR8888, SCREEN_W as u32, SCREEN_H as u32)
    .map_err(|e| e.to_string())?;

    let mut window_buffer = vec![0x00; SCREEN_W * SCREEN_H];

    /*
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
    */

    let keymap = vec![
            (sdl2::keyboard::Keycode::D, GBem::joypad::Key::Right),
            (sdl2::keyboard::Keycode::W, GBem::joypad::Key::Up),
            (sdl2::keyboard::Keycode::A, GBem::joypad::Key::Left),
            (sdl2::keyboard::Keycode::S, GBem::joypad::Key::Down),
            (sdl2::keyboard::Keycode::RIGHT, GBem::joypad::Key::A),
            (sdl2::keyboard::Keycode::LEFT, GBem::joypad::Key::B),
            (sdl2::keyboard::Keycode::SPACE, GBem::joypad::Key::Select),
            (sdl2::keyboard::Keycode::KP_ENTER, GBem::joypad::Key::Start),
        ];

    let mut event_pump = sdl_context.event_pump()?;
    'running: loop 
    {
        // Execute an instruction
        mbrd.next();

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
            update_with_buffer(&mut canvas, &mut texture, &window_buffer, SCREEN_W, SCREEN_H);
        }
        

        if !mbrd.cpu.flip() {
            continue;
        }

        // Handling keyboard events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(key), .. } => {
                    if let Some((_, gbkey)) = keymap.iter().find(|(k, _)| *k == key) {
                        mbrd.mmu.borrow_mut().joypad.keydown(gbkey.clone());
                    }
                }
                Event::KeyUp { keycode: Some(key), .. } => {
                    if let Some((_, gbkey)) = keymap.iter().find(|(k, _)| *k == key) {
                        mbrd.mmu.borrow_mut().joypad.keyup(gbkey.clone());
                    }
                }
                _ => {}
            }
        }
    }
    mbrd.mmu.borrow_mut().cartridge.sav();
    Ok(())
}

