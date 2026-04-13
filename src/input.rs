use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::actions;
use crate::app::AppState;
use crate::config::ViewMode;
use crate::keybindings::Action;
use crate::transforms::{ZOOM_MAX, ZOOM_MIN};
use crate::window;

const ZOOM_STEP: f64 = 1.15;
const SCROLL_STEP: f64 = 50.0;

/// Check if Ctrl key is currently pressed
fn ctrl_pressed() -> bool {
    unsafe { GetKeyState(VK_CONTROL.0 as i32) as u16 & 0x8000 != 0 }
}

/// Check if Shift key is currently pressed
fn shift_pressed() -> bool {
    unsafe { GetKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000 != 0 }
}

pub fn handle_key(state: &mut AppState, hwnd: HWND, vk: VIRTUAL_KEY) {
    // --- Thumbnail mode key handling ---
    if state.thumbnail_view.is_some() {
        handle_thumbnail_key(state, hwnd, vk);
        return;
    }

    // Ctrl+Shift+> : lossless CW rotation
    if vk == VK_OEM_PERIOD && ctrl_pressed() && shift_pressed() {
        if let Some(file) = state.filelist.current() {
            let path = file.path.clone();
            let transform = crate::jpeg_rotate::RotateDirection::Clockwise90.to_wic_transform();
            match state.image_loader.save_rotated(&path, transform) {
                Ok(()) => {
                    state.load_current_image();
                    window::invalidate(hwnd);
                }
                Err(e) => eprintln!("Lossless rotate failed: {e}"),
            }
        }
        return;
    }
    // Ctrl+Shift+< : lossless CCW rotation
    if vk == VK_OEM_COMMA && ctrl_pressed() && shift_pressed() {
        if let Some(file) = state.filelist.current() {
            let path = file.path.clone();
            let transform =
                crate::jpeg_rotate::RotateDirection::CounterClockwise90.to_wic_transform();
            match state.image_loader.save_rotated(&path, transform) {
                Ok(()) => {
                    state.load_current_image();
                    window::invalidate(hwnd);
                }
                Err(e) => eprintln!("Lossless rotate failed: {e}"),
            }
        }
        return;
    }

    // 'z' key: enter/exit Zoom mode
    if vk == VK_Z && !ctrl_pressed() && !shift_pressed() {
        if state.mode == ViewMode::Zoom {
            state.mode = ViewMode::Normal;
        } else {
            state.mode = ViewMode::Zoom;
            state.drag_start = None;
        }
        window::invalidate(hwnd);
        return;
    }

    // 'r' key: enter/exit Rotate mode
    if vk == VK_R && !ctrl_pressed() && !shift_pressed() {
        if state.mode == ViewMode::Rotate {
            state.mode = ViewMode::Normal;
        } else {
            state.mode = ViewMode::Rotate;
            state.drag_start = None;
        }
        window::invalidate(hwnd);
        return;
    }

    // Escape exits interactive modes before quitting
    if vk == VK_ESCAPE && (state.mode == ViewMode::Zoom || state.mode == ViewMode::Rotate) {
        state.mode = ViewMode::Normal;
        state.drag_start = None;
        window::invalidate(hwnd);
        return;
    }
    // Check numbered action keys (1-9) — these override keybinding map when an action is set
    let action_index = match vk {
        VK_1 => Some(0usize),
        VK_2 => Some(1),
        VK_3 => Some(2),
        VK_4 => Some(3),
        VK_5 => Some(4),
        VK_6 => Some(5),
        VK_7 => Some(6),
        VK_8 => Some(7),
        VK_9 => Some(8),
        _ => None,
    };

    if let Some(idx) = action_index
        && let Some(Some(action_str)) = state.numbered_actions.get(idx)
    {
        if let Some(file) = state.filelist.current() {
            let file_clone = file.clone();
            let action_str = action_str.clone();
            let index = state.filelist.current_index();
            let total = state.filelist.len();
            actions::execute_action(&action_str, &file_clone, index, total);
        }
        return;
    }

    // Default action on Enter
    if vk == VK_RETURN {
        if let Some(ref action_str) = state.options.action.clone()
            && let Some(file) = state.filelist.current()
        {
            let file_clone = file.clone();
            let index = state.filelist.current_index();
            let total = state.filelist.len();
            actions::execute_action(action_str, &file_clone, index, total);
        }
        return;
    }

    // Look up action from keybinding map
    let action = match state.keybindings.get(&vk.0) {
        Some(a) => *a,
        None => return,
    };

    dispatch_action(action, state, hwnd);
}

