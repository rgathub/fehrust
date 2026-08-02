# fehrust

A fast, keyboard-driven image viewer for **Windows**, inspired by [feh](https://feh.finalrewind.org/). Built in Rust using native Windows APIs (Win32, Direct2D, WIC, DirectWrite) — no cross-platform GUI toolkit, no Electron, no bloat.

## Current Status

fehrust is a functional Windows-native image viewer with single-image, slideshow, thumbnail, contact-sheet, multi-window, list, filtering, wallpaper, HTTP/HTTPS, EXIF, captions, file watching, and custom action support. The project currently produces Windows x86_64 binaries; Linux and macOS are not supported.

## Features

- **Hardware-accelerated rendering** via Direct2D
- **Broad format support** via Windows Imaging Component (WIC) — JPEG, PNG, BMP, GIF, TIFF, WebP, HEIF, AVIF, ICO, SVG, RAW (CR2, NEF, ARW, DNG), and any format with an installed WIC codec
- **Keyboard-driven** with configurable keybindings
- **Multiple view modes** — single image, slideshow, thumbnails, index/contact sheet, multi-window, list
- **EXIF metadata** display and auto-rotation
- **Image transforms** — zoom, pan, rotate, flip with transparency checkerboard
- **Desktop integration** — set wallpaper, fullscreen, multi-monitor, HiDPI, context menus
- **HTTP/HTTPS** image fetching with local caching
- **File watching** for auto-reload on changes
- **Custom actions** — run shell commands with format string expansion

## Installation

```
cargo install --path .
```

Or build from source:

```
cargo build --release
```

The binary will be at `target\release\fehrust.exe`.

## Requirements

- Windows with the Windows Imaging Component (WIC) codecs needed for the image formats you want to open
- Rust stable toolchain (edition 2024)

## Usage

```
fehrust [OPTIONS] [FILES/DIRECTORIES...]
```

### Examples

```bash
# View a single image
fehrust photo.jpg

# View all images in a directory
fehrust C:\Photos

# Recursive slideshow with 3-second delay
fehrust -r -D 3 C:\Photos

# Fullscreen, scale down large images
fehrust -F -. photo.jpg

# Thumbnail grid mode
fehrust -t C:\Photos

# Open each image in its own window
fehrust --multiwindow *.jpg

# List image info without opening a window
fehrust -L C:\Photos

# Set wallpaper (fit mode)
fehrust --wallpaper-mode fill photo.jpg
# Then press 'w' in the viewer

# Sort by file size, reversed
fehrust -S size -n C:\Photos

# Filter by dimensions
fehrust --min-dimension 1920x1080 C:\Photos

# Custom actions on numeric keys
fehrust --action1 "explorer /select,%f" --action2 "copy %f C:\Favorites\" C:\Photos

# Load images from a URL
fehrust https://example.com/image.jpg

# Watch for file changes and auto-reload
fehrust --auto-reload C:\Screenshots
```

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `Space` / `→` | Next image |
| `Backspace` / `←` | Previous image |
| `Home` | First image |
| `End` | Last image |
| `PgUp` | Jump forward 5 images |
| `PgDn` | Jump back 5 images |

### Zoom & Pan

| Key | Action |
|-----|--------|
| `+` / Numpad `+` | Zoom in |
| `-` / Numpad `-` | Zoom out |
| `*` | Fit to window |
| `0` | Actual size (100%) |
| `↑` / `↓` | Scroll up / down |
| Mouse drag | Pan image |
| `Ctrl` + Scroll wheel | Zoom in/out |

### Transforms

| Key | Action |
|-----|--------|
| `>` (`.`) | Rotate clockwise 90° |
| `<` (`,`) | Rotate counter-clockwise 90° |
| `\` | Flip horizontal |
| `/` | Flip vertical |
| `Ctrl+Shift+>` | Lossless rotate CW (saves to disk) |
| `Ctrl+Shift+<` | Lossless rotate CCW (saves to disk) |

### Display

| Key | Action |
|-----|--------|
| `d` | Toggle filename overlay |
| `i` | Toggle info overlay (EXIF, dimensions, etc.) |
| `x` / `F11` | Toggle fullscreen |

### Modes

| Key | Action |
|-----|--------|
| `z` | Enter zoom mode (drag to zoom) |
| `r` | Enter rotate mode (drag to rotate) |
| `Escape` | Exit current mode / quit |

### Actions

| Key | Action |
|-----|--------|
| `p` | Pause/resume slideshow |
| `s` | Save a copy of the current image |
| `w` | Set current image as desktop wallpaper |
| `Del` | Remove image from list (not from disk) |
| `Enter` | Execute default action (`--action`) |
| `1`–`9` | Execute custom action (`--action1` through `--action9`) |
| `q` | Quit |

### Mouse

| Input | Action |
|-------|--------|
| Scroll wheel | Next / previous image |
| `Ctrl` + Scroll | Zoom in / out |
| Left-click drag | Pan image |
| Right-click | Context menu |
| Double-click (thumbnails) | Open image in viewer |

All keybindings are configurable with `--key-binding "key action"`.

## Command-Line Options

### Display

| Flag | Description |
|------|-------------|
| `-F`, `--fullscreen` | Start in fullscreen mode |
| `-.`, `--scale-down` | Scale down images larger than the window |
| `-g`, `--geometry WxH+X+Y` | Set window size and position |
| `-x`, `--borderless` | Borderless window |
| `-d`, `--draw-filename` | Show filename overlay |
| `--draw-info` | Show info overlay on start |
| `-Y`, `--hide-pointer` | Hide mouse cursor |
| `--zoom LEVEL` | Default zoom (percent, "max", or "fill") |
| `-^`, `--title FORMAT` | Window title format string |

### File Selection

| Flag | Description |
|------|-------------|
| `-r`, `--recursive` | Recurse into directories |
| `-S`, `--sort MODE` | Sort: name, filename, dirname, mtime, size, width, height, pixels, format, none |
| `-n`, `--reverse` | Reverse sort order |
| `-z`, `--randomize` | Randomize file list |
| `--filelist FILE` | Load file paths from a text file |
| `--filelist-save FILE` | Save collected file list to a text file |
| `--start-at FILE` | Start at a specific file |
| `--min-dimension WxH` | Only show images at least this large |
| `--max-dimension WxH` | Only show images at most this large |

### Modes

| Flag | Description |
|------|-------------|
| `-D`, `--slideshow-delay SEC` | Slideshow auto-advance interval |
| `--on-last-slide ACTION` | Behavior on last slide: resume, quit, hold |
| `-t`, `--thumbnails` | Thumbnail grid mode |
| `--index` | Contact sheet / index mode |
| `--multiwindow` | Open each image in a separate window |
| `-L`, `--list` | Print file info to stdout and exit |
| `--customlist FORMAT` | Print custom-formatted file info and exit |
| `--loadable` | Print paths of loadable images and exit |
| `--unloadable` | Print paths of unloadable images and exit |

### Actions & Integration

| Flag | Description |
|------|-------------|
| `--action CMD` | Default action on Enter (supports `%f`, `%n`, `%u`, `%l`) |
| `--action1`–`--action9 CMD` | Custom actions on keys 1–9 |
| `--wallpaper-mode MODE` | Wallpaper style: center, fill, fit, stretch, tile, span |
| `--auto-reload` | Watch files for changes and auto-reload |
| `--caption-path DIR` | Load caption text from `DIR/{stem}.txt` sidecar files |
| `--key-binding "KEY ACTION"` | Override a keybinding (repeatable) |

### Output Control

| Flag | Description |
|------|-------------|
| `-q`, `--quiet` | Suppress non-error output |
| `--verbose` | Verbose output |

## Format Strings

Used in `--title`, `--list-format`, `--customlist`, and `--action`:

| Specifier | Expands to |
|-----------|------------|
| `%f` | Full file path |
| `%n` | File name only |
| `%u` | Current index (1-based) |
| `%l` | Total file count |
| `%w` | Image width (pixels) |
| `%h` | Image height (pixels) |
| `%z` | Current zoom level |
| `%s` | File size (bytes) |
| `%v` | fehrust version |
| `%a` | Playing/paused status |

## Architecture

```
src/
├── main.rs           Entry point, CLI parsing, mode dispatch
├── app.rs            Application state, message loop orchestrator
├── window.rs         Win32 window creation, WndProc, fullscreen, DPI
├── renderer.rs       Direct2D rendering pipeline, text overlays
├── image_loader.rs   WIC image loading, saving, format conversion
├── filelist.rs       File collection, sorting, filtering, navigation
├── input.rs          Keyboard and mouse event handling
├── keybindings.rs    Configurable key→action mapping
├── menu.rs           Win32 context menu
├── thumbnail.rs      Thumbnail/index grid view
├── overlay.rs        Info overlay text construction
├── exif.rs           EXIF metadata reading (kamadak-exif)
├── format.rs         Format string (%f, %u, etc.) expansion
├── transforms.rs     Zoom/pan math helpers
├── wallpaper.rs      Desktop wallpaper setting via Win32 registry + API
├── actions.rs        Custom shell command execution
├── filewatcher.rs    Directory change notification (Win32)
├── http.rs           HTTP/HTTPS image fetching with temp-file caching
├── jpeg_rotate.rs    Lossless rotation via WIC transforms
├── slideshow.rs      Slideshow timer logic
└── config.rs         CLI options (clap), enums, geometry parsing
```

### Technology Stack

| Component | Technology |
|-----------|-----------|
| Windowing | Win32 API (`CreateWindowExW`, message loop) |
| Rendering | Direct2D (hardware-accelerated) |
| Image decoding | Windows Imaging Component (WIC) |
| Text rendering | DirectWrite |
| EXIF | `kamadak-exif` crate |
| CLI parsing | `clap` (derive) |
| HTTP | `ureq` |
| Directory walk | `walkdir` |
| Natural sorting | `natord` |

## Development and Testing

Run these commands from the repository root on Windows:

```powershell
# Check formatting
cargo fmt -- --check

# Run lint checks
cargo clippy -- -D warnings

# Run the complete test suite
cargo test

# Build the optimized executable
cargo build --release
```

The CLI tests cover help, version, list, custom-list, and loadable modes. The image-loader integration tests validate Windows fixture handling and are run with:

```powershell
cargo test --test image_loader_test
```

## Compared to feh

fehrust is a Windows-native reimplementation of feh's core functionality. Key differences:

| | feh | fehrust |
|-|-----|---------|
| Platform | Linux/X11 | Windows |
| Rendering | Imlib2 (software) | Direct2D (GPU) |
| Image loading | Imlib2 | WIC (extensible codecs) |
| Language | C | Rust |
| Windowing | X11/Xlib | Win32 API |
| HTTP | libcurl | ureq |
| EXIF | libexif | kamadak-exif |
| Config file | `~/.config/feh/` | CLI flags only |

### Not ported

- X11-specific features (embedded windows, Enlightenment IPC)
- POSIX signals (`SIGUSR1`/`SIGUSR2`)
- `~/.fehbg` script (Windows wallpaper is persistent natively)
- Raw terminal/stdin control

## License

MIT

## Release Process

Releases are Windows x86_64 GitHub Releases created by `.github/workflows/release.yml`.

1. Choose the next semantic version, such as `0.1.2`.
2. Update the `version` field in `Cargo.toml`.
3. Run `cargo check` so `Cargo.lock` records the new package version.
4. Run the formatting, lint, build, and test commands from [Development and Testing](#development-and-testing).
5. Commit and push the version change to `main`:

   ```powershell
   git add Cargo.toml Cargo.lock
   git commit -m "Release v0.1.2"
   git push origin main
   ```

6. Create and push an annotated `v*` tag:

   ```powershell
   git tag -a v0.1.2 -m "Release v0.1.2"
   git push origin v0.1.2
   ```

Pushing the tag starts the Windows release workflow. It builds and tests the project, packages `fehrust.exe`, `README.md`, and any license files into `fehrust-v0.1.2-windows-x86_64.zip`, and creates a GitHub Release with generated release notes.
