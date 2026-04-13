use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::thread;

use windows::core::PCWSTR;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Threading::WaitForSingleObject;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Custom message posted when directory contents change
pub const WM_FILE_CHANGED: u32 = WM_USER + 1;

/// Start a background thread that watches `dir` for file changes
/// and posts WM_FILE_CHANGED to `hwnd` when detected.
pub fn start_watcher(dir: PathBuf, hwnd: HWND) {
    // HWND is not Send, so pass the raw isize value
    let hwnd_raw = hwnd.0 as isize;
    thread::spawn(move || {
        let hwnd = HWND(hwnd_raw as *mut _);
        unsafe {
            watcher_loop(dir, hwnd);
        }
    });
}

unsafe fn watcher_loop(dir: PathBuf, hwnd: HWND) {
    let dir_wide: Vec<u16> = dir
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let handle = unsafe {
        FindFirstChangeNotificationW(
            PCWSTR(dir_wide.as_ptr()),
            false,
            FILE_NOTIFY_CHANGE_FILE_NAME | FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_SIZE,
        )
    };

    let handle = match handle {
        Ok(h) => h,
        Err(e) => {
            eprintln!("fehrust: failed to watch directory {:?}: {}", dir, e);
            return;
        }
    };

    loop {
        let result = unsafe { WaitForSingleObject(handle, 2000) };
        if result == WAIT_OBJECT_0 {
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_FILE_CHANGED, WPARAM(0), LPARAM(0));
            }

            // Re-arm the notification
            let ok = unsafe { FindNextChangeNotification(handle) };
            if ok.is_err() {
                break;
            }
        }
    }

    unsafe {
        let _ = FindCloseChangeNotification(handle);
    }
}
