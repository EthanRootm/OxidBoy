
use sdl2::{render::{Canvas, Texture}, video::Window};

pub fn update_with_buffer(
    canvas: &mut Canvas<Window>,
    texture: &mut Texture,
    window_buffer: &[u32],
    screen_w: usize,
) -> Result<(), String> {
    texture.update(None,
        bytemuck::cast_slice(window_buffer),
        screen_w * 4
    ).map_err(|e| e.to_string())?;

    canvas.clear();
    canvas.copy(texture, None, None)?;
    canvas.present();

    Ok(())
}