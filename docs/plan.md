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

## Prioritized enhancement backlog

This is the actionable task list for the next development cycles. Tasks are
ordered by risk and user impact; security and performance work comes before
feature expansion. Each item includes a definition of done so progress can be
reviewed consistently.

### P0 — security and data integrity

- [ ] **Make remote-image caching transactional.** Write downloads to a
  temporary file, validate the completed file, then atomically rename it into
  the cache. Reject truncated or invalid cached entries and add tests for
  interrupted downloads, response-size limits, redirects, and invalid
  content.
- [ ] **Bound image decoding resources.** Enforce configurable pixel and
  decoded-memory limits before creating full WIC bitmaps. Reject or clearly
  report images that exceed the limits, including decompression-bomb-style
  inputs.
- [ ] **Make destructive file operations recoverable.** Require confirmation
  for permanent deletion, prefer the Windows Recycle Bin, and add collision
  handling plus rollback guarantees for cross-volume moves. Test permission
  failures, existing targets, read-only files, and partial failures.
- [ ] **Harden custom action execution.** Parse Windows command arguments with
  path-safe quoting instead of whitespace splitting, preserve spaces and
  special characters in substitutions, and document the trust model. Add
  tests for quoted arguments, empty substitutions, and filenames containing
  shell metacharacters.
- [ ] **Close file-watcher lifecycles.** Add cancellation and shutdown
  signaling so watcher threads and native notification handles are released
  when a window closes. Do not post messages to destroyed windows; test
  shutdown and rapid open/close cycles.
- [ ] **Harden release supply-chain controls.** Publish SHA-256 checksums and
  an SBOM, use least-privilege workflow permissions, pin third-party actions
  to reviewed versions, and add artifact-signing provenance where practical.

### P1 — performance and responsiveness

- [x] **Move network work off the UI thread.** Fetch remote images
  asynchronously with cancellation, progress, connection/read timeouts, and
  bounded concurrency. The window must remain responsive while a URL loads.
- [x] **Move large-image decoding off the UI thread.** Decode on a worker,
  cancel stale requests when navigation changes, and publish results only if
  they still match the selected file.
- [x] **Use bounded thumbnail decoding and eviction.** Decode thumbnails at
  their display size where WIC permits, cap the Direct2D bitmap cache, and
  evict least-recently-used entries when memory or item limits are reached.
- [x] **Reduce repeated rendering allocations.** Cache the DirectWrite factory
  and reusable Direct2D brushes; recreate device-dependent resources after a
  device-loss event and restore the current bitmap.
- [x] **Make directory discovery incremental.** Avoid blocking startup on very
  large recursive trees, expose progress/cancellation, and define symlink
  traversal behavior to prevent accidental scans outside the requested tree.
- [x] **Improve file-watcher relevance.** Re-arm watching when navigation
  enters another directory and debounce bursts of filesystem notifications
  before reloading.

### P2 — reliability and test coverage

- [ ] **Add integration coverage for failure paths.** Cover malformed and
  oversized images, all HTTP error classes, cache recovery, watcher events,
  delete/move failures, file-list parsing, and installer upgrade/uninstall
  behavior.
- [ ] **Add WIC format and metadata fixtures.** Include representative formats
  and EXIF orientations supported by the extension filter, and verify that
  unsupported or multi-frame images fail or degrade predictably.
- [ ] **Add GUI smoke tests.** Exercise window creation, navigation,
  resize/HiDPI, slideshow shutdown, thumbnail scrolling, multi-window mode,
  and device-loss recovery on the Windows CI runner.
- [ ] **Set coverage and regression gates.** Establish a baseline for
  non-GUI code, publish it in CI, and fail coverage checks only when the
  threshold policy is documented and the measured scope is stable.

### P3 — user experience and maintainability

- [ ] **Persist user preferences safely.** Add an explicit configuration
  location and schema/version migration for window state, keybindings,
  slideshow settings, and non-sensitive preferences.
- [ ] **Add navigation aids.** Support recent files, search/filtering, drag and
  drop, and clearer empty/error states without disrupting keyboard workflows.
- [ ] **Improve accessibility and discoverability.** Provide visible shortcut
  hints, high-contrast-friendly overlays, scalable text, and screen-reader
  labels where Win32 controls are exposed.
- [ ] **Expand supported image handling.** Detect formats using WIC capability
  or decode probing rather than relying only on a fixed extension list, and
  define behavior for animated formats.

Tasks should be moved to the changelog and README only after they are
implemented and verified; this section is intentionally the source of truth
for unfinished work.
