# Changelog

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
