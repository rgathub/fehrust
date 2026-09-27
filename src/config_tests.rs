use super::*;
use clap::Parser;

fn default_opts() -> Options {
    Options::parse_from(["fehrust", "test.jpg"])
}

// --- on_last_slide_action ---

#[test]
fn on_last_slide_quit() {
    let opts = Options::parse_from(["fehrust", "--on-last-slide", "quit", "test.jpg"]);
    assert_eq!(opts.on_last_slide_action(), OnLastSlide::Quit);
}

#[test]
fn on_last_slide_hold() {
    let opts = Options::parse_from(["fehrust", "--on-last-slide", "hold", "test.jpg"]);
    assert_eq!(opts.on_last_slide_action(), OnLastSlide::Hold);
}

#[test]
fn on_last_slide_resume_default() {
    let opts = default_opts();
    assert_eq!(opts.on_last_slide_action(), OnLastSlide::Resume);
}

#[test]
fn on_last_slide_unknown_is_resume() {
    let opts = Options::parse_from(["fehrust", "--on-last-slide", "bogus", "test.jpg"]);
    assert_eq!(opts.on_last_slide_action(), OnLastSlide::Resume);
}

// --- wallpaper_mode ---

#[test]
fn wallpaper_mode_fill() {
    let opts = Options::parse_from(["fehrust", "--wallpaper-mode", "fill", "test.jpg"]);
    assert_eq!(opts.wallpaper_mode(), crate::wallpaper::WallpaperMode::Fill);
}

#[test]
fn wallpaper_mode_fit() {
    let opts = Options::parse_from(["fehrust", "--wallpaper-mode", "fit", "test.jpg"]);
    assert_eq!(opts.wallpaper_mode(), crate::wallpaper::WallpaperMode::Fit);
}

#[test]
fn wallpaper_mode_stretch() {
    let opts = Options::parse_from(["fehrust", "--wallpaper-mode", "stretch", "test.jpg"]);
    assert_eq!(
        opts.wallpaper_mode(),
        crate::wallpaper::WallpaperMode::Stretch
    );
}

#[test]
fn wallpaper_mode_tile() {
    let opts = Options::parse_from(["fehrust", "--wallpaper-mode", "tile", "test.jpg"]);
    assert_eq!(opts.wallpaper_mode(), crate::wallpaper::WallpaperMode::Tile);
}

#[test]
fn wallpaper_mode_span() {
    let opts = Options::parse_from(["fehrust", "--wallpaper-mode", "span", "test.jpg"]);
    assert_eq!(opts.wallpaper_mode(), crate::wallpaper::WallpaperMode::Span);
}

#[test]
fn wallpaper_mode_center_default() {
    let opts = default_opts();
    assert_eq!(
        opts.wallpaper_mode(),
        crate::wallpaper::WallpaperMode::Center
    );
}

#[test]
fn wallpaper_mode_unknown_is_center() {
    let opts = Options::parse_from(["fehrust", "--wallpaper-mode", "unknown", "test.jpg"]);
    assert_eq!(
        opts.wallpaper_mode(),
        crate::wallpaper::WallpaperMode::Center
    );
}

// --- parse_geometry ---

#[test]
fn parse_geometry_wxh() {
    let opts = Options::parse_from(["fehrust", "--geometry", "800x600", "test.jpg"]);
    assert_eq!(opts.parse_geometry(), Some((800, 600, None, None)));
}

#[test]
fn parse_geometry_wxh_plus_offsets() {
    let opts = Options::parse_from(["fehrust", "--geometry", "800x600+10+20", "test.jpg"]);
    assert_eq!(opts.parse_geometry(), Some((800, 600, Some(10), Some(20))));
}

#[test]
fn parse_geometry_invalid() {
    let opts = Options::parse_from(["fehrust", "--geometry", "notvalid", "test.jpg"]);
    assert_eq!(opts.parse_geometry(), None);
}

#[test]
fn parse_geometry_none() {
    let opts = default_opts();
    assert_eq!(opts.parse_geometry(), None);
}

// --- parse_dimension ---

#[test]
fn parse_dimension_valid() {
    let s = Some("1920x1080".to_string());
    assert_eq!(Options::parse_dimension(&s), Some((1920, 1080)));
}

#[test]
fn parse_dimension_invalid() {
    let s = Some("bad".to_string());
    assert_eq!(Options::parse_dimension(&s), None);
}

#[test]
fn parse_dimension_none() {
    assert_eq!(Options::parse_dimension(&None), None);
}

#[test]
fn move_destination() {
    let opts = Options::parse_from(["fehrust", "--move", "C:\\Moved", "test.jpg"]);
    assert_eq!(opts.move_to, Some("C:\\Moved".to_string()));
}

// --- numbered_actions ---

#[test]
fn numbered_actions_default_all_none() {
    let opts = default_opts();
    let actions = opts.numbered_actions();
    assert_eq!(actions.len(), 9);
    assert!(actions.iter().all(|a| a.is_none()));
}

#[test]
fn numbered_actions_with_some_set() {
    let opts = Options::parse_from([
        "fehrust",
        "--action1",
        "echo %f",
        "--action3",
        "open %f",
        "test.jpg",
    ]);
    let actions = opts.numbered_actions();
    assert_eq!(actions[0], Some("echo %f".to_string()));
    assert_eq!(actions[1], None);
    assert_eq!(actions[2], Some("open %f".to_string()));
    assert_eq!(actions.len(), 9);
}
