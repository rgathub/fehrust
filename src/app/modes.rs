use super::*;

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

    let hwnd = window::create_window("fehrust", init_w, init_h, borderless, fullscreen)?;

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
    if let Some(ms) = slideshow_timer_ms(slideshow_delay) {
        unsafe {
            SetTimer(Some(hwnd), SLIDESHOW_TIMER_ID, ms, None);
        }
    }

    // Store state pointer for WndProc
    window::set_app_state(hwnd, &mut state as *mut AppState);

    // Initial paint
    window::invalidate(hwnd);

    // Start file watcher if --auto-reload is set
    if auto_reload
        && state.watcher.is_none()
        && let Some(file) = state.filelist.current()
        && let Some(parent) = file.path.parent()
    {
        state.watcher = Some(crate::filewatcher::start_watcher(
            parent.to_path_buf(),
            hwnd,
        ));
    }
    state.start_background_tasks(hwnd);

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
