# fehrust — Test Coverage Plan

## Overview

fehrust has 21 source modules. This plan categorizes each by testability and defines concrete test suites.

### Testability Tiers

| Tier | Description | Modules |
|------|-------------|---------|
| **A — Pure logic** | No Win32 calls, unit-testable | config, format, transforms, keybindings, actions (expand), http (url_to_filename), exif (orientation/format), overlay (format_file_size) |
| **B — Filesystem** | Needs temp files/dirs, but no Win32 GUI | filelist (navigation/sort), exif (read_exif with fixture files) |
| **C — Win32 dependent** | Requires running Windows APIs | image_loader, renderer, window, menu, wallpaper, filewatcher, jpeg_rotate, thumbnail |
| **D — Integration** | Full app-level flows | app (run, multiwindow, list mode), input (dispatch) |

### Strategy

- **Tier A**: Inline `#[cfg(test)] mod tests` in each module — fast, no setup
- **Tier B**: `#[cfg(test)]` with temp files via `tempfile` crate (add as dev-dependency)
- **Tier C**: `tests/` integration tests that run on Windows CI only — gated behind `#[cfg(target_os = "windows")]`
- **Tier D**: CLI smoke tests via `assert_cmd` + `predicates` crates — run `fehrust.exe` with args and check exit code/stdout

---

## Phase 1 — Pure Logic Unit Tests (Tier A)

### 1A: config.rs (~12 tests)

| Test | Function | Asserts |
|------|----------|---------|
| on_last_slide_quit | `on_last_slide_action()` | "quit" → Quit |
| on_last_slide_hold | `on_last_slide_action()` | "hold" → Hold |
| on_last_slide_default | `on_last_slide_action()` | "resume" / "" / "foo" → Resume |
| wallpaper_mode_all | `wallpaper_mode()` | "center"→Center, "fill"→Fill, "fit"→Fit, "stretch"→Stretch, "tile"→Tile, "span"→Span |
| wallpaper_mode_default | `wallpaper_mode()` | "bogus" → Center |
| parse_geometry_wxh | `parse_geometry()` | "800x600" → (800,600,None,None) |
| parse_geometry_wxh_xy | `parse_geometry()` | "800x600+10+20" → (800,600,Some(10),Some(20)) |
| parse_geometry_invalid | `parse_geometry()` | "invalid" → None |
| parse_geometry_none | `parse_geometry()` | None field → None |
| parse_dimension_valid | `parse_dimension()` | Some("1920x1080") → Some((1920,1080)) |
| parse_dimension_invalid | `parse_dimension()` | Some("bad") → None |
| parse_dimension_none | `parse_dimension()` | None → None |

### 1B: format.rs (~14 tests)

| Test | Asserts |
|------|---------|
| expand_f | `%f` → full path |
| expand_n | `%n` → filename |
| expand_u | `%u` with index=0 → "1" (1-based) |
| expand_l | `%l` with total=42 → "42" |
| expand_z | `%z` with zoom=1.5 → "1.50" |
| expand_w | `%w` with width=1920 → "1920" |
| expand_h | `%h` with height=1080 → "1080" |
| expand_v | `%v` → package version string |
| expand_a_playing | `%a` paused=false → "playing" |
| expand_a_paused | `%a` paused=true → "paused" |
| expand_s | `%s` with size=Some(4096) → "4096" |
| expand_literal_percent | `%%` → "%" |
| expand_backslash_n | `\\n` → newline |
| expand_unknown | `%x` → "%x" (passed through) |
| expand_multiple | `"%u of %l — %n"` → combined |
| expand_none_file | file=None, `%f` → empty |
| expand_trailing_percent | `"hello%"` → "hello%" |

### 1C: transforms.rs (~6 tests)

| Test | Asserts |
|------|---------|
| fit_zoom_smaller | img 640x480, vp 1280x960 → 1.0 (no upscale) |
| fit_zoom_larger | img 1920x1080, vp 800x600 → ~0.417 |
| fit_zoom_exact | img 800x600, vp 800x600 → 1.0 |
| fit_zoom_tall_image | img 400x800, vp 800x600 → 0.75 |
| fit_zoom_wide_image | img 1600x400, vp 800x600 → 0.5 |
| constants | ZOOM_MIN=0.002, ZOOM_MAX=2000.0 |

### 1D: keybindings.rs (~18 tests)

