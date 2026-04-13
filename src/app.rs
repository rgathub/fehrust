use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::config::{Options, ViewMode};
use crate::filelist::FileList;
use crate::format::expand_format;
use crate::image_loader::{ImageLoader, LoadedImage};
use crate::renderer::Renderer;
use crate::transforms;
use crate::window;

const SLIDESHOW_TIMER_ID: usize = 1;
const DEFAULT_WIDTH: u32 = 800;
const DEFAULT_HEIGHT: u32 = 600;

pub struct AppState {
    pub options: Options,
    pub filelist: FileList,
    pub renderer: Renderer,
    pub image_loader: ImageLoader,
    pub current_image: Option<LoadedImage>,
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

    window_width: u32,
    window_height: u32,
}

impl AppState {
    pub fn new(options: Options) -> windows::core::Result<Self> {
        let renderer = Renderer::new()?;
        let image_loader = ImageLoader::new()?;

        let mut filelist = FileList::collect(&options.files, options.recursive);

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

        // Jump to start-at file
        if let Some(ref start) = options.start_at {
            filelist.jump_to(start);
        }

        Ok(Self {
            options,
            filelist,
            renderer,
            image_loader,
            current_image: None,
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

        if let Some(file) = self.filelist.current() {
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
                }
                Err(e) => {
                    eprintln!(
                        "Failed to load {}: {e}",
                        file.path.display()
                    );
                    self.current_image = None;
                }
            }
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

    pub fn paint(&self) -> windows::core::Result<()> {
        let filename = self
            .filelist
            .current()
            .map(|f| f.path.to_string_lossy().into_owned())
            .unwrap_or_default();

        self.renderer.render(
            self.zoom,
            self.pan_x,
            self.pan_y,
            self.rotation,
            self.options.draw_filename,
            &filename,
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
    let fullscreen = options.fullscreen;
    let borderless = options.borderless;
    let scale_down = options.scale_down;
    let slideshow_delay = options.slideshow_delay;

    let (init_w, init_h) = options
        .parse_geometry()
        .map(|(w, h, _, _)| (w, h))
        .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT));

    let mut state = AppState::new(options)?;

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

    // Load first image
    state.load_current_image();

    if scale_down {
        state.zoom_to_fit();
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
