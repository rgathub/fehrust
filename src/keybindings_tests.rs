use super::*;

#[test]
fn default_has_quit() {
    let map = default_bindings();
    assert_eq!(map[&VK_Q.0], Action::Quit);
}

#[test]
fn default_has_next() {
    let map = default_bindings();
    assert_eq!(map[&VK_SPACE.0], Action::Next);
}

#[test]
fn default_has_prev() {
    let map = default_bindings();
    assert_eq!(map[&VK_BACK.0], Action::Prev);
}

#[test]
fn default_has_zoom() {
    let map = default_bindings();
    assert_eq!(map[&VK_OEM_PLUS.0], Action::ZoomIn);
}

#[test]
fn default_has_fullscreen() {
    let map = default_bindings();
    assert_eq!(map[&VK_F11.0], Action::ToggleFullscreen);
}

#[test]
fn default_has_delete_from_disk() {
    let map = default_bindings();
    assert_eq!(map[&VK_E.0], Action::DeleteFromDisk);
}

#[test]
fn default_has_move() {
    let map = default_bindings();
    assert_eq!(map[&VK_M.0], Action::Move);
}

#[test]
fn default_count() {
    let map = default_bindings();
    assert!(map.len() >= 30);
}

#[test]
fn parse_key_q() {
    assert_eq!(parse_key_name("q"), Some(VK_Q.0));
}

#[test]
fn parse_key_escape() {
    assert_eq!(parse_key_name("escape"), Some(VK_ESCAPE.0));
}

#[test]
fn parse_key_space() {
    assert_eq!(parse_key_name("space"), Some(VK_SPACE.0));
}

#[test]
fn parse_key_f1() {
    assert_eq!(parse_key_name("f1"), Some(VK_F1.0));
}

#[test]
fn parse_key_invalid() {
    assert_eq!(parse_key_name("bogus"), None);
}

#[test]
fn parse_key_case_insensitive() {
    assert_eq!(parse_key_name("ESCAPE"), Some(VK_ESCAPE.0));
}

#[test]
fn parse_action_next() {
    assert_eq!(parse_action_name("next"), Some(Action::Next));
}

#[test]
fn parse_action_aliases() {
    assert_eq!(parse_action_name("zoom_in"), Some(Action::ZoomIn));
    assert_eq!(parse_action_name("zoomin"), Some(Action::ZoomIn));
    assert_eq!(parse_action_name("zoom-in"), Some(Action::ZoomIn));
}

#[test]
fn parse_action_invalid() {
    assert_eq!(parse_action_name("bogus"), None);
}

#[test]
fn build_keymap_empty() {
    let defaults = default_bindings();
    let built = build_keymap(&[]);
    assert_eq!(defaults.len(), built.len());
    for (k, v) in &defaults {
        assert_eq!(built.get(k), Some(v));
    }
}

#[test]
fn build_keymap_override() {
    let map = build_keymap(&["q next".to_string()]);
    assert_eq!(map[&VK_Q.0], Action::Next);
}

#[test]
fn build_keymap_invalid_ignored() {
    let defaults = default_bindings();
    let map = build_keymap(&["invalid".to_string()]);
    assert_eq!(defaults.len(), map.len());
    for (k, v) in &defaults {
        assert_eq!(map.get(k), Some(v));
    }
}
