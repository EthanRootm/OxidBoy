use std::path::Path;
use sdl2::{keyboard::Keycode, pixels::Color, rect::Rect};
use sdl2::render::{Canvas, Texture, TextureCreator}; 
use sdl2::video::{Window, WindowContext};
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::mouse::MouseButton;
use sdl2::event::Event;
use crate::{gpu::{SCREEN_H, SCREEN_W}, joypad};

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

pub fn render_pause<'a>(
    canvas: &mut Canvas<Window>,
    texturecreate: &TextureCreator<WindowContext>,
    scale: u32,
    event_pump: &mut sdl2::EventPump,
    ttf_context: &'a Sdl2TtfContext
) -> Result<u32, String> {
    let path: &Path = Path::new(&"./assets/fonts/font.ttf");
    let font = load_font(ttf_context, path);

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
                    } else if press_button(x, y, setting_target.x, setting_target.y, setting_target.w, setting_target.h) {
                        return Ok(2);
                    }
                }
            }
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return Ok(3);
            }
            _ => {}
        }
    }
    Ok(0)
}

 ///  Struct holding Settings of user
 /// 
 ///  Holds the controls of this current run time's user
 /// 
 /// # Fields
 /// - controls, A saved keymap for controls to function
 /// 
 /// # Methods
 /// - set_control: Used to inialize the keymap
 /// - update_key: Used to update the keymap
pub struct Settings {
    pub controls: Vec<(Keycode, joypad::Key)>,
}

impl Settings {
    pub fn set_control(keymap: Vec<(Keycode, joypad::Key)>) -> Self {
        Settings {
            controls : keymap,
        }
    }

 ///  Updates the keybinds with inputted key
 /// 
 ///  This function looks for the given keyname then finds where that is in the keymap changing it to be new_key for the session
 /// 
 /// # Parameters
 /// - self, Used to edit the session's controls
 /// - key_name, The name of the key that is to be changed
 /// - new_key, The keycode for the key the user wanted to switch with existing
 /// 
 /// # Panics
 /// This function panics when the funtion is given a key name that is not available
    pub fn update_key(&mut self, key_name: &str, new_key: Keycode) -> bool {
        let keycode;
        match key_name {
            "Right" => keycode = joypad::Key::Right,
            "Up" => keycode = joypad::Key::Up,
            "Left" => keycode = joypad::Key::Left,
            "Down" => keycode = joypad::Key::Down,
            "A" => keycode =  joypad::Key::A,
            "B" => keycode = joypad::Key::B,
            "Select" => keycode = joypad::Key::Select,
            "Start" => keycode = joypad::Key::Start,
            _ => unreachable!()
        }
        if let Some(control) = self.controls.iter_mut().find(|(_, emucode)|{
            *emucode == keycode })
            {
                control.0 = new_key;
                return true;
        } else {
            return false;
        }
    }
}

 ///  Struct holding screen data
 /// 
 ///  Holds the context of text and the buttons for the settings screen
 /// 
 /// # Fields
 /// - settings, A copy of the Settings struct used to store the controls of the emulator
 /// - ttf_context, The context used to form text on screen
 /// - change_control, exit, Buttons to be displayed on the screen
 /// 
 /// # Methods
 /// - power_up: Used to inialize the buttons and context
 /// - render_settings: Used to render settings screen
 /// - draw_button: Used to draw buttons onto the screen
 /// - handle_events: Used to get input from buttons
 /// - await_key: Used to get the keys the user wants to change
pub struct SettingScreen<'a> {
    pub settings: Settings,
    ttf_context: &'a Sdl2TtfContext,
    change_control: Rect,
    exit: Rect,
}

impl <'a>SettingScreen<'a>{
    pub fn power_up(settings: Settings, ttf_context: &'a Sdl2TtfContext) -> Self {
        SettingScreen {settings, ttf_context, change_control:Rect::new(100, 50, 150, 75), exit:Rect::new(100, 150, 50, 50)}
    }

 ///  Render the settings screen 
 /// 
 ///  Renders the screen and buttons to scale whith window
    pub fn render_settings(
        &mut self,
        canvas: &mut Canvas<Window>,
        scale: u32,
    ) {
        canvas.set_draw_color(Color::RGBA(0, 0, 0, 255));
        let rect = sdl2::rect::Rect::new(0, 0, (SCREEN_W as u32) * scale, (SCREEN_H as u32) * scale);
        canvas.fill_rect(rect).unwrap();

        self.draw_button(canvas, &self.change_control, "Change control");
        self.draw_button(canvas, &self.exit, "Leave");

        canvas.present();
    }

    fn draw_button(&self, canvas: &mut Canvas<Window>, rect: &Rect, text: &str) {
        canvas.set_draw_color(Color::RGBA(100, 100, 100, 255));
        canvas.fill_rect(*rect).unwrap();

        let font = load_font(&self.ttf_context, Path::new("./assets/fonts/font.ttf"));
        let surface = font.render(text).blended(Color::RGBA(255, 255, 255, 255)).unwrap();
        let texturecreate = canvas.texture_creator();
        let texture = texturecreate.create_texture_from_surface(&surface).unwrap();
        canvas.copy(&texture, None, Some(*rect)).unwrap();
    }

    pub fn handle_events(&mut self, event_pump: &mut sdl2::EventPump) -> bool {
        loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return false;
                }
                Event::MouseButtonDown { x, y, .. } => {
                    if self.change_control.contains_point((x,y)) {
                        return true;
                    } else if self.exit.contains_point((x,y)) {
                        return false;
                    }
                }
                _ => {}
            }
        }   
    }
    }

 ///  Waits for each control to get a new key
 /// 
 ///  With every key available draw a button to ask the user what keys they want and sets them
 /// 
 /// # Parameters
 /// - self, Used to edit the session's controls
 /// - canvas, used to draw buttons on the screen
 /// - event_pump, Used to gather user input and keycodes
 /// 
 /// # Panics
 /// This function panics when the funtion is given a key name that is not available
    pub fn await_key(&mut self, canvas: &mut Canvas<Window>, event_pump: &mut sdl2::EventPump) {
        let rotation = vec!["Right", "Up", "Left", "Down", "A", "B", "Select", "Start"];

        for keys in rotation{
            canvas.clear();
            let text = format!("Changing {}", keys);
            self.draw_button(canvas, &self.change_control, &text);
            canvas.present();

            let mut keyChanged = false;
            while !keyChanged {
                for event in event_pump.poll_iter() {
                    if let Event::KeyDown { keycode: Some(key), .. } = event{
                        if self.settings.update_key(&keys, key) {
                            keyChanged = true;
                        } else {
                            eprintln!("Something went wrong please try again");
                        }
                        
                    }
                }
            }
            
        }
    }
}

pub fn press_button(x: i32, y: i32, button_x: i32, button_y: i32, width: i32, height: i32) -> bool {
    x >= button_x && x <= button_x + width &&
    y >= button_y && y <= button_y + height
}

pub fn load_font<'a>(ttf_context: &'a Sdl2TtfContext, path: &Path) -> Font<'a, 'static> {
    ttf_context.load_font(path, 32).unwrap()
}