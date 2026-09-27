# Changelog

## [Unreleased]

## [0.3.0] - 2026-09-26

### Added

- Added asynchronous remote fetching, recursive discovery, and image decoding.
- Added opt-in `--performance-metrics` logging to stderr and `perf.log`.
- Added aggregate file-list and recursive-discovery timings with item counts.
- Added decoded-image pixel and byte limits for memory protection.
- Added a Windows UI smoke test and expanded failure-path coverage.

### Improved

- Improved worker cancellation and shutdown by joining background tasks before window teardown.
- Improved remote cache safety with atomic temporary-file publication.
- Improved multi-window startup by loading remote URLs asynchronously.
- Improved thumbnail cache consistency, recursive discovery deduplication, and asynchronous dimension filtering.

### Fixed

- Prevented stale background workers from posting results after a window is destroyed.
- Prevented recursive auto-reload from traversing large trees on the UI thread.

## [0.2.1] - 2026-09-24

- Added MIT license text and open-source project policies.
- Added contributor, security, and code-of-conduct documentation.
- Added Dependabot updates and a scheduled Cargo dependency audit.
- Installer no longer offers a desktop shortcut and adds its per-user install directory to `PATH`.

## [0.2.0] - 2026-09-24

### Added

- Added a per-user Windows installer with optional desktop shortcuts and image file associations.
- Added CI and release validation for installer installation, launch, registry registration, and uninstall.
- Added `--move DIRECTORY` and the `m` action for moving the current image.
- Added delete and move commands to the application menus and default keybindings.

### Improved

- Added cross-volume move support by falling back to copy-then-delete when Windows cannot rename across drives.
- Added `.jxl` and `.raw` image associations to the installer.
- Reset slideshow timing after manual image navigation and image moves.
- Improved rendering cleanup when replacing the current image.
- Added downloadable Windows installer artifacts to CI and release packaging.

### Fixed

- Installer uninstall now removes Open With registry subkeys instead of leaving stale associations behind.
- Release validation now checks the actual association registry keys after uninstall.
