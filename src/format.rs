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
