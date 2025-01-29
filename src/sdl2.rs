use std::path::Path;
use sdl2::{keyboard::Keycode, pixels::Color, render::{Canvas, Texture, TextureCreator}, video::{Window, WindowContext}};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::mouse::MouseButton;
use sdl2::event::Event;
use crate::gpu::{SCREEN_H, SCREEN_W};

 /// # Updates the Screen with a ARGB buffer
 /// * Canvas, The screen to be updated
 /// * Texture, The data that gets casted to canvas
 /// * window_buffer, The ARGB window buffer to update texture
pub fn update_with_buffer(
    canvas: &mut Canvas<Window>,
    texture: &mut Texture,
    window_buffer: &[u32],
) -> Result<(), String> {
    texture.update(None,
        bytemuck::cast_slice(window_buffer),
        SCREEN_W * 4
    ).map_err(|e| e.to_string())?;


    // clear screen
    canvas.clear();
    // copy gameboy graphics
    canvas.copy(texture, None, None)?;
    // display graphics
    canvas.present();
    // confirm that the program successful finished 
    Ok(())
}

pub fn render_pause(
    canvas: &mut Canvas<Window>,
    texturecreate: &TextureCreator<WindowContext>,
    scale: u32,
    event_pump: &mut sdl2::EventPump,
) -> Result<u32, String> {
    let ttf_context = sdl2::ttf::init(). map_err(|e| e.to_string())?;
    let path: &Path = Path::new(&"./assets/fonts/font.ttf");
    let font = load_font(&ttf_context, path);

    canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
    let rect = sdl2::rect::Rect::new(0, 0, (SCREEN_W as u32) * scale, (SCREEN_H as u32) * scale);
    canvas.fill_rect(rect)?;

    let pause_surface = font.render("Paused").blended(Color::RGBA(100, 100, 100, 255)).unwrap();
    let x = ((SCREEN_W as u32 * scale) - pause_surface.width()) / 2;
    let pause_target = sdl2::rect::Rect::new(x as i32, 10, pause_surface.width(), pause_surface.height());
    let pause_texture = texturecreate.create_texture_from_surface(&pause_surface).unwrap();
    canvas.copy(&pause_texture, None, Some(pause_target)).unwrap();

    let quit_surface = font.render("Quit").blended(Color::RGBA(150, 100, 150, 255)).unwrap();
    let x = ((SCREEN_W as u32 * scale) - quit_surface.width()) / 2;
    let quit_target = sdl2::rect::Rect::new(x as i32, 200 + pause_surface.height() as i32, quit_surface.width(), quit_surface.height());
    let quit_texture = texturecreate.create_texture_from_surface(&quit_surface).unwrap();
    canvas.copy(&quit_texture, None, Some(quit_target)).unwrap();

    let setting_surface = font.render("Settings").blended(Color::RGBA(100, 100, 100, 255)).unwrap();
    let x = ((SCREEN_W as u32 * scale) - setting_surface.width()) / 2;
    let setting_target = sdl2::rect::Rect::new(x as i32, 50 + pause_surface.height() as i32, setting_surface.width(), setting_surface.height());
    let setting_texture = texturecreate.create_texture_from_surface(&setting_surface).unwrap();
    canvas.copy(&setting_texture, None, Some(setting_target)).unwrap();

    canvas.present();

    for event in event_pump.poll_iter() {
        match event {
            Event::MouseButtonDown { x, y, mouse_btn, ..} => {
                if mouse_btn == MouseButton::Left {
                    // Quit Button
                    if press_button(x, y, quit_target.x, quit_target.y, quit_target.w, quit_target.h) {
                        return Ok(1);
                    }
                }
            }
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return Ok(2);
            }
            _ => {}
        }
    }
    Ok(0)
}

pub fn press_button(x: i32, y: i32, button_x: i32, button_y: i32, width: i32, height: i32) -> bool {
    x >= button_x && x <= button_x + width &&
    y >= button_y && y <= button_y + height
}

pub fn load_font<'a>(ttf_context: &'a Sdl2TtfContext, path: &Path) -> Font<'a, 'static> {
    ttf_context.load_font(path, 32).unwrap()
}