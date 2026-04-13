use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::config::{Options, ViewMode};
use crate::exif;
use crate::filelist::FileList;
use crate::format::expand_format;
use crate::image_loader::{ImageLoader, LoadedImage};
use crate::keybindings::{self, KeyMap};
use crate::renderer::Renderer;
use crate::thumbnail::ThumbnailView;
use crate::transforms;
use crate::window;

use std::path::Path;

const SLIDESHOW_TIMER_ID: usize = 1;
const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;

pub struct AppState {
    pub options: Options,
    pub filelist: FileList,
    pub renderer: Renderer,
    pub image_loader: ImageLoader,
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

    window_width: u32,
    window_height: u32,
}

impl AppState {
    pub fn new(options: Options) -> windows::core::Result<Self> {
        let renderer = Renderer::new()?;
        let image_loader = ImageLoader::new()?;

        let mut filelist = if let Some(ref fl_path) = options.filelist {
            FileList::from_filelist(Path::new(fl_path))
        } else {
            FileList::collect(&options.files, options.recursive)
        };

        if filelist.is_empty() {
            return Err(windows::core::Error::new(
                E_FAIL,
                "No image files found",
            ));
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
        if let Some(ref save_path) = options.filelist_save {
            if let Err(e) = filelist.save_filelist(Path::new(save_path)) {
                eprintln!("Failed to save filelist: {e}");
            }
        }

        // Jump to start-at file
        if let Some(ref start) = options.start_at {
            filelist.jump_to(start);
        }

        let keybindings = keybindings::build_keymap(&options.key_binding);
        let numbered_actions = options.numbered_actions();

        Ok(Self {
            options,
            filelist,
            renderer,
            image_loader,
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
            window_width: DEFAULT_WIDTH,
            window_height: DEFAULT_HEIGHT,
        })
    }

    pub fn load_current_image(&mut self) {
        self.pan_x = 0.0;
        self.pan_y = 0.0;
        self.rotation = 0.0;
        self.flip_h = false;
        self.flip_v = false;
        self.current_exif = None;
        self.current_caption = None;

        if let Some(file) = self.filelist.current() {
            let exif_info = exif::read_exif(&file.path);

            // Load caption if caption_path is set
            if let Some(ref caption_path) = self.options.caption_path {
                if let Some(stem) = file.path.file_stem() {
                    let caption_file = Path::new(caption_path)
                        .join(format!("{}.txt", stem.to_string_lossy()));
                    if let Ok(text) = std::fs::read_to_string(&caption_file) {
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            self.current_caption = Some(trimmed);
                        }
                    }
                }
            }

            match self.image_loader.load(&file.path) {
                Ok(image) => {
                    if let Err(e) = self.renderer.load_bitmap(
                        &image,
                        self.image_loader.wic_factory(),
                    ) {
                        eprintln!("Failed to create bitmap: {e}");
                    }
                    self.current_image = Some(image);
                    self.zoom_to_fit();

                    // Apply EXIF auto-rotation
                    if let Some(ref exif) = exif_info {
                        if exif.orientation != 1 {
                            let (rot, fh, fv) =
                                exif::exif_orientation_to_rotation(exif.orientation);
                            self.rotation = rot;
                            self.flip_h = fh;
                            self.flip_v = fv;
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to load {}: {e}",
                        file.path.display()
                    );
                    self.current_image = None;
                }
            }

            self.current_exif = exif_info;
        }

        self.update_title();
    }

    pub fn zoom_to_fit(&mut self) {
        if let Some(ref img) = self.current_image {
            if self.window_width > 0 && self.window_height > 0 {
                self.zoom = transforms::fit_zoom(
                    img.width as f64,
                    img.height as f64,
                    self.window_width as f64,
                    self.window_height as f64,
                );
                self.pan_x = 0.0;
                self.pan_y = 0.0;
            }
        }
    }

    pub fn navigate_next(&mut self) {
        self.filelist.next();
        self.load_current_image();
    }

    pub fn navigate_prev(&mut self) {
        self.filelist.prev();
        self.load_current_image();
    }

    pub fn paint(&mut self) -> windows::core::Result<()> {
        // Thumbnail / index mode rendering
        if let Some(ref mut thumb_view) = self.thumbnail_view {
            if let Some(rt) = self.renderer.render_target() {
                return thumb_view.render(rt, &self.filelist, &self.image_loader);
            }
            return Ok(());
        }

        let filename = self
            .filelist
            .current()
            .map(|f| f.path.to_string_lossy().into_owned())
            .unwrap_or_default();

        let info_text = if self.options.draw_info {
            crate::overlay::build_info_string(self)
        } else {
            String::new()
        };

        self.renderer.render(
            self.zoom,
            self.pan_x,
            self.pan_y,
            self.rotation,
            self.flip_h,
            self.flip_v,
            self.options.draw_filename,
            &filename,
            self.options.draw_info,
            &info_text,
            self.dpi_scale,
        )
    }

    pub fn handle_resize(&mut self, width: u32, height: u32) -> windows::core::Result<()> {
        self.window_width = width;
        self.window_height = height;
        self.renderer.resize(width, height)?;

        // Re-fit image on resize if we're at fit-to-window zoom
        if self.options.scale_down {
            self.zoom_to_fit();
        }

        Ok(())
    }

    pub fn handle_timer(&mut self) {
        if !self.paused && self.filelist.len() > 1 {
            self.navigate_next();
        }
    }

    pub fn remove_current_from_list(&mut self, hwnd: HWND) {
        if !self.filelist.remove_current() {
            // List is empty, quit
            unsafe {
                let _ = DestroyWindow(hwnd);
            }
            return;
        }
        self.load_current_image();
        window::invalidate(hwnd);
    }

    pub fn update_title(&self) {
        if self.hwnd == HWND::default() {
            return;
        }

        let (img_w, img_h) = self
            .current_image
            .as_ref()
            .map(|i| (Some(i.width), Some(i.height)))
            .unwrap_or((None, None));

        let title = expand_format(
            &self.options.title,
            self.filelist.current(),
            self.filelist.current_index(),
            self.filelist.len(),
            self.zoom,
            img_w,
            img_h,
            self.paused,
        );

        window::update_title(self.hwnd, &title);
    }
}

pub fn run(options: Options) -> windows::core::Result<()> {
    // --- Early exit modes (no window needed) ---

    // List mode: print file info to stdout
    if options.list || options.customlist.is_some() {
        return run_list_mode(&options);
    }

    // Loadable/unloadable filter mode
    if options.loadable || options.unloadable {
        return run_filter_mode(&options);
    }

    // Multi-window mode
    if options.multiwindow {
        return run_multiwindow(options);
    }

    // Set DPI awareness before creating any windows
    window::set_dpi_awareness();

    let fullscreen = options.fullscreen;
    let borderless = options.borderless;
    let scale_down = options.scale_down;
    let slideshow_delay = options.slideshow_delay;
    let auto_reload = options.auto_reload;
    let thumb_mode = options.thumbnails;
    let index_mode = options.index;

    let (init_w, init_h) = options
        .parse_geometry()
        .map(|(w, h, _, _)| (w, h))
        .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT));

    let mut state = AppState::new(options)?;

    // Enter thumbnail/index mode if requested
    if thumb_mode || index_mode {
        state.thumbnail_view = Some(ThumbnailView::new(index_mode));
    }

    let hwnd = window::create_window(
        "fehrust",
        init_w,
        init_h,
        borderless,
        fullscreen,
    )?;

    state.hwnd = hwnd;

    // Get actual client size
    let mut rect = RECT::default();
    unsafe {
        let _ = GetClientRect(hwnd, &mut rect);
    }
    let client_w = (rect.right - rect.left) as u32;
    let client_h = (rect.bottom - rect.top) as u32;

    state.window_width = if client_w > 0 { client_w } else { init_w };
    state.window_height = if client_h > 0 { client_h } else { init_h };

    state
        .renderer
        .create_render_target(hwnd, state.window_width, state.window_height)?;

    // Load first image (skip in thumbnail/index mode — thumbnails are loaded lazily)
    if state.thumbnail_view.is_none() {
        state.load_current_image();

        if scale_down {
            state.zoom_to_fit();
        }
    }

    // Set up slideshow timer
    if let Some(delay) = slideshow_delay {
        let ms = (delay * 1000.0) as u32;
        if ms > 0 {
            unsafe {
                SetTimer(Some(hwnd), SLIDESHOW_TIMER_ID, ms, None);
            }
        }
    }

    // Store state pointer for WndProc
    window::set_app_state(&mut state as *mut AppState);

    // Initial paint
    window::invalidate(hwnd);

    // Start file watcher if --auto-reload is set
    if auto_reload {
        if let Some(file) = state.filelist.current() {
            if let Some(parent) = file.path.parent() {
                crate::filewatcher::start_watcher(parent.to_path_buf(), hwnd);
            }
        }
    }

    // Enter message loop
    window::run_message_loop();

    // Clean up timer
    if slideshow_delay.is_some() {
        unsafe {
            let _ = KillTimer(Some(hwnd), SLIDESHOW_TIMER_ID);
        }
    }

    Ok(())
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

    let fmt = options.customlist.as_deref().unwrap_or(&options.list_format);
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

    if let Some(first) = states.first_mut() {
        window::set_app_state(first as *mut AppState);
    }

    for state in &states {
        window::invalidate(state.hwnd);
    }

    window::run_message_loop();

    Ok(())
}
