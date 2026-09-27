use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::config::{Options, ViewMode};
use crate::exif;
use crate::filelist::FileList;
use crate::format::expand_format;
use crate::image_loader::{DecodedImage, ImageLoader, LoadedImage};
use crate::keybindings::{self, KeyMap};
use crate::renderer::Renderer;
use crate::thumbnail::ThumbnailView;
use crate::transforms;
use crate::window;

use std::collections::VecDeque;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread;

const SLIDESHOW_TIMER_ID: usize = 1;
const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;
pub const WM_REMOTE_IMAGE: u32 = WM_USER + 2;
pub const WM_IMAGE_LOADED: u32 = WM_USER + 3;
pub const WM_FILE_DISCOVERED: u32 = WM_USER + 4;

type ImageLoadMessage = (u64, PathBuf, Result<DecodedImage, String>);

mod file_operations;
mod loading;
mod modes;
mod navigation;
mod rendering;
pub use modes::run;

fn slideshow_timer_ms(delay: Option<f64>) -> Option<u32> {
    let delay = delay?;
    let ms = (delay * 1000.0) as u32;
    (ms > 0).then_some(ms)
}

fn move_file(path: &Path, target: &Path) -> io::Result<()> {
    match std::fs::rename(path, target) {
        Ok(()) => Ok(()),
        Err(error) if error.raw_os_error() == Some(17) => {
            if target.exists() {
                return Err(error);
            }

            std::fs::copy(path, target)?;
            if let Err(remove_error) = std::fs::remove_file(path) {
                let _ = std::fs::remove_file(target);
                return Err(remove_error);
            }
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub struct AppState {
    pub options: Options,
    pub filelist: FileList,
    pub image_loader: ImageLoader,
    pub renderer: Renderer,
    pub current_image: Option<LoadedImage>,
    pub current_exif: Option<exif::ExifInfo>,
    pub hwnd: HWND,

    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
    pub rotation: f64,
    pub flip_h: bool,
    pub flip_v: bool,
    pub mode: ViewMode,
    pub paused: bool,
    pub is_fullscreen: bool,
    pub saved_rect: RECT,

    // Drag state
    pub drag_start: Option<(i32, i32)>,
    pub drag_pan_start: (f64, f64),

    // DPI
    pub dpi_scale: f32,

    // Caption
    pub current_caption: Option<String>,

    // Keybindings
    pub keybindings: KeyMap,

    // Numbered actions (action1..action9)
    pub numbered_actions: Vec<Option<String>>,

    // Thumbnail mode
    pub thumbnail_view: Option<ThumbnailView>,
    image_load_tx: mpsc::Sender<ImageLoadMessage>,
    image_load_rx: Receiver<ImageLoadMessage>,
    load_cancel: Arc<AtomicU64>,
    remote_image_tx: mpsc::Sender<Result<crate::filelist::FehFile, String>>,
    remote_image_rx: Receiver<Result<crate::filelist::FehFile, String>>,
    discovery_tx: mpsc::Sender<crate::filelist::FehFile>,
    discovery_rx: Receiver<crate::filelist::FehFile>,
    next_load_id: u64,
    pub(crate) watcher: Option<crate::filewatcher::WatcherHandle>,
    pub(crate) background_cancel: Arc<AtomicBool>,
    window_width: u32,
    window_height: u32,
}

impl AppState {
    pub fn new(options: Options) -> windows::core::Result<Self> {
        let renderer = Renderer::new()?;
        let image_loader = ImageLoader::new()?;

        let has_remote_urls = options
            .files
            .iter()
            .any(|path| crate::filelist::is_remote_url(path));
        let mut filelist = if let Some(ref fl_path) = options.filelist {
            FileList::from_filelist(Path::new(fl_path))
        } else if has_remote_urls || options.recursive {
            FileList::collect_local(&options.files, false)
        } else {
            FileList::collect(&options.files, options.recursive)
        };

        let has_recursive_directories =
            options.recursive && options.files.iter().any(|path| Path::new(path).is_dir());
        if filelist.is_empty() && !has_remote_urls && !has_recursive_directories {
            return Err(windows::core::Error::new(E_FAIL, "No image files found"));
        }

        // Sort
        if options.randomize {
            filelist.randomize();
        } else {
            filelist.sort_by(&options.sort, options.reverse);
        }

        // Dimension filtering
        if options.min_dimension.is_some() || options.max_dimension.is_some() {
            let min_dim = Options::parse_dimension(&options.min_dimension);
            let max_dim = Options::parse_dimension(&options.max_dimension);
            filelist.filter_dimensions(&image_loader, min_dim, max_dim);
            if filelist.is_empty() {
                return Err(windows::core::Error::new(
                    E_FAIL,
                    "No images match dimension filter",
                ));
            }
        }

        // Save filelist if requested
        if let Some(ref save_path) = options.filelist_save
            && let Err(e) = filelist.save_filelist(Path::new(save_path))
        {
            eprintln!("Failed to save filelist: {e}");
        }

        // Jump to start-at file
        if let Some(ref start) = options.start_at {
            filelist.jump_to(start);
        }

        let keybindings = keybindings::build_keymap(&options.key_binding);
        let numbered_actions = options.numbered_actions();
        let (image_load_tx, image_load_rx) = mpsc::channel();
        let load_cancel = Arc::new(AtomicU64::new(0));
        let (remote_image_tx, remote_image_rx) = mpsc::channel();
        let (discovery_tx, discovery_rx) = mpsc::channel();
        let background_cancel = Arc::new(AtomicBool::new(false));

        Ok(Self {
            options,
            filelist,
            image_loader,
            renderer,
            current_image: None,
            current_exif: None,
            hwnd: HWND::default(),
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            rotation: 0.0,
            flip_h: false,
            flip_v: false,
            mode: ViewMode::Normal,
            paused: false,
            is_fullscreen: false,
            saved_rect: RECT::default(),
            drag_start: None,
            drag_pan_start: (0.0, 0.0),
            dpi_scale: 1.0,
            current_caption: None,
            keybindings,
            numbered_actions,
            thumbnail_view: None,
            image_load_tx,
            image_load_rx,
            load_cancel,
            remote_image_tx,
            remote_image_rx,
            discovery_tx,
            discovery_rx,
            next_load_id: 0,
            watcher: None,
            background_cancel,
            window_width: DEFAULT_WIDTH,
            window_height: DEFAULT_HEIGHT,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::slideshow_timer_ms;

    #[test]
    fn slideshow_timer_ms_requires_a_delay() {
        assert_eq!(slideshow_timer_ms(None), None);
    }

    #[test]
    fn slideshow_timer_ms_converts_seconds_to_milliseconds() {
        assert_eq!(slideshow_timer_ms(Some(2.5)), Some(2500));
    }

    #[test]
    fn slideshow_timer_ms_ignores_non_positive_delays() {
        assert_eq!(slideshow_timer_ms(Some(0.0)), None);
        assert_eq!(slideshow_timer_ms(Some(-1.0)), None);
    }
}

/// Build a FileList from options (--filelist or CLI args)
fn build_filelist(options: &Options) -> FileList {
    let mut filelist = if let Some(ref fl_path) = options.filelist {
        FileList::from_filelist(Path::new(fl_path))
    } else {
        FileList::collect(&options.files, options.recursive)
    };

    if options.randomize {
        filelist.randomize();
    } else {
        filelist.sort_by(&options.sort, options.reverse);
    }

    filelist
}

/// List mode: print file info and exit
fn run_list_mode(options: &Options) -> windows::core::Result<()> {
    let image_loader = ImageLoader::new()?;
    let mut filelist = build_filelist(options);

    let fmt = options
        .customlist
        .as_deref()
        .unwrap_or(&options.list_format);
    let total = filelist.len();

    // Populate file sizes
    for file in filelist.files_mut().iter_mut() {
        file.load_stat();
    }

    for (i, file) in filelist.files().iter().enumerate() {
        let (w, h) = match image_loader.get_dimensions(&file.path) {
            Ok((w, h)) => (Some(w), Some(h)),
            Err(_) => (None, None),
        };
        let line = expand_format(fmt, Some(file), i, total, 1.0, w, h, false);
        println!("{}", line);
    }

    Ok(())
}

/// Filter mode: print loadable or unloadable file paths and exit
fn run_filter_mode(options: &Options) -> windows::core::Result<()> {
    let image_loader = ImageLoader::new()?;
    let filelist = build_filelist(options);

    for file in filelist.files() {
        let loads = image_loader.load(&file.path).is_ok();
        if (options.loadable && loads) || (options.unloadable && !loads) {
            println!("{}", file.path.display());
        }
    }

    Ok(())
}

/// Multi-window mode: open a separate window for each file
pub fn run_multiwindow(options: Options) -> windows::core::Result<()> {
    window::set_dpi_awareness();

    let (init_w, init_h) = options
        .parse_geometry()
        .map(|(w, h, _, _)| (w, h))
        .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT));

    let filelist = build_filelist(&options);
    if filelist.is_empty() {
        return Err(windows::core::Error::new(E_FAIL, "No image files found"));
    }

    // Create one AppState per window, each with a single-file filelist.
    let mut states: Vec<AppState> = Vec::new();

    for file in filelist.files() {
        let single_list = FileList::from_single(file.clone());
        let mut single_opts = options.clone();
        single_opts.multiwindow = false;

        let renderer = Renderer::new()?;
        let image_loader = ImageLoader::new()?;
        let keybindings = keybindings::build_keymap(&single_opts.key_binding);
        let numbered_actions = single_opts.numbered_actions();
        let (image_load_tx, image_load_rx) = mpsc::channel();
        let load_cancel = Arc::new(AtomicU64::new(0));
        let (remote_image_tx, remote_image_rx) = mpsc::channel();
        let (discovery_tx, discovery_rx) = mpsc::channel();
        let background_cancel = Arc::new(AtomicBool::new(false));

        let title = file.name.clone();
        let hwnd = window::create_window(
            &title,
            init_w,
            init_h,
            single_opts.borderless,
            single_opts.fullscreen,
        )?;

        let mut state = AppState {
            options: single_opts,
            filelist: single_list,
            renderer,
            image_loader,
            current_image: None,
            current_exif: None,
            hwnd,
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            rotation: 0.0,
            flip_h: false,
            flip_v: false,
            mode: ViewMode::Normal,
            paused: false,
            is_fullscreen: false,
            saved_rect: RECT::default(),
            drag_start: None,
            drag_pan_start: (0.0, 0.0),
            dpi_scale: 1.0,
            current_caption: None,
            keybindings,
            numbered_actions,
            thumbnail_view: None,
            image_load_tx,
            image_load_rx,
            load_cancel,
            remote_image_tx,
            remote_image_rx,
            discovery_tx,
            discovery_rx,
            next_load_id: 0,
            watcher: None,
            background_cancel,
            window_width: init_w,
            window_height: init_h,
        };

        let mut rect = RECT::default();
        unsafe {
            let _ = GetClientRect(hwnd, &mut rect);
        }
        let cw = (rect.right - rect.left) as u32;
        let ch = (rect.bottom - rect.top) as u32;
        state.window_width = if cw > 0 { cw } else { init_w };
        state.window_height = if ch > 0 { ch } else { init_h };

        state
            .renderer
            .create_render_target(hwnd, state.window_width, state.window_height)?;
        state.load_current_image();
        if state.options.scale_down {
            state.zoom_to_fit();
        }

        states.push(state);
    }

    for state in &mut states {
        window::set_app_state(state.hwnd, state as *mut AppState);
    }

    for state in &states {
        window::invalidate(state.hwnd);
    }

    window::run_message_loop();

    Ok(())
}
