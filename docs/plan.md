# fehrust Project Plan

## Current scope

fehrust is a Windows-only image viewer written in Rust. It uses Win32 for
windows and input, Direct2D and DirectWrite for rendering, and Windows
Imaging Component (WIC) for image decoding. Released binaries target
Windows x86_64.

The project intentionally does not provide a cross-platform GUI abstraction.
Features that depend on X11, POSIX signals, terminal control, or feh's
`~/.fehbg` script are out of scope for the Windows port.

## Current architecture

| Area | Implementation |
|------|----------------|
| CLI parsing | `clap` derive |
| Window and message loop | Win32 through `windows` |
| Rendering and text | Direct2D and DirectWrite |
| Image decoding and saving | WIC through `windows` |
| EXIF metadata | `kamadak-exif` |
| HTTP image loading | synchronous `ureq`, cached in a temporary directory |
| Directory traversal | `walkdir` |
| Natural sorting | `natord` |
| File watching | Win32 `ReadDirectoryChangesW` |
| Installer | Inno Setup per-user installer |

The main modules are described in the architecture section of the
[README](../README.md). The implementation is currently organized around:

- CLI and early-exit modes in `config.rs` and `app.rs`.
- File discovery, sorting, filtering, navigation, and file-list persistence
  in `filelist.rs`.
- WIC loading/saving in `image_loader.rs`.
- Win32 windowing and event dispatch in `window.rs` and `input.rs`.
- Direct2D rendering in `renderer.rs`.
- Thumbnail/index rendering in `thumbnail.rs`.
- Wallpaper, file watching, menus, actions, and JPEG rotation in their
  respective modules.

## Implemented capabilities

The current release includes:

- Single-image, slideshow, thumbnail, index, multi-window, and list modes.
- Recursive discovery, sorting, randomization, dimension filtering, and
  file-list load/save.
- Zoom, pan, rotate, flip, fullscreen, borderless, HiDPI, and overlays.
- EXIF display and orientation handling.
- Wallpaper integration and supported image file associations.
- HTTP/HTTPS image loading, temporary caching, and file auto-reload.
- Custom actions, image save, deletion, moving, and lossless JPEG rotation.

The authoritative user-facing option and shortcut reference is
[README.md](../README.md); this document is not a second CLI specification.

## Planned work

Potential future work, in priority order:

1. Add more end-to-end tests for malformed images, HTTP failures, file
   watching, moving/deleting, and installer upgrades.
2. Add fixture coverage for more WIC formats and EXIF orientations.
3. Improve error reporting and cancellation behavior for long-running image
   loads and external actions.
4. Improve accessibility and keyboard discoverability without changing the
   keyboard-driven workflow.
5. Evaluate additional release targets only if there is a clear Windows
   architecture requirement.

Plans are intentionally kept smaller than the implementation: completed
features should be documented in the README and changelog rather than left
in a historical task list.
