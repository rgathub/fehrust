# fehrust — Windows-native feh Image Viewer in Rust

## Problem Statement

Recreate the [feh](https://github.com/derf/feh) image viewer in Rust, targeting **Windows only** using **native Windows APIs** (Win32, Direct2D, WIC) instead of X11/Imlib2. The goal is a fast, keyboard-driven, command-line image viewer that preserves feh's core UX while being idiomatic Rust and a first-class Windows citizen.

## Approach

Replace feh's Unix/X11 stack with Windows equivalents:

| feh (C/X11)          | fehrust (Rust/Windows)                          |
|----------------------|-------------------------------------------------|
| X11/Xlib windowing   | Win32 API via `windows-rs` (CreateWindowExW, message loop) |
| Imlib2 image loading | Windows Imaging Component (WIC) via `windows-rs` |
| Imlib2 rendering     | Direct2D via `windows-rs` for HW-accelerated rendering |
| Xinerama multi-mon   | Win32 `EnumDisplayMonitors` / `MonitorFromWindow` |
| X11 wallpaper atoms  | `SystemParametersInfoW(SPI_SETDESKWALLPAPER)` |
| libcurl HTTP         | `reqwest` (or `ureq` for sync/no-tokio) |
| libexif              | `kamadak-exif` crate |
| getopt_long          | `clap` crate |
| gib_list linked list | `Vec<T>` / standard iterators |
| X11 menu system      | Win32 popup menus (`CreatePopupMenu`, `TrackPopupMenu`) |
| POSIX signals        | Not applicable on Windows (use named pipes or just keyboard) |
| select() event loop  | Win32 message loop (`GetMessage`/`PeekMessage`) |
| inotify file watch   | `ReadDirectoryChangesW` or `notify` crate |
| termios/stdin ctrl   | Not ported (Windows terminal is different; use keyboard only) |

## Architecture

```
fehrust/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, CLI parsing, mode dispatch
│   ├── app.rs               # Application state, main message loop
│   ├── window.rs            # Win32 window creation, WndProc, message handling
│   ├── renderer.rs          # Direct2D rendering pipeline
│   ├── image_loader.rs      # WIC image loading + format detection
│   ├── filelist.rs          # File collection, sorting, filtering, navigation
│   ├── slideshow.rs         # Slideshow mode logic, timer management
│   ├── thumbnail.rs         # Thumbnail/index mode
│   ├── input.rs             # Key bindings, mouse handling, event dispatch
│   ├── menu.rs              # Win32 context menus
│   ├── wallpaper.rs         # Windows desktop wallpaper setting
│   ├── overlay.rs           # Text overlays (filename, info, EXIF, captions)
│   ├── exif.rs              # EXIF reading and formatting
│   ├── http.rs              # HTTP/HTTPS image fetching
│   ├── transforms.rs        # Zoom, pan, rotate, flip operations
│   ├── config.rs            # Options struct, themes, key binding config
│   └── format.rs            # Format string (%f, %u, %l, etc.) expansion
```

## Key Crates

| Crate              | Purpose                                         |
|--------------------|-------------------------------------------------|
| `windows`          | Win32 API, Direct2D, WIC, shell APIs             |
| `clap`             | CLI argument parsing                             |
| `image`            | Fallback image decoding (formats WIC doesn't support) |
| `kamadak-exif`     | EXIF metadata reading                            |
| `walkdir`          | Recursive directory traversal                    |
| `ureq` or `reqwest`| HTTP/HTTPS image fetching (sync preferred)       |
| `notify`           | File system watching for auto-reload             |
| `natord`           | Natural/version-aware string sorting             |

## Phased Implementation Plan

### Phase 1: Foundation — Single Image Viewer
Core window + image rendering for a single image file.

- **1a**: Project scaffolding (Cargo.toml, crate structure, `windows-rs` setup)
- **1b**: Win32 window creation with message loop (`CreateWindowExW`, `WndProc`)
- **1c**: WIC image loading (decode any image file → HBITMAP or Direct2D bitmap)
- **1d**: Direct2D rendering pipeline (draw loaded image in window, handle resize)
- **1e**: Basic zoom/pan (mouse wheel zoom, click-drag pan, fit-to-window)
- **1f**: CLI parsing with `clap` (accept file paths, basic flags like `--fullscreen`)

### Phase 2: File List & Navigation
Navigate through multiple images.

- **2a**: File list collection from args (files, directories, recursive with `walkdir`)
- **2b**: Sorting modes (name, filename, size, mtime, dimensions, format, random)
- **2c**: Slideshow navigation (next/prev/jump-fwd/jump-back/first/last/random)
- **2d**: Slideshow auto-advance timer (`SetTimer`/`WM_TIMER`)
- **2e**: Window title formatting with `%f`, `%u`, `%l`, etc. specifiers

### Phase 3: Keyboard & Input System
Full keyboard-driven control matching feh's bindings.

- **3a**: Key binding system (configurable keysym → action mapping)
- **3b**: Default key bindings matching feh (arrows, +/-, q, x, d, etc.)
- **3c**: Mouse button bindings (scroll wheel, middle-click, right-click)
- **3d**: Interactive modes: pan (MODE_PAN), zoom (MODE_ZOOM), rotate (MODE_ROTATE)

### Phase 4: Image Transforms & Overlays
Visual features beyond basic viewing.

- **4a**: Rotation (90°/270° and arbitrary angle)
- **4b**: Flip/mirror operations
- **4c**: Transparency checkerboard pattern
- **4d**: Text overlays — filename, info string, zoom level
- **4e**: EXIF reading and overlay display (`kamadak-exif`)
- **4f**: EXIF auto-rotation on load
- **4g**: Caption support (load from sidecar files, display as overlay)

### Phase 5: Advanced Modes
Thumbnail, index, multi-window, and list modes.

- **5a**: Thumbnail mode (grid of thumbnails, click to view)
- **5b**: Index/contact sheet mode (render all thumbnails to single image)
- **5c**: Multi-window mode (one window per image)
- **5d**: List mode (text output of file info to stdout)
- **5e**: Custom list format mode (`--customlist`)
- **5f**: Loadables/unloadables filter modes

### Phase 6: Desktop Integration
Windows-specific integrations.

- **6a**: Wallpaper setting (`SystemParametersInfoW` with tile/center/stretch/fill/fit)
- **6b**: Fullscreen toggle (borderless fullscreen window, `WS_POPUP`)
- **6c**: Multi-monitor support (`EnumDisplayMonitors`, per-monitor DPI)
- **6d**: Context menu (Win32 `CreatePopupMenu` with feh-like actions)
- **6e**: Hide cursor option

### Phase 7: Network & File Watching
HTTP support and auto-reload.

- **7a**: HTTP/HTTPS image fetching with `ureq` (download to temp, display)
- **7b**: `--keep-http` caching of downloaded images
- **7c**: File watching with `notify` crate for `--auto-reload`
- **7d**: Filelist load/save (`--filelist`)

### Phase 8: Custom Actions & Polish
Power-user features.

- **8a**: Custom action system (`--action` with format string expansion, `CreateProcess`)
- **8b**: Image save (export current view with transforms applied)
- **8c**: In-place lossless JPEG rotation (via `jpegtran` or equivalent)
- **8d**: Dimension filtering (`--min-dimension`, `--max-dimension`)
- **8e**: Image deletion/removal from list
- **8f**: Per-monitor DPI awareness and HiDPI scaling

## Design Decisions

### Why Direct2D over GDI?
- Hardware-accelerated rendering (smooth zoom/pan at high res)
- Built-in support for bitmap scaling with quality interpolation modes
- Matrix transforms for rotation without manual pixel manipulation
- Better alpha/transparency handling

### Why WIC over `image` crate for primary loading?
- Native Windows codec support (JPEG, PNG, BMP, GIF, TIFF, HEIF, WebP, RAW via codecs)
- Automatic codec extension via Windows codec packs (e.g., install a codec, feh sees it)
- Direct integration with Direct2D (WIC → ID2D1Bitmap with zero-copy)
- Falls back to `image` crate for any format WIC can't handle

### Event Loop Design
Single-threaded Win32 message loop (matches feh's single-threaded X11 design):
```
main() → parse_options() → init_app()
  → match mode {
      Slideshow → init_slideshow(),
      Thumbnail → init_thumbnail(),
      ...
    }
  → loop {
      GetMessage() / PeekMessage()
      TranslateMessage() / DispatchMessage()
      // WndProc handles WM_PAINT, WM_KEYDOWN, WM_MOUSEWHEEL, WM_TIMER, etc.
    }
```

### State Management
```rust
struct App {
    window: HWND,
    render_target: ID2D1HwndRenderTarget,
    filelist: FileList,
    current_image: Option<LoadedImage>,
    zoom: f64,         // 0.002..2000.0
    pan: (f64, f64),   // image offset
    rotation: f64,     // degrees
    mode: ViewMode,    // Normal, Pan, Zoom, Rotate
    options: Options,
    key_bindings: KeyBindings,
}
```

## Features NOT Ported (and why)

- **POSIX signals (SIGUSR1/2)**: No equivalent on Windows. Use named pipe IPC or just keyboard.
- **Enlightenment WM IPC**: X11-specific. Windows wallpaper API is simpler.
- **stdin terminal control**: feh's raw terminal mode doesn't translate to Windows console well. Focus on window keyboard input.
- **X11 embedded window (`--window-id`)**: X11-specific reparenting. Could do Win32 child window embedding if needed later.
- **gib_style text rendering**: Use DirectWrite via Direct2D instead.
- **`~/.fehbg` script**: Windows wallpaper is persistent natively.

## Notes

- The `windows` crate (`windows-rs`) provides safe Rust bindings to the entire Win32 API including Direct2D, WIC, DirectWrite, Shell, and more. It's the official Microsoft-maintained crate.
- Direct2D rendering requires a `ID2D1Factory` and `ID2D1HwndRenderTarget`. The render target is tied to the window and handles DPI automatically.
- WIC (`IWICImagingFactory`) can decode directly to `IWICBitmap` which converts to `ID2D1Bitmap` for rendering.
- For natural sorting (like feh's `strverscmp`), use the `natord` crate or implement a simple version-aware compare.
