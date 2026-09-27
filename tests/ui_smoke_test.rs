#![cfg(windows)]
#![allow(unexpected_cfgs)]

use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GetWindowRect, HWND_TOP, IsWindowVisible, SWP_NOMOVE, SWP_NOZORDER, SendMessageW,
    SetWindowPos, WM_CLOSE,
};
use windows::core::PCWSTR;

const WINDOW_CLASS: &str = "FehRustWindow";

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn find_main_window() -> Option<HWND> {
    let class_name = wide(WINDOW_CLASS);
    unsafe { FindWindowW(PCWSTR(class_name.as_ptr()), None) }.ok()
}

fn stop_process(child: &mut Child, hwnd: Option<HWND>) {
    if let Some(hwnd) = hwnd {
        unsafe {
            let _ = SendMessageW(hwnd, WM_CLOSE, Some(WPARAM(0)), Some(LPARAM(0)));
        }
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if child.try_wait().unwrap().is_some() {
            return;
        }
        thread::sleep(Duration::from_millis(50));
    }

    child.kill().unwrap();
    let _ = child.wait();
}

#[test]
#[cfg_attr(coverage, ignore)]
fn opens_window_resizes_and_exits_cleanly() {
    let fixture = std::fs::canonicalize("tests/fixtures/test_1x1.png").unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_fehrust"))
        .arg("--geometry")
        .arg("320x240")
        .arg(fixture)
        .spawn()
        .unwrap();

    let deadline = Instant::now() + Duration::from_secs(10);
    let hwnd = loop {
        if let Some(hwnd) = find_main_window() {
            break hwnd;
        }
        if child.try_wait().unwrap().is_some() {
            panic!("fehrust exited before creating its window");
        }
        if Instant::now() >= deadline {
            stop_process(&mut child, None);
            panic!("timed out waiting for the fehrust window");
        }
        thread::sleep(Duration::from_millis(50));
    };

    assert!(unsafe { IsWindowVisible(hwnd).as_bool() });

    let mut before = RECT::default();
    unsafe {
        GetWindowRect(hwnd, &mut before).unwrap();
        SetWindowPos(
            hwnd,
            Some(HWND_TOP),
            0,
            0,
            420,
            300,
            SWP_NOMOVE | SWP_NOZORDER,
        )
        .unwrap();
    }

    let mut after = RECT::default();
    unsafe {
        GetWindowRect(hwnd, &mut after).unwrap();
    }
    assert!(after.right - after.left >= 400);
    assert!(after.bottom - after.top >= 280);
    assert!(before.right > before.left);
    assert!(before.bottom > before.top);

    stop_process(&mut child, Some(hwnd));
    assert_eq!(child.try_wait().unwrap().unwrap().code(), Some(0));
}
