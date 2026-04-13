use std::collections::HashMap;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Next,
    Prev,
    ZoomIn,
    ZoomOut,
    FitWindow,
    ActualSize,
    RotateCW,
    RotateCCW,
    FlipH,
    FlipV,
    ToggleFullscreen,
    ToggleInfo,
    ToggleFilename,
    Pause,
    Quit,
    Delete,
    Wallpaper,
    Save,
    ScrollUp,
    ScrollDown,
    JumpFirst,
    JumpLast,
    JumpForward,
    JumpBack,
}

/// Map from virtual key code (u16) to Action
pub type KeyMap = HashMap<u16, Action>;

/// Build the default key bindings (matching the original hardcoded input.rs)
pub fn default_bindings() -> KeyMap {
    let mut m = KeyMap::new();

    // Quit
    m.insert(VK_Q.0, Action::Quit);
    m.insert(VK_ESCAPE.0, Action::Quit);

    // Navigation
    m.insert(VK_SPACE.0, Action::Next);
    m.insert(VK_RIGHT.0, Action::Next);
    m.insert(VK_BACK.0, Action::Prev);
    m.insert(VK_LEFT.0, Action::Prev);
    m.insert(VK_HOME.0, Action::JumpFirst);
    m.insert(VK_END.0, Action::JumpLast);
    m.insert(VK_PRIOR.0, Action::JumpForward); // PgUp
    m.insert(VK_NEXT.0, Action::JumpBack); // PgDn

    // Zoom
    m.insert(VK_OEM_PLUS.0, Action::ZoomIn);
    m.insert(VK_ADD.0, Action::ZoomIn);
    m.insert(VK_OEM_MINUS.0, Action::ZoomOut);
    m.insert(VK_SUBTRACT.0, Action::ZoomOut);
    m.insert(VK_MULTIPLY.0, Action::FitWindow);
    m.insert(VK_0.0, Action::ActualSize);

    // Scroll
    m.insert(VK_UP.0, Action::ScrollUp);
    m.insert(VK_DOWN.0, Action::ScrollDown);

    // Fullscreen
    m.insert(VK_X.0, Action::ToggleFullscreen);
    m.insert(VK_F11.0, Action::ToggleFullscreen);

    // Overlays
    m.insert(VK_D.0, Action::ToggleFilename);
    m.insert(VK_I.0, Action::ToggleInfo);

    // Rotation / flip
    m.insert(VK_OEM_PERIOD.0, Action::RotateCW);
    m.insert(VK_OEM_COMMA.0, Action::RotateCCW);
    m.insert(VK_OEM_2.0, Action::FlipV);
    m.insert(VK_OEM_5.0, Action::FlipH);

    // Pause
    m.insert(VK_P.0, Action::Pause);

    // Delete
    m.insert(VK_DELETE.0, Action::Delete);

    // Wallpaper / Save
    m.insert(VK_W.0, Action::Wallpaper);
    m.insert(VK_S.0, Action::Save);

    m
}

/// Parse a key name string to a virtual key code (u16)
fn parse_key_name(name: &str) -> Option<u16> {
    match name.to_lowercase().as_str() {
        "q" => Some(VK_Q.0),
        "w" => Some(VK_W.0),
        "e" => Some(VK_E.0),
        "r" => Some(VK_R.0),
        "t" => Some(VK_T.0),
        "y" => Some(VK_Y.0),
        "u" => Some(VK_U.0),
        "i" => Some(VK_I.0),
        "o" => Some(VK_O.0),
        "p" => Some(VK_P.0),
        "a" => Some(VK_A.0),
        "s" => Some(VK_S.0),
        "d" => Some(VK_D.0),
        "f" => Some(VK_F.0),
        "g" => Some(VK_G.0),
        "h" => Some(VK_H.0),
        "j" => Some(VK_J.0),
        "k" => Some(VK_K.0),
        "l" => Some(VK_L.0),
        "z" => Some(VK_Z.0),
        "x" => Some(VK_X.0),
        "c" => Some(VK_C.0),
        "v" => Some(VK_V.0),
        "b" => Some(VK_B.0),
        "n" => Some(VK_N.0),
        "m" => Some(VK_M.0),
        "0" => Some(VK_0.0),
        "1" => Some(VK_1.0),
        "2" => Some(VK_2.0),
        "3" => Some(VK_3.0),
        "4" => Some(VK_4.0),
        "5" => Some(VK_5.0),
        "6" => Some(VK_6.0),
        "7" => Some(VK_7.0),
        "8" => Some(VK_8.0),
        "9" => Some(VK_9.0),
        "space" => Some(VK_SPACE.0),
        "enter" | "return" => Some(VK_RETURN.0),
        "escape" | "esc" => Some(VK_ESCAPE.0),
        "backspace" | "back" => Some(VK_BACK.0),
        "delete" | "del" => Some(VK_DELETE.0),
        "up" => Some(VK_UP.0),
        "down" => Some(VK_DOWN.0),
        "left" => Some(VK_LEFT.0),
        "right" => Some(VK_RIGHT.0),
        "home" => Some(VK_HOME.0),
        "end" => Some(VK_END.0),
        "f1" => Some(VK_F1.0),
        "f2" => Some(VK_F2.0),
        "f3" => Some(VK_F3.0),
        "f4" => Some(VK_F4.0),
        "f5" => Some(VK_F5.0),
        "f6" => Some(VK_F6.0),
        "f7" => Some(VK_F7.0),
        "f8" => Some(VK_F8.0),
        "f9" => Some(VK_F9.0),
        "f10" => Some(VK_F10.0),
        "f11" => Some(VK_F11.0),
        "f12" => Some(VK_F12.0),
        "plus" | "+" => Some(VK_OEM_PLUS.0),
        "minus" | "-" => Some(VK_OEM_MINUS.0),
        "period" | "." | ">" => Some(VK_OEM_PERIOD.0),
        "comma" | "," | "<" => Some(VK_OEM_COMMA.0),
        "/" => Some(VK_OEM_2.0),
        "\\" => Some(VK_OEM_5.0),
        "*" => Some(VK_MULTIPLY.0),
        _ => None,
    }
}

