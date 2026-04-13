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

    /// Behavior on last slide: resume, quit, hold
    #[arg(long, default_value = "resume")]
    pub on_last_slide: String,

    /// Start at a specific file
    #[arg(long)]
    pub start_at: Option<String>,
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
}
