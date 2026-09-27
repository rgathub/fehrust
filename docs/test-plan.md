# fehrust Test Plan

## Verification commands

Run these commands from the repository root on Windows:

```powershell
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

For line coverage, install `cargo-llvm-cov` and run:

```powershell
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --summary-only
```

CI runs the same coverage command and uploads an LCOV artifact for each push
and pull request.

## Current automated coverage

The current suite contains 129 unit tests and 7 integration tests:

| Suite | Coverage |
|-------|----------|
| Unit tests in `src/` | Configuration parsing, format expansion, action expansion, EXIF formatting/orientation, file-list navigation/sorting/persistence, key bindings, overlay formatting, renderer matrix math, and transform math |
| `tests/cli_test.rs` | `--help`, `--version`, list mode, and custom-list mode |
| `tests/image_loader_test.rs` | Windows fixture discovery, fixture metadata, and the CLI `--loadable` path |

The measured baseline on Windows is currently 38.42% line coverage
(1,447 of 3,766 executable lines). This is a baseline, not a quality target:
the uncovered portion is dominated by Win32/Direct2D windowing, rendering,
menus, wallpaper integration, file watching, and interactive input.

## Coverage priorities

Future tests should focus on behavior with the highest user or data-loss
impact:

1. Invalid and unsupported image files, including WIC decoding failures.
2. Move, delete, save, and lossless rotation failures and cross-volume cases.
3. HTTP status errors, invalid responses, cache collisions, and interrupted
   downloads.
4. File watcher reload behavior and window lifecycle/error paths.
5. Installer upgrade, rollback, file association, and PATH behavior.

Interactive GUI tests should remain Windows-only and non-destructive. Tests
that modify user files should use temporary directories and explicit fixtures.