/// Parse an action name string to an Action
fn parse_action_name(name: &str) -> Option<Action> {
    match name.to_lowercase().as_str() {
        "next" => Some(Action::Next),
        "prev" | "previous" => Some(Action::Prev),
        "zoom_in" | "zoomin" | "zoom-in" => Some(Action::ZoomIn),
        "zoom_out" | "zoomout" | "zoom-out" => Some(Action::ZoomOut),
        "fit_window" | "fitwindow" | "fit-window" | "fit" => Some(Action::FitWindow),
        "actual_size" | "actualsize" | "actual-size" => Some(Action::ActualSize),
        "rotate_cw" | "rotatecw" | "rotate-cw" => Some(Action::RotateCW),
        "rotate_ccw" | "rotateccw" | "rotate-ccw" => Some(Action::RotateCCW),
        "flip_h" | "fliph" | "flip-h" => Some(Action::FlipH),
        "flip_v" | "flipv" | "flip-v" => Some(Action::FlipV),
        "toggle_fullscreen" | "fullscreen" => Some(Action::ToggleFullscreen),
        "toggle_info" | "info" => Some(Action::ToggleInfo),
        "toggle_filename" | "filename" => Some(Action::ToggleFilename),
        "pause" => Some(Action::Pause),
        "quit" | "exit" => Some(Action::Quit),
        "delete" => Some(Action::Delete),
        "wallpaper" => Some(Action::Wallpaper),
        "save" => Some(Action::Save),
        "scroll_up" | "scrollup" | "scroll-up" => Some(Action::ScrollUp),
        "scroll_down" | "scrolldown" | "scroll-down" => Some(Action::ScrollDown),
        "jump_first" | "jumpfirst" | "jump-first" | "first" => Some(Action::JumpFirst),
        "jump_last" | "jumplast" | "jump-last" | "last" => Some(Action::JumpLast),
        "jump_forward" | "jumpforward" | "jump-forward" => Some(Action::JumpForward),
        "jump_back" | "jumpback" | "jump-back" => Some(Action::JumpBack),
        _ => None,
    }
}

/// Parse user-supplied key binding strings and merge into the default map.
/// Each string should be "key action", e.g. "q quit", "n next".
pub fn build_keymap(custom_bindings: &[String]) -> KeyMap {
    let mut map = default_bindings();

    for binding in custom_bindings {
        let parts: Vec<&str> = binding.split_whitespace().collect();
        if parts.len() != 2 {
            eprintln!(
                "fehrust: invalid key binding '{}' (expected 'key action')",
                binding
            );
            continue;
        }
        let key = match parse_key_name(parts[0]) {
            Some(k) => k,
            None => {
                eprintln!("fehrust: unknown key name '{}'", parts[0]);
                continue;
            }
        };
        let action = match parse_action_name(parts[1]) {
            Some(a) => a,
            None => {
                eprintln!("fehrust: unknown action '{}'", parts[1]);
                continue;
            }
        };
        map.insert(key, action);
    }

    map
}