| Test | Asserts |
|------|---------|
| default_has_quit | default_bindings\[VK_Q\] == Quit |
| default_has_next | default_bindings\[VK_SPACE\] == Next |
| default_has_prev | default_bindings\[VK_BACK\] == Prev |
| default_has_zoom | default_bindings\[VK_OEM_PLUS\] == ZoomIn |
| default_has_fullscreen | default_bindings\[VK_F11\] == ToggleFullscreen |
| default_count | default_bindings has ≥30 entries |
| parse_key_q | parse_key_name("q") → Some(VK_Q.0) |
| parse_key_escape | parse_key_name("escape") → Some(VK_ESCAPE.0) |
| parse_key_space | parse_key_name("space") → Some(VK_SPACE.0) |
| parse_key_f1 | parse_key_name("f1") → Some(VK_F1.0) |
| parse_key_invalid | parse_key_name("bogus") → None |
| parse_key_case | parse_key_name("ESCAPE") → Some (case-insensitive) |
| parse_action_next | parse_action_name("next") → Some(Next) |
| parse_action_aliases | "zoom_in"/"zoomin"/"zoom-in" → all ZoomIn |
| parse_action_invalid | parse_action_name("bogus") → None |
| build_keymap_empty | build_keymap(\[\]) == default_bindings() |
| build_keymap_override | build_keymap(\["q next"\]) → q maps to Next |
| build_keymap_invalid | build_keymap(\["invalid"\]) → ignored, defaults unchanged |

### 1E: actions.rs (~6 tests)

| Test | Asserts |
|------|---------|
| expand_f | `expand_action("echo %f", file, ..)` → "echo /path/to/img.jpg" |
| expand_n | `%n` → filename only |
| expand_u | `%u` with index=3, total=10 → "4" (1-based) |
| expand_l | `%l` with total=10 → "10" |
| expand_percent | `%%` → "%" |
| expand_mixed | `"cp %f /dest/%n"` → correctly expanded |

### 1F: http.rs (~5 tests)

| Test | Asserts |
|------|---------|
| url_to_filename_jpg | "http://example.com/photo.jpg" → ends with ".jpg" |
| url_to_filename_png | "http://example.com/photo.PNG" → ends with ".png" (lowercase) |
| url_to_filename_no_ext | "http://example.com/data" → ends with ".jpg" (default) |
| url_to_filename_deterministic | same URL → same filename |
| url_to_filename_different | different URLs → different filenames |

### 1G: exif.rs (~10 tests)

| Test | Asserts |
|------|---------|
| orientation_1 | exif_orientation_to_rotation(1) → (0.0, false, false) |
| orientation_3 | exif_orientation_to_rotation(3) → (180.0, false, false) |
| orientation_6 | exif_orientation_to_rotation(6) → (90.0, false, false) |
| orientation_8 | exif_orientation_to_rotation(8) → (270.0, false, false) |
| orientation_2_flip | exif_orientation_to_rotation(2) → (0.0, true, false) |
| orientation_unknown | exif_orientation_to_rotation(99) → (0.0, false, false) |
| format_summary_all | ExifInfo with all fields → contains camera, date, etc. |
| format_summary_none | ExifInfo all None → empty string |
| format_summary_partial | Only camera set → one line |
| format_summary_gps | GPS set → contains "GPS:" line |

### 1H: overlay.rs (~5 tests)

| Test | Asserts |
|------|---------|
| format_file_size_bytes | 512 → "512 B" |
| format_file_size_kb | 2048 → "2.0 KB" |
| format_file_size_mb | 3×1024×1024 → "3.0 MB" |
| format_file_size_gb | 2×1024³ → "2.0 GB" |
| format_file_size_zero | 0 → "0 B" |

> **Note**: `format_file_size` is currently private. Either make it `pub(crate)` or place tests inside `#[cfg(test)] mod tests` within overlay.rs.

---

## Phase 2 — FileList Unit Tests (Tier B, ~25 tests)

These test `FileList` navigation/mutation logic using in-memory file lists (no disk needed for most).

### 2A: filelist.rs — navigation

| Test | Asserts |
|------|---------|
| from_single | from_single(file) → len=1, current()=file |
| current_empty | empty list → current() is None |
| next_wraps | 3 files, call next() 3× → wraps to 0 |
| prev_wraps | 3 files at 0, prev() → goes to 2 |
| next_empty | empty list, next() → returns false |
| jump_first | at index 5, jump_first() → index=0 |
| jump_last | 10 files, jump_last() → index=9 |
| jump_forward | 10 files at 0, jump_forward(3) → index=3 |
| jump_forward_wrap | 10 files at 8, jump_forward(5) → index=3 |
| jump_back | 10 files at 5, jump_back(3) → index=2 |
| jump_back_wrap | 10 files at 1, jump_back(3) → index=8 |
| set_current_valid | set_current(5) on 10 files → index=5 |
| set_current_oob | set_current(20) on 10 files → unchanged |
| remove_current | 3 files, remove index 1 → len=2, valid index |
| remove_last | 1 file, remove → returns false (empty) |
| remove_at_end | 3 files at index 2, remove → index adjusts to 1 |

