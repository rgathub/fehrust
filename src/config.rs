use clap::Parser;

/// A Windows-native image viewer inspired by feh
#[derive(Parser, Debug, Clone)]
#[command(name = "fehrust", version, about)]
pub struct Options {
    /// Image files or directories to display
    #[arg(default_value = ".")]
    pub files: Vec<String>,

    /// Start in fullscreen mode
    #[arg(short = 'F', long)]
    pub fullscreen: bool,

    /// Scale down images that are larger than the window
    #[arg(short = '.', long)]
    pub scale_down: bool,

    /// Set window geometry (WxH or WxH+X+Y)
    #[arg(short, long)]
    pub geometry: Option<String>,

    /// Set default zoom (percent, "max", or "fill")
    #[arg(long)]
    pub zoom: Option<String>,

    /// Window title format string
    #[arg(short = '^', long, default_value = "fehrust [%u of %l] - %f")]
    pub title: String,

    /// Draw filename on image
    #[arg(short, long)]
    pub draw_filename: bool,

    /// Recursive directory traversal
    #[arg(short, long)]
    pub recursive: bool,

    /// Slideshow delay in seconds
    #[arg(short = 'D', long)]
    pub slideshow_delay: Option<f64>,

    /// Sort images (name, filename, dirname, mtime, size, width, height, pixels, format, none)
    #[arg(short = 'S', long, default_value = "name")]
    pub sort: String,

    /// Reverse sort order
    #[arg(short = 'n', long)]
    pub reverse: bool,

    /// Randomize file list
    #[arg(short = 'z', long)]
    pub randomize: bool,

    /// Quiet mode
    #[arg(short, long)]
    pub quiet: bool,

    /// Verbose mode
    #[arg(long)]
    pub verbose: bool,

    /// Borderless window
    #[arg(short = 'x', long)]
    pub borderless: bool,

    /// Display info overlay
    #[arg(long)]
    pub draw_info: bool,

    /// Hide pointer
    #[arg(short = 'Y', long)]
    pub hide_pointer: bool,

    /// Wallpaper mode: center, fill, fit, stretch, tile, span
    #[arg(long, default_value = "center")]
    pub wallpaper_mode: String,

    /// Behavior on last slide: resume, quit, hold
    #[arg(long, default_value = "resume")]
    pub on_last_slide: String,

    /// Start at a specific file
    #[arg(long)]
    pub start_at: Option<String>,

    /// List mode: print file info to stdout and exit
    #[arg(short = 'L', long)]
    pub list: bool,

    /// Format string for --list output (default: "%f\t%wx%h\t%s")
    #[arg(long, default_value = "%f\t%wx%h\t%s")]
    pub list_format: String,

    /// Custom list mode: print file info using provided format string and exit
    #[arg(long)]
    pub customlist: Option<String>,

    /// Print only loadable image paths to stdout and exit
    #[arg(long)]
    pub loadable: bool,

    /// Print only unloadable image paths to stdout and exit
    #[arg(long)]
    pub unloadable: bool,

    /// Minimum image dimensions (WxH) filter
    #[arg(long)]
    pub min_dimension: Option<String>,

    /// Maximum image dimensions (WxH) filter
    #[arg(long)]
    pub max_dimension: Option<String>,

    /// Load file list from a text file (one path per line)
    #[arg(long)]
    pub filelist: Option<String>,

    /// Save the collected file list to a text file
    #[arg(long)]
    pub filelist_save: Option<String>,

    /// Auto-reload when files change on disk
    #[arg(long)]
    pub auto_reload: bool,

    /// Default action command (executed on Enter). Use %f=filepath, %n=filename, %u=index, %l=total
    #[arg(long = "action")]
    pub action: Option<String>,

