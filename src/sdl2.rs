use std::path::Path;
use sdl2::{pixels::Color, render::{Canvas, Texture, TextureCreator}, video::{Window, WindowContext}};
use sdl2::ttf::{Font, Sdl2TtfContext};
use crate::gpu::{SCREEN_H, SCREEN_W};

// Render
pub fn update_with_buffer(
    canvas: &mut Canvas<Window>,
    texture: &mut Texture,
    window_buffer: &[u32],
    screen_w: usize,
    pause: bool,
    texturecreate: &TextureCreator<WindowContext>,
    scale: u32,
    ttf_context: &Sdl2TtfContext,
    path: &Path,
    
) -> Result<(), String> {
    if !pause {
    texture.update(None,
        bytemuck::cast_slice(window_buffer),
        screen_w * 4
    ).map_err(|e| e.to_string())?;


    canvas.clear();
    canvas.copy(texture, None, None)?;
} else {

    let font = load_font(ttf_context, path);
    let _ = render_pause(canvas, font, texturecreate, scale);
}
    canvas.present();

    Ok(())
}

fn render_pause(
    canvas: &mut Canvas<Window>,
    font: sdl2::ttf::Font,
    texturecreate: &TextureCreator<WindowContext>,
    scale: u32,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGBA(0, 0, 0, 128));
    let rect = sdl2::rect::Rect::new(0, 0, (SCREEN_W as u32) * scale, (SCREEN_H as u32) * scale);
    canvas.fill_rect(rect)?;

    let surface =font.render("Paused").blended(Color::RGBA(255, 255, 255, 255)).unwrap();
    let target = sdl2::rect::Rect::new(300, 250, surface.width(), surface.height());
    let texture = texturecreate.create_texture_from_surface(&surface).unwrap();
    canvas.copy(&texture, None, Some(target)).unwrap();
    Ok(())
}

pub fn load_font<'a>(ttf_context: &'a Sdl2TtfContext, path: &Path) -> Font<'a, 'static> {
    ttf_context.load_font(path, 32).unwrap()
}