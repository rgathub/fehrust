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

The current suite contains 147 unit tests and 8 integration tests:

| Suite | Coverage |
|-------|----------|
| Unit tests in `src/` and sibling `*_tests.rs` modules | Configuration parsing, format expansion, action expansion, EXIF formatting/orientation, file-list navigation/sorting/persistence, key bindings, overlay formatting, renderer matrix math, and transform math |
| `tests/cli_test.rs` | `--help`, `--version`, list mode, and custom-list mode |
| `tests/image_loader_test.rs` | Windows fixture discovery, fixture metadata, and the CLI `--loadable` path |
| `tests/ui_smoke_test.rs` | Native window creation, visibility, resize handling, and clean shutdown |

The measured baseline on Windows is currently 33.24% line coverage
(1,231 of 3,703 executable lines). CI enforces a minimum of 30% total line
coverage. The extracted sibling test modules are not included in the
production coverage denominator. The uncovered portion is dominated by
Win32/Direct2D windowing, rendering, menus, wallpaper integration, and
interactive input.
The UI smoke test runs in normal CI test runs but is excluded from the
instrumented coverage run because native window teardown is not stable under
LLVM coverage instrumentation.

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

Large unit-test groups live in sibling modules such as
`filelist_tests.rs`, `config_tests.rs`, `keybindings_tests.rs`,
`exif_tests.rs`, and `renderer_tests.rs`; smaller test groups remain beside
the implementation they exercise.

The UI smoke test uses the Win32 APIs already required by the application,
rather than a separate automation service. It is intentionally limited to
stable window-lifecycle checks; pixel-level rendering checks should use
dedicated screenshot fixtures because Direct2D content is not exposed as
child UI Automation controls.
