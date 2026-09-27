use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Threading::WaitForSingleObject;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::PCWSTR;

/// Custom message posted when directory contents change.
pub const WM_FILE_CHANGED: u32 = WM_USER + 1;

pub struct WatcherHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl WatcherHandle {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Start a background thread that watches `dir` for file changes.
pub fn start_watcher(dir: PathBuf, hwnd: HWND) -> WatcherHandle {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let hwnd_raw = hwnd.0 as isize;
    let thread = thread::spawn(move || {
        let hwnd = HWND(hwnd_raw as *mut _);
        unsafe {
            watcher_loop(dir, hwnd, thread_stop);
        }
    });
    WatcherHandle {
        stop,
        thread: Some(thread),
    }
}

unsafe fn watcher_loop(dir: PathBuf, hwnd: HWND, stop: Arc<AtomicBool>) {
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

    let mut last_notification = Instant::now() - Duration::from_secs(1);
    loop {
        if stop.load(Ordering::Acquire) {
            break;
        }
        let result = unsafe { WaitForSingleObject(handle, 250) };
        if result == WAIT_OBJECT_0 {
            let now = Instant::now();
            if now.duration_since(last_notification) >= Duration::from_millis(200) {
                unsafe {
                    let _ = PostMessageW(Some(hwnd), WM_FILE_CHANGED, WPARAM(0), LPARAM(0));
                }
                last_notification = now;
            }

            if unsafe { FindNextChangeNotification(handle) }.is_err() {
                break;
            }
        } else if result != WAIT_TIMEOUT {
            break;
        }
    }

    unsafe {
        let _ = FindCloseChangeNotification(handle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watcher_can_be_stopped_without_leaking() {
        let dir = tempfile::tempdir().unwrap();
        let watcher = start_watcher(dir.path().to_path_buf(), HWND::default());
        watcher.stop();
    }
}
