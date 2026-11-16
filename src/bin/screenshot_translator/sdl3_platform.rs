use std::time::Instant;

use imgui::{BackendFlags, Context, MouseButton};
use sdl3::event::Event;
use sdl3::mouse::MouseButton as SdlMouseButton;
use sdl3::video::Window;

pub struct Platform {
    last_frame: Instant,
}

impl Platform {
    pub fn new(context: &mut Context) -> Self {
        let io = context.io_mut();
        io.backend_flags.insert(BackendFlags::HAS_MOUSE_CURSORS);
        io.backend_flags.insert(BackendFlags::HAS_SET_MOUSE_POS);
        context.set_platform_name(Some("screenshot-translator-sdl3".to_owned()));
        Self {
            last_frame: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, context: &mut Context, event: &Event) {
        let io = context.io_mut();
        match *event {
            Event::MouseMotion { x, y, .. } => io.add_mouse_pos_event([x, y]),
            Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                io.add_mouse_pos_event([x, y]);
                handle_mouse_button(io, mouse_btn, true);
            }
            Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                io.add_mouse_pos_event([x, y]);
                handle_mouse_button(io, mouse_btn, false);
            }
            Event::MouseWheel { y, .. } => io.add_mouse_wheel_event([0.0, y]),
            Event::TextInput { ref text, .. } => {
                for character in text.chars() {
                    io.add_input_character(character);
                }
            }
            _ => {}
        }
    }

    pub fn prepare_frame(&mut self, context: &mut Context, window: &Window) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_frame);
        self.last_frame = now;
        let (logical_width, logical_height) = window.size();
        let (pixel_width, pixel_height) = window.size_in_pixels();
        let io = context.io_mut();
        io.update_delta_time(elapsed);
        io.display_size = [logical_width as f32, logical_height as f32];
        io.display_framebuffer_scale = [
            pixel_width as f32 / logical_width.max(1) as f32,
            pixel_height as f32 / logical_height.max(1) as f32,
        ];
    }
}

fn handle_mouse_button(io: &mut imgui::Io, button: SdlMouseButton, pressed: bool) {
    let button = match button {
        SdlMouseButton::Left => MouseButton::Left,
        SdlMouseButton::Right => MouseButton::Right,
        SdlMouseButton::Middle => MouseButton::Middle,
        SdlMouseButton::X1 => MouseButton::Extra1,
        SdlMouseButton::X2 => MouseButton::Extra2,
        _ => return,
    };
    io.add_mouse_button_event(button, pressed);
}