    /// Action 1 (key 1)
    #[arg(long = "action1")]
    pub action1: Option<String>,
    /// Action 2 (key 2)
    #[arg(long = "action2")]
    pub action2: Option<String>,
    /// Action 3 (key 3)
    #[arg(long = "action3")]
    pub action3: Option<String>,
    /// Action 4 (key 4)
    #[arg(long = "action4")]
    pub action4: Option<String>,
    /// Action 5 (key 5)
    #[arg(long = "action5")]
    pub action5: Option<String>,
    /// Action 6 (key 6)
    #[arg(long = "action6")]
    pub action6: Option<String>,
    /// Action 7 (key 7)
    #[arg(long = "action7")]
    pub action7: Option<String>,
    /// Action 8 (key 8)
    #[arg(long = "action8")]
    pub action8: Option<String>,
    /// Action 9 (key 9)
    #[arg(long = "action9")]
    pub action9: Option<String>,

    /// Path to caption files directory. For image.jpg, reads {caption_path}/image.txt
    #[arg(long)]
    pub caption_path: Option<String>,

    /// Custom key binding in format "key action" (e.g. "q quit", "n next")
    #[arg(long = "key-binding", number_of_values = 1)]
    pub key_binding: Vec<String>,

    /// Start in thumbnail grid mode
    #[arg(short = 't', long)]
    pub thumbnails: bool,

    /// Open each file in its own window
    #[arg(long)]
    pub multiwindow: bool,

    /// Show contact sheet / index view
    #[arg(long)]
    pub index: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Normal,
    Pan,
    Zoom,
    Rotate,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OnLastSlide {
    Resume,
    Quit,
    Hold,
}

impl Options {
    pub fn on_last_slide_action(&self) -> OnLastSlide {
        match self.on_last_slide.as_str() {
            "quit" => OnLastSlide::Quit,
            "hold" => OnLastSlide::Hold,
            _ => OnLastSlide::Resume,
        }
    }

    pub fn wallpaper_mode(&self) -> crate::wallpaper::WallpaperMode {
        use crate::wallpaper::WallpaperMode;
        match self.wallpaper_mode.as_str() {
            "fill" => WallpaperMode::Fill,
            "fit" => WallpaperMode::Fit,
            "stretch" => WallpaperMode::Stretch,
            "tile" => WallpaperMode::Tile,
            "span" => WallpaperMode::Span,
            _ => WallpaperMode::Center,
        }
    }

    /// Return numbered actions as a Vec (index 0 = action1, etc.)
    pub fn numbered_actions(&self) -> Vec<Option<String>> {
        vec![
            self.action1.clone(),
            self.action2.clone(),
            self.action3.clone(),
            self.action4.clone(),
            self.action5.clone(),
            self.action6.clone(),
            self.action7.clone(),
            self.action8.clone(),
            self.action9.clone(),
        ]
    }

    pub fn parse_geometry(&self) -> Option<(u32, u32, Option<i32>, Option<i32>)> {
        let geom = self.geometry.as_ref()?;
        let (dims, offsets) = if let Some(plus_idx) = geom.find('+') {
            (&geom[..plus_idx], Some(&geom[plus_idx + 1..]))
        } else {
            (geom.as_str(), None)
        };

        let parts: Vec<&str> = dims.split('x').collect();
        if parts.len() != 2 {
            return None;
        }
        let w: u32 = parts[0].parse().ok()?;
        let h: u32 = parts[1].parse().ok()?;

        let (ox, oy) = if let Some(off) = offsets {
            let oparts: Vec<&str> = off.split('+').collect();
            if oparts.len() == 2 {
                (oparts[0].parse().ok(), oparts[1].parse().ok())
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        Some((w, h, ox, oy))
    }

    /// Parse a "WxH" dimension string into (width, height).
    pub fn parse_dimension(s: &Option<String>) -> Option<(u32, u32)> {
        let s = s.as_ref()?;
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() != 2 {
            return None;
        }
        let w: u32 = parts[0].parse().ok()?;
        let h: u32 = parts[1].parse().ok()?;
        Some((w, h))
    }
}

#[cfg(test)]
mod tests {
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
}