### 2B: filelist.rs — sorting

| Test | Asserts |
|------|---------|
| sort_name_natural | "img2","img10","img1" → "img1","img2","img10" |
| sort_name_case | "Bbb","aaa" → "aaa","Bbb" (case-insensitive) |
| sort_reverse | sorted + reverse=true → reversed order |
| sort_none | sort_by("none") → order unchanged |
| sort_format | mixed extensions → grouped by extension |
| randomize_preserves | randomize() → same length, same elements |

### 2C: filelist.rs — is_image_file

| Test | Asserts |
|------|---------|
| is_image_jpg | "photo.jpg" → true |
| is_image_png_upper | "photo.PNG" → true |
| is_image_txt | "readme.txt" → false |
| is_image_no_ext | "binary" → false |
| is_image_heic | "photo.heic" → true |

### 2D: filelist.rs — file I/O (needs tempfile)

| Test | Asserts |
|------|---------|
| from_filelist | write paths to temp file, load → correct files |
| save_filelist | create list, save, re-read → matches |
| jump_to_match | jump_to("name") → finds by basename |
| jump_to_no_match | jump_to("missing") → index unchanged |

---

## Phase 3 — Win32 Integration Tests (Tier C)

These require a Windows machine. Place in `tests/` directory.

### 3A: tests/image_loader_test.rs

| Test | Asserts |
|------|---------|
| load_png | load a bundled 1x1 PNG → width=1, height=1 |
| load_jpg | load a bundled JPEG → correct dimensions |
| load_invalid | load a text file → returns Err |
| get_dimensions | get_dimensions on PNG → correct w,h |
| save_roundtrip | load PNG → save as PNG → reload → same dimensions |

> Requires test fixture images in `tests/fixtures/`. Generate them programmatically or commit small ones (~100 bytes each).

### 3B: tests/renderer_test.rs

| Test | Asserts |
|------|---------|
| create_renderer | Renderer::new() → Ok |
| multiply_identity | identity × identity → identity |
| multiply_known | known rotation × flip → expected values |

### 3C: tests/window_test.rs

| Test | Asserts |
|------|---------|
| dpi_awareness_set | set_dpi_awareness() doesn't panic |

---

## Phase 4 — CLI Smoke Tests (Tier D)

Add `assert_cmd` and `predicates` as dev-dependencies. Test `fehrust.exe` end-to-end.

### 4A: tests/cli_test.rs

| Test | Asserts |
|------|---------|
| no_args_exits | `fehrust` with no images → exits with error or shows empty |
| help_flag | `fehrust --help` → exit 0, output contains "fehrust" |
| version_flag | `fehrust --version` → exit 0, output contains version |
| list_mode | `fehrust -L fixture.png` → stdout contains filename, dimensions |
| customlist | `fehrust --customlist "%n" fixture.png` → stdout = filename |
| loadable | `fehrust --loadable fixture.png` → stdout = path |
| unloadable | `fehrust --unloadable fixture.txt` → stdout = path |
| invalid_file | `fehrust nonexistent.jpg` → exit with error |

---

## Dev Dependencies to Add

```toml
[dev-dependencies]
tempfile = "3"
assert_cmd = "2"
predicates = "3"
```

---

## Implementation Order

| Phase | Tests | Count | Priority |
|-------|-------|-------|----------|
| **1** | Pure logic unit tests | ~76 | **High** — easy wins, catches regressions |
| **2** | FileList unit tests | ~25 | **High** — core navigation logic |
| **3** | Win32 integration | ~7 | **Medium** — needs fixtures, CI setup |
| **4** | CLI smoke tests | ~8 | **Medium** — end-to-end confidence |
| **Total** | | **~116** | |

## Notes

- **No mocking framework needed** — pure-logic functions are well-separated from Win32 calls
- `format_file_size` and `is_image_file` are private; either make `pub(crate)` or test inside `#[cfg(test)] mod tests` within their modules
- `expand_action` in actions.rs is also private — test inside the module
- `parse_key_name` and `parse_action_name` in keybindings.rs are private — test inside the module
- Test fixture images can be generated at test time using raw bytes (1x1 PNG is 67 bytes, easily embedded as `const`)
- `renderer::multiply_matrix3x2` is a pure function but uses `windows_numerics::Matrix3x2` — still unit-testable
