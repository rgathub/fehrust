use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Input::KeyboardAndMouse::*,
    Win32::UI::WindowsAndMessaging::*,
};

use std::cell::RefCell;

use crate::app::AppState;
use crate::input;
use crate::menu;

/// Window class name
const CLASS_NAME: &str = "FehRustWindow";

thread_local! {
    /// Store a pointer to AppState for the WndProc callback
    static APP_STATE: RefCell<Option<*mut AppState>> = const { RefCell::new(None) };
}

pub fn set_app_state(state: *mut AppState) {
    APP_STATE.with(|s| *s.borrow_mut() = Some(state));
}

pub fn create_window(
    title: &str,
    width: u32,
    height: u32,
    borderless: bool,
    fullscreen: bool,
) -> Result<HWND> {
    unsafe {
        let hinstance = GetModuleHandleW(None)?;

        let class_name_wide: Vec<u16> = CLASS_NAME.encode_utf16().chain(std::iter::once(0)).collect();

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_DBLCLKS,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance.into(),
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH::default(),
            lpszClassName: PCWSTR(class_name_wide.as_ptr()),
            ..Default::default()
        };

        let atom = RegisterClassExW(&wc);
        if atom == 0 {
            return Err(Error::from_win32());
        }

        let style = if borderless || fullscreen {
            WS_POPUP | WS_VISIBLE
        } else {
            WS_OVERLAPPEDWINDOW | WS_VISIBLE
        };

        let (x, y, w, h) = if fullscreen {
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            (0, 0, screen_w, screen_h)
        } else {
            // Adjust window rect so client area is the requested size
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            AdjustWindowRectEx(&mut rect, style, false, WINDOW_EX_STYLE::default())?;
            (
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                rect.right - rect.left,
                rect.bottom - rect.top,
            )
        };

        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR(class_name_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            style,
            x,
            y,
            w,
            h,
            None,
            None,
            Some(hinstance.into()),
            None,
        )?;

        Ok(hwnd)
    }
}

pub fn run_message_loop() -> i32 {
    unsafe {
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        msg.wParam.0 as i32
    }
}

pub fn update_title(hwnd: HWND, title: &str) {
    let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let _ = SetWindowTextW(hwnd, PCWSTR(title_wide.as_ptr()));
    }
}

pub fn invalidate(hwnd: HWND) {
    unsafe {
        let _ = InvalidateRect(Some(hwnd), None, false);
    }
}

pub fn set_cursor_visible(visible: bool) {
    unsafe {
        ShowCursor(visible);
    }
}

pub fn toggle_fullscreen(hwnd: HWND, is_fullscreen: &mut bool, saved_rect: &mut RECT) {
    unsafe {
        if *is_fullscreen {
            // Restore
            SetWindowLongW(hwnd, GWL_STYLE, (WS_OVERLAPPEDWINDOW | WS_VISIBLE).0 as i32);
            let _ = SetWindowPos(
                hwnd,
                None,
                saved_rect.left,
                saved_rect.top,
                saved_rect.right - saved_rect.left,
                saved_rect.bottom - saved_rect.top,
                SWP_FRAMECHANGED | SWP_NOZORDER,
            );
            *is_fullscreen = false;
        } else {
            // Save current rect
            let _ = GetWindowRect(hwnd, saved_rect);
            // Go fullscreen
            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            SetWindowLongW(hwnd, GWL_STYLE, (WS_POPUP | WS_VISIBLE).0 as i32);
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOP),
                0,
                0,
                screen_w,
                screen_h,
                SWP_FRAMECHANGED,
            );
            *is_fullscreen = true;
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    APP_STATE.with(|state_cell| {
        let state_opt = state_cell.borrow();
        let state = match *state_opt {
            Some(ptr) => unsafe { &mut *ptr },
            None => return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        };

        match msg {
            WM_PAINT => {
                let _ = state.paint();
                let mut ps = PAINTSTRUCT::default();
                unsafe {
                    let _ = BeginPaint(hwnd, &mut ps);
                    let _ = EndPaint(hwnd, &ps);
                }
                LRESULT(0)
            }
            WM_SIZE => {
                let width = (lparam.0 & 0xFFFF) as u32;
                let height = ((lparam.0 >> 16) & 0xFFFF) as u32;
                if width > 0 && height > 0 {
                    let _ = state.handle_resize(width, height);
                }
                LRESULT(0)
            }
            WM_KEYDOWN => {
                let vk = VIRTUAL_KEY(wparam.0 as u16);
                input::handle_key(state, hwnd, vk);
                LRESULT(0)
            }
            WM_MOUSEWHEEL => {
                let delta = ((wparam.0 >> 16) & 0xFFFF) as i16;
                let keys = (wparam.0 & 0xFFFF) as u16;
                let ctrl = keys & 0x0008 != 0; // MK_CONTROL
                input::handle_mouse_wheel(state, hwnd, delta, ctrl);
                LRESULT(0)
            }
            WM_LBUTTONDOWN => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                input::handle_mouse_down(state, hwnd, x, y);
                LRESULT(0)
            }
            WM_MOUSEMOVE => {
                let x = (lparam.0 & 0xFFFF) as i16 as i32;
                let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                if wparam.0 & 0x0001 != 0 { // MK_LBUTTON
                    input::handle_mouse_drag(state, hwnd, x, y);
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                input::handle_mouse_up(state);
                LRESULT(0)
            }
            WM_RBUTTONUP => {
                menu::show_context_menu(hwnd);
                LRESULT(0)
            }
            WM_SETCURSOR => {
                if state.options.hide_pointer {
                    let hit_test = (lparam.0 & 0xFFFF) as u16;
                    if hit_test == 1 {
                        // HTCLIENT
                        unsafe {
                            SetCursor(None);
                        }
                        return LRESULT(1);
                    }
                }
                unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
            }
            WM_COMMAND => {
                let cmd = (wparam.0 & 0xFFFF) as u16;
                menu::handle_menu_command(state, hwnd, cmd);
                LRESULT(0)
            }
            WM_TIMER => {
                state.handle_timer();
                invalidate(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                LRESULT(0)
            }
            _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
        }
    })
}
