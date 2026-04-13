use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::app::AppState;
use crate::config::ViewMode;
use crate::window;

const ZOOM_STEP: f64 = 1.15;
const ZOOM_MIN: f64 = 0.002;
const ZOOM_MAX: f64 = 2000.0;
const SCROLL_STEP: f64 = 50.0;

pub fn handle_key(state: &mut AppState, hwnd: HWND, vk: VIRTUAL_KEY) {
    match vk {
        // Quit
        VK_Q | VK_ESCAPE => unsafe {
            let _ = DestroyWindow(hwnd);
        },

        // Navigation
        VK_SPACE | VK_RIGHT if state.filelist.len() > 1 => {
            state.navigate_next();
            window::invalidate(hwnd);
        }
        VK_BACK | VK_LEFT if state.filelist.len() > 1 => {
            state.navigate_prev();
            window::invalidate(hwnd);
        }
        VK_HOME => {
            state.filelist.jump_first();
            state.load_current_image();
            window::invalidate(hwnd);
        }
        VK_END => {
            state.filelist.jump_last();
            state.load_current_image();
            window::invalidate(hwnd);
        }

        // Zoom
        VK_OEM_PLUS | VK_ADD => {
            state.zoom = (state.zoom * ZOOM_STEP).min(ZOOM_MAX);
            window::invalidate(hwnd);
        }
        VK_OEM_MINUS | VK_SUBTRACT => {
            state.zoom = (state.zoom / ZOOM_STEP).max(ZOOM_MIN);
            window::invalidate(hwnd);
        }
        // Zoom to fit
        VK_MULTIPLY => {
            state.zoom_to_fit();
            window::invalidate(hwnd);
        }

        // Scroll/pan with arrow keys when zoomed
        VK_UP => {
            state.pan_y += SCROLL_STEP;
            window::invalidate(hwnd);
        }
        VK_DOWN => {
            state.pan_y -= SCROLL_STEP;
            window::invalidate(hwnd);
        }

        // Fullscreen toggle
        VK_X | VK_F11 => {
            window::toggle_fullscreen(
                hwnd,
                &mut state.is_fullscreen,
                &mut state.saved_rect,
            );
            window::invalidate(hwnd);
        }

        // Draw filename toggle
        VK_D => {
            state.options.draw_filename = !state.options.draw_filename;
            window::invalidate(hwnd);
        }

        // Rotation
        VK_OEM_PERIOD => {
            // > key — rotate 90 CW
            state.rotation = (state.rotation + 90.0) % 360.0;
            window::invalidate(hwnd);
        }
        VK_OEM_COMMA => {
            // < key — rotate 90 CCW
            state.rotation = (state.rotation - 90.0 + 360.0) % 360.0;
            window::invalidate(hwnd);
        }

        // Flip (underscore key / slash key stand-in)
        VK_OEM_2 => {
            // / key — flip vertical
            state.flip_v = !state.flip_v;
            window::invalidate(hwnd);
        }
        VK_OEM_5 => {
            // \ key — flip horizontal
            state.flip_h = !state.flip_h;
            window::invalidate(hwnd);
        }

        // Pause slideshow
        VK_P => {
            state.paused = !state.paused;
            window::invalidate(hwnd);
        }

        // Reset zoom to 100%
        VK_0 => {
            state.zoom = 1.0;
            state.pan_x = 0.0;
            state.pan_y = 0.0;
            window::invalidate(hwnd);
        }

        _ => {}
    }
}

pub fn handle_mouse_wheel(state: &mut AppState, hwnd: HWND, delta: i16, ctrl: bool) {
    if ctrl {
        // Ctrl+wheel: zoom
        if delta > 0 {
            state.zoom = (state.zoom * ZOOM_STEP).min(ZOOM_MAX);
        } else {
            state.zoom = (state.zoom / ZOOM_STEP).max(ZOOM_MIN);
        }
    } else {
        // Wheel without ctrl: next/prev image
        if delta > 0 {
            state.navigate_prev();
        } else {
            state.navigate_next();
        }
    }
    window::invalidate(hwnd);
}

pub fn handle_mouse_down(state: &mut AppState, _hwnd: HWND, x: i32, y: i32) {
    state.mode = ViewMode::Pan;
    state.drag_start = Some((x, y));
    state.drag_pan_start = (state.pan_x, state.pan_y);
}

pub fn handle_mouse_drag(state: &mut AppState, hwnd: HWND, x: i32, y: i32) {
    if let Some((sx, sy)) = state.drag_start {
        let dx = (x - sx) as f64;
        let dy = (y - sy) as f64;
        state.pan_x = state.drag_pan_start.0 + dx;
        state.pan_y = state.drag_pan_start.1 + dy;
        window::invalidate(hwnd);
    }
}

pub fn handle_mouse_up(state: &mut AppState) {
    state.mode = ViewMode::Normal;
    state.drag_start = None;
}
