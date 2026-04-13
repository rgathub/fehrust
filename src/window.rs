use windows::{
    Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Input::KeyboardAndMouse::*, Win32::UI::WindowsAndMessaging::*, core::*,
};

use std::cell::RefCell;

use crate::app::AppState;
use crate::filewatcher;
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

/// Set per-monitor DPI awareness. Tries V2, then V1, then legacy.
pub fn set_dpi_awareness() {
    use windows::Win32::UI::HiDpi::*;
    unsafe {
        // Try V2 first
        if SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).is_ok() {
            return;
        }
        // Fall back to V1
        if SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE).is_ok() {
            return;
        }
        // Legacy fallback
        let _ = SetProcessDPIAware();
    }
}

/// Get monitor info for the monitor containing the given window.
fn get_monitor_rect(hwnd: HWND) -> RECT {
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut mi = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetMonitorInfoW(monitor, &mut mi).as_bool() {
            mi.rcMonitor
        } else {
            // Fallback to primary screen
            RECT {
                left: 0,
                top: 0,
                right: GetSystemMetrics(SM_CXSCREEN),
                bottom: GetSystemMetrics(SM_CYSCREEN),
            }
        }
    }
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

        let class_name_wide: Vec<u16> = CLASS_NAME
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();

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
            // Go fullscreen on the monitor where the window currently lives
            let monitor_rect = get_monitor_rect(hwnd);
            SetWindowLongW(hwnd, GWL_STYLE, (WS_POPUP | WS_VISIBLE).0 as i32);
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOP),
                monitor_rect.left,
                monitor_rect.top,
                monitor_rect.right - monitor_rect.left,
                monitor_rect.bottom - monitor_rect.top,
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
                if wparam.0 & 0x0001 != 0 {
                    // MK_LBUTTON
                    input::handle_mouse_drag(state, hwnd, x, y);
                }
                LRESULT(0)
            }
            WM_LBUTTONUP => {
                input::handle_mouse_up(state);
                LRESULT(0)
            }
            WM_LBUTTONDBLCLK => {
                // Double-click in thumbnail mode opens the image
                if state.thumbnail_view.is_some() {
                    let x = (lparam.0 & 0xFFFF) as i16 as i32;
                    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
                    let mut rect = RECT::default();
                    unsafe {
                        let _ = GetClientRect(hwnd, &mut rect);
                    }
                    let vw = (rect.right - rect.left) as f32;
                    if let Some(ref thumb_view) = state.thumbnail_view
                        && let Some(idx) = thumb_view.handle_click(x as f32, y as f32, vw)
                        && idx < state.filelist.len()
                    {
                        state.thumbnail_view = None;
                        state.filelist.set_current(idx);
                        state.load_current_image();
                        invalidate(hwnd);
                    }
                }
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
            WM_DPICHANGED => {
                // wparam: LOWORD = new X dpi, HIWORD = new Y dpi
                let new_dpi = (wparam.0 & 0xFFFF) as u32;
                state.dpi_scale = new_dpi as f32 / 96.0;

                // lparam points to a suggested RECT for the window
                let suggested_rect = unsafe { &*(lparam.0 as *const RECT) };
                unsafe {
                    let _ = SetWindowPos(
                        hwnd,
                        None,
                        suggested_rect.left,
                        suggested_rect.top,
                        suggested_rect.right - suggested_rect.left,
                        suggested_rect.bottom - suggested_rect.top,
                        SWP_NOZORDER | SWP_NOACTIVATE,
                    );
                }
                invalidate(hwnd);
                LRESULT(0)
            }
            x if x == filewatcher::WM_FILE_CHANGED => {
                // File system change detected — reload file list and current image
                let recursive = state.options.recursive;
                let files = state.options.files.clone();
                let sort = state.options.sort.clone();
                let reverse = state.options.reverse;
                let current_path = state.filelist.current().map(|f| f.path.clone());

                let mut new_filelist = crate::filelist::FileList::collect(&files, recursive);
                new_filelist.sort_by(&sort, reverse);

                // Try to stay on the same file
                if let Some(ref path) = current_path {
                    new_filelist.jump_to(&path.to_string_lossy());
                }

                if !new_filelist.is_empty() {
                    state.filelist = new_filelist;
                    state.load_current_image();
                    invalidate(hwnd);
                }
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