fn dispatch_action(action: Action, state: &mut AppState, hwnd: HWND) {
    match action {
        Action::Quit => unsafe {
            let _ = DestroyWindow(hwnd);
        },
        Action::Next if state.filelist.len() > 1 => {
            state.navigate_next();
            window::invalidate(hwnd);
        }
        Action::Prev if state.filelist.len() > 1 => {
            state.navigate_prev();
            window::invalidate(hwnd);
        }
        Action::JumpFirst => {
            state.filelist.jump_first();
            state.load_current_image();
            window::invalidate(hwnd);
        }
        Action::JumpLast => {
            state.filelist.jump_last();
            state.load_current_image();
            window::invalidate(hwnd);
        }
        Action::JumpForward => {
            state.filelist.jump_forward(5);
            state.load_current_image();
            window::invalidate(hwnd);
        }
        Action::JumpBack => {
            state.filelist.jump_back(5);
            state.load_current_image();
            window::invalidate(hwnd);
        }
        Action::ZoomIn => {
            state.zoom = (state.zoom * ZOOM_STEP).min(ZOOM_MAX);
            window::invalidate(hwnd);
        }
        Action::ZoomOut => {
            state.zoom = (state.zoom / ZOOM_STEP).max(ZOOM_MIN);
            window::invalidate(hwnd);
        }
        Action::FitWindow => {
            state.zoom_to_fit();
            window::invalidate(hwnd);
        }
        Action::ActualSize => {
            state.zoom = 1.0;
            state.pan_x = 0.0;
            state.pan_y = 0.0;
            window::invalidate(hwnd);
        }
        Action::ScrollUp => {
            state.pan_y += SCROLL_STEP;
            window::invalidate(hwnd);
        }
        Action::ScrollDown => {
            state.pan_y -= SCROLL_STEP;
            window::invalidate(hwnd);
        }
        Action::ToggleFullscreen => {
            window::toggle_fullscreen(hwnd, &mut state.is_fullscreen, &mut state.saved_rect);
            window::invalidate(hwnd);
        }
        Action::ToggleFilename => {
            state.options.draw_filename = !state.options.draw_filename;
            window::invalidate(hwnd);
        }
        Action::ToggleInfo => {
            state.options.draw_info = !state.options.draw_info;
            window::invalidate(hwnd);
        }
        Action::RotateCW => {
            state.rotation = (state.rotation + 90.0) % 360.0;
            window::invalidate(hwnd);
        }
        Action::RotateCCW => {
            state.rotation = (state.rotation - 90.0 + 360.0) % 360.0;
            window::invalidate(hwnd);
        }
        Action::FlipV => {
            state.flip_v = !state.flip_v;
            window::invalidate(hwnd);
        }
        Action::FlipH => {
            state.flip_h = !state.flip_h;
            window::invalidate(hwnd);
        }
        Action::Pause => {
            state.paused = !state.paused;
            window::invalidate(hwnd);
        }
        Action::Delete => {
            state.remove_current_from_list(hwnd);
        }
        Action::Wallpaper => {
            if let Some(file) = state.filelist.current() {
                let path = file.path.clone();
                let mode = state.options.wallpaper_mode();
                match crate::wallpaper::set_wallpaper(&path, mode) {
                    Ok(()) => {}
                    Err(e) => eprintln!("Failed to set wallpaper: {e}"),
                }
            }
        }
        Action::Save => {
            let save_path = state.filelist.current().map(|f| {
                let stem = f
                    .path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned();
                let ext = f
                    .path
                    .extension()
                    .map(|e| e.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let new_name = if ext.is_empty() {
                    format!("{}_copy", stem)
                } else {
                    format!("{}_copy.{}", stem, ext)
                };
                f.path
                    .parent()
                    .unwrap_or(std::path::Path::new("."))
                    .join(&new_name)
            });

            if let Some(save_path) = save_path
                && let Some(ref image) = state.current_image
            {
                match state.image_loader.save(image, &save_path) {
                    Ok(()) => eprintln!("Saved: {}", save_path.display()),
                    Err(e) => eprintln!("Save failed: {e}"),
                }
            }
        }
        _ => {}
    }
}

pub fn handle_mouse_wheel(state: &mut AppState, hwnd: HWND, delta: i16, ctrl: bool) {
    // Thumbnail mode scrolling
    if let Some(ref mut thumb_view) = state.thumbnail_view {
        let scroll_amount = if delta > 0 { -60.0 } else { 60.0 };
        let file_count = state.filelist.len();
        let mut rect = RECT::default();
        unsafe {
            let _ = GetClientRect(hwnd, &mut rect);
        }
        let vw = (rect.right - rect.left) as f32;
        let vh = (rect.bottom - rect.top) as f32;
        thumb_view.scroll(scroll_amount, file_count, vw, vh);
        window::invalidate(hwnd);
        return;
    }

    if ctrl {
        if delta > 0 {
            state.zoom = (state.zoom * ZOOM_STEP).min(ZOOM_MAX);
        } else {
            state.zoom = (state.zoom / ZOOM_STEP).max(ZOOM_MIN);
        }
    } else {
        if delta > 0 {
            state.navigate_prev();
        } else {
            state.navigate_next();
        }
    }
    window::invalidate(hwnd);
}

pub fn handle_mouse_down(state: &mut AppState, hwnd: HWND, x: i32, y: i32) {
    // Thumbnail mode click
    if let Some(ref mut thumb_view) = state.thumbnail_view {
        let mut rect = RECT::default();
        unsafe {
            let _ = GetClientRect(hwnd, &mut rect);
        }
        let vw = (rect.right - rect.left) as f32;
        if let Some(idx) = thumb_view.handle_click(x as f32, y as f32, vw)
            && idx < state.filelist.len()
        {
            thumb_view.selected = idx;
            window::invalidate(hwnd);
        }
        return;
    }

    // In Zoom/Rotate mode, start drag but keep the mode
    if state.mode == ViewMode::Zoom || state.mode == ViewMode::Rotate {
        state.drag_start = Some((x, y));
        state.drag_pan_start = (state.zoom, state.rotation);
        return;
    }

    state.mode = ViewMode::Pan;
    state.drag_start = Some((x, y));
    state.drag_pan_start = (state.pan_x, state.pan_y);
}

pub fn handle_mouse_drag(state: &mut AppState, hwnd: HWND, x: i32, y: i32) {
    if state.thumbnail_view.is_some() {
        return;
    }

    if let Some((sx, sy)) = state.drag_start {
        match state.mode {
            ViewMode::Zoom => {
                // Drag up = zoom in, drag down = zoom out
                let dy = (sy - y) as f64;
                let factor = 1.0 + dy * 0.005;
                let base_zoom = state.drag_pan_start.0;
                state.zoom = (base_zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX);
                window::invalidate(hwnd);
            }
            ViewMode::Rotate => {
                // Drag left/right = rotate
                let dx = (x - sx) as f64;
                let base_rot = state.drag_pan_start.1;
                state.rotation = (base_rot + dx * 0.5) % 360.0;
                if state.rotation < 0.0 {
                    state.rotation += 360.0;
                }
                window::invalidate(hwnd);
            }
            _ => {
                // Pan mode
                let dx = (x - sx) as f64;
                let dy = (y - sy) as f64;
                state.pan_x = state.drag_pan_start.0 + dx;
                state.pan_y = state.drag_pan_start.1 + dy;
                window::invalidate(hwnd);
            }
        }
    }
}

pub fn handle_mouse_up(state: &mut AppState) {
    state.drag_start = None;
    // Only reset mode to Normal if we were in Pan mode
    if state.mode == ViewMode::Pan {
        state.mode = ViewMode::Normal;
    }
}

/// Handle keys in thumbnail/index mode
fn handle_thumbnail_key(state: &mut AppState, hwnd: HWND, vk: VIRTUAL_KEY) {
    let file_count = state.filelist.len();
    if file_count == 0 {
        return;
    }

    let mut rect = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut rect);
    }
    let vw = (rect.right - rect.left) as f32;

    let thumb_view = state.thumbnail_view.as_mut().unwrap();
    let cols = thumb_view.cols_for(vw);

    match vk {
        VK_ESCAPE | VK_Q => unsafe {
            let _ = DestroyWindow(hwnd);
        },
        VK_RIGHT => {
            if thumb_view.selected + 1 < file_count {
                thumb_view.selected += 1;
            }
            window::invalidate(hwnd);
        }
        VK_LEFT => {
            if thumb_view.selected > 0 {
                thumb_view.selected -= 1;
            }
            window::invalidate(hwnd);
        }
        VK_DOWN => {
            if thumb_view.selected + cols < file_count {
                thumb_view.selected += cols;
            }
            window::invalidate(hwnd);
        }
        VK_UP => {
            if thumb_view.selected >= cols {
                thumb_view.selected -= cols;
            }
            window::invalidate(hwnd);
        }
        VK_RETURN => {
            // Switch to normal viewer mode for the selected image
            let selected = thumb_view.selected;
            state.thumbnail_view = None;
            state.filelist.set_current(selected);
            state.load_current_image();
            window::invalidate(hwnd);
        }
        VK_HOME => {
            thumb_view.selected = 0;
            thumb_view.scroll_y = 0.0;
            window::invalidate(hwnd);
        }
        VK_END => {
            thumb_view.selected = file_count.saturating_sub(1);
            window::invalidate(hwnd);
        }
        _ => {}
    }
}
