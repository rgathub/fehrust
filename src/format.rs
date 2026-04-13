use crate::filelist::FehFile;

/// Expand format specifiers in a title/format string.
///
/// Supported specifiers (matching feh):
///   %f — full file path
///   %n — file basename
///   %u — current file index (1-based)
///   %l — total file count
///   %z — zoom level (e.g. "1.00")
///   %w — image width
///   %h — image height
///   %v — program version
///   %a — "playing" or "paused"
///   %% — literal %
#[allow(clippy::too_many_arguments)]
pub fn expand_format(
    fmt: &str,
    file: Option<&FehFile>,
    index: usize,
    total: usize,
    zoom: f64,
    img_width: Option<u32>,
    img_height: Option<u32>,
    paused: bool,
) -> String {
    let mut result = String::with_capacity(fmt.len() * 2);
    let mut chars = fmt.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next() {
                Some('f') => {
                    if let Some(f) = file {
                        result.push_str(&f.path.to_string_lossy());
                    }
                }
                Some('n') => {
                    if let Some(f) = file {
                        result.push_str(&f.name);
                    }
                }
                Some('u') => {
                    result.push_str(&(index + 1).to_string());
                }
                Some('l') => {
                    result.push_str(&total.to_string());
                }
                Some('z') => {
                    result.push_str(&format!("{:.2}", zoom));
                }
                Some('w') => {
                    if let Some(w) = img_width {
                        result.push_str(&w.to_string());
                    }
                }
                Some('h') => {
                    if let Some(h) = img_height {
                        result.push_str(&h.to_string());
                    }
                }
                Some('v') => {
                    result.push_str(env!("CARGO_PKG_VERSION"));
                }
                Some('a') => {
                    result.push_str(if paused { "paused" } else { "playing" });
                }
                Some('s') => {
                    if let Some(f) = file
                        && let Some(size) = f.size
                    {
                        result.push_str(&size.to_string());
                    }
                }
                Some('%') => {
                    result.push('%');
                }
                Some(other) => {
                    result.push('%');
                    result.push(other);
                }
                None => {
                    result.push('%');
                }
            }
        } else if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_file() -> FehFile {
        FehFile {
            path: PathBuf::from("C:\\images\\photo.jpg"),
            name: "photo.jpg".to_string(),
            size: Some(4096),
            mtime: None,
            width: Some(1920),
            height: Some(1080),
        }
    }

    #[test]
    fn expand_f() {
        let f = test_file();
        let result = expand_format("%f", Some(&f), 0, 1, 1.0, None, None, false);
        assert_eq!(result, "C:\\images\\photo.jpg");
    }

    #[test]
    fn expand_n() {
        let f = test_file();
        let result = expand_format("%n", Some(&f), 0, 1, 1.0, None, None, false);
        assert_eq!(result, "photo.jpg");
    }

    #[test]
    fn expand_u() {
        let result = expand_format("%u", None, 4, 10, 1.0, None, None, false);
        assert_eq!(result, "5"); // 1-based
    }

    #[test]
    fn expand_l() {
        let result = expand_format("%l", None, 0, 42, 1.0, None, None, false);
        assert_eq!(result, "42");
    }

    #[test]
    fn expand_z() {
        let result = expand_format("%z", None, 0, 1, 1.5, None, None, false);
        assert_eq!(result, "1.50");
    }

    #[test]
    fn expand_w() {
        let result = expand_format("%w", None, 0, 1, 1.0, Some(800), None, false);
        assert_eq!(result, "800");
    }

    #[test]
    fn expand_h() {
        let result = expand_format("%h", None, 0, 1, 1.0, None, Some(600), false);
        assert_eq!(result, "600");
    }

    #[test]
    fn expand_v() {
        let result = expand_format("%v", None, 0, 1, 1.0, None, None, false);
        assert_eq!(result, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn expand_a_playing() {
        let result = expand_format("%a", None, 0, 1, 1.0, None, None, false);
        assert_eq!(result, "playing");
    }

    #[test]
    fn expand_a_paused() {
        let result = expand_format("%a", None, 0, 1, 1.0, None, None, true);
        assert_eq!(result, "paused");
    }

    #[test]
    fn expand_s() {
        let f = test_file();
        let result = expand_format("%s", Some(&f), 0, 1, 1.0, None, None, false);
        assert_eq!(result, "4096");
    }

    #[test]
    fn expand_literal_percent() {
        let result = expand_format("100%%", None, 0, 1, 1.0, None, None, false);
        assert_eq!(result, "100%");
    }

    #[test]
    fn expand_backslash_n() {
        let result = expand_format("line1\\nline2", None, 0, 1, 1.0, None, None, false);
        assert_eq!(result, "line1\nline2");
    }

    #[test]
    fn expand_unknown() {
        let result = expand_format("%Q", None, 0, 1, 1.0, None, None, false);
        assert_eq!(result, "%Q");
    }

    #[test]
    fn expand_multiple() {
        let f = test_file();
        let result = expand_format("%n [%u of %l]", Some(&f), 2, 10, 1.0, None, None, false);
        assert_eq!(result, "photo.jpg [3 of 10]");
    }

    #[test]
    fn expand_none_file() {
        let result = expand_format("%f %n %s", None, 0, 1, 1.0, None, None, false);
        assert_eq!(result, "  ");
    }

    #[test]
    fn expand_trailing_percent() {
        let result = expand_format("hello%", None, 0, 1, 1.0, None, None, false);
        assert_eq!(result, "hello%");
    }
}
