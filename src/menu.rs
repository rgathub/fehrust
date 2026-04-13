use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

use crate::app::AppState;
use crate::window;

pub const IDM_NEXT: u16 = 1001;
pub const IDM_PREV: u16 = 1002;
pub const IDM_ZOOM_IN: u16 = 1003;
pub const IDM_ZOOM_OUT: u16 = 1004;
pub const IDM_FIT_WINDOW: u16 = 1005;
pub const IDM_ACTUAL_SIZE: u16 = 1006;
pub const IDM_ROTATE_CW: u16 = 1007;
pub const IDM_ROTATE_CCW: u16 = 1008;
pub const IDM_FLIP_H: u16 = 1009;
pub const IDM_FLIP_V: u16 = 1010;
pub const IDM_FULLSCREEN: u16 = 1011;
pub const IDM_TOGGLE_INFO: u16 = 1012;
pub const IDM_DELETE_LIST: u16 = 1013;
pub const IDM_QUIT: u16 = 1014;

pub fn show_context_menu(hwnd: HWND) {
    unsafe {
        let menu = match CreatePopupMenu() {
            Ok(m) => m,
            Err(_) => return,
        };

        let items: &[(u16, &str)] = &[
            (IDM_NEXT, "Next\tSpace"),
            (IDM_PREV, "Previous\tBackspace"),
            (0, ""),
            (IDM_ZOOM_IN, "Zoom In\t+"),
            (IDM_ZOOM_OUT, "Zoom Out\t-"),
            (IDM_FIT_WINDOW, "Fit to Window\t*"),
            (IDM_ACTUAL_SIZE, "Actual Size (100%)\t0"),
            (0, ""),
            (IDM_ROTATE_CW, "Rotate CW\t>"),
            (IDM_ROTATE_CCW, "Rotate CCW\t<"),
            (IDM_FLIP_H, "Flip Horizontal\t\\"),
            (IDM_FLIP_V, "Flip Vertical\t/"),
            (0, ""),
            (IDM_FULLSCREEN, "Toggle Fullscreen\tF11"),
            (IDM_TOGGLE_INFO, "Toggle Info\ti"),
            (0, ""),
            (IDM_DELETE_LIST, "Delete from List\tDel"),
            (IDM_QUIT, "Quit\tQ"),
        ];

        for &(id, text) in items {
            if id == 0 {
                let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
            } else {
                let text_wide: Vec<u16> =
                    text.encode_utf16().chain(std::iter::once(0)).collect();
                let _ = AppendMenuW(
                    menu,
                    MF_STRING,
                    id as usize,
                    PCWSTR(text_wide.as_ptr()),
                );
            }
        }

        let mut pt = POINT::default();
        let _ = GetCursorPos(&mut pt);

        let _ = TrackPopupMenu(menu, TPM_RIGHTBUTTON, pt.x, pt.y, Some(0), hwnd, None);
        let _ = DestroyMenu(menu);
    }
}

pub fn handle_menu_command(state: &mut AppState, hwnd: HWND, cmd: u16) {
    match cmd {
        IDM_NEXT => {
            state.navigate_next();
            window::invalidate(hwnd);
        }
        IDM_PREV => {
            state.navigate_prev();
            window::invalidate(hwnd);
        }
        IDM_ZOOM_IN => {
            state.zoom *= 1.15;
            window::invalidate(hwnd);
        }
        IDM_ZOOM_OUT => {
            state.zoom /= 1.15;
            window::invalidate(hwnd);
        }
        IDM_FIT_WINDOW => {
            state.zoom_to_fit();
            window::invalidate(hwnd);
        }
        IDM_ACTUAL_SIZE => {
            state.zoom = 1.0;
            state.pan_x = 0.0;
            state.pan_y = 0.0;
            window::invalidate(hwnd);
        }
        IDM_ROTATE_CW => {
            state.rotation = (state.rotation + 90.0) % 360.0;
            window::invalidate(hwnd);
        }
        IDM_ROTATE_CCW => {
            state.rotation = (state.rotation - 90.0 + 360.0) % 360.0;
            window::invalidate(hwnd);
        }
        IDM_FLIP_H => {
            state.flip_h = !state.flip_h;
            window::invalidate(hwnd);
        }
        IDM_FLIP_V => {
            state.flip_v = !state.flip_v;
            window::invalidate(hwnd);
        }
        IDM_FULLSCREEN => {
            window::toggle_fullscreen(hwnd, &mut state.is_fullscreen, &mut state.saved_rect);
            window::invalidate(hwnd);
        }
        IDM_TOGGLE_INFO => {
            state.options.draw_info = !state.options.draw_info;
            window::invalidate(hwnd);
        }
        IDM_DELETE_LIST => {
            state.remove_current_from_list(hwnd);
        }
        IDM_QUIT => unsafe {
            let _ = DestroyWindow(hwnd);
        },
        _ => {}
    }
}
