use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Download an image from a URL and cache it locally.
/// Returns the path to the cached file.
pub fn fetch_image(url: &str) -> Result<PathBuf, String> {
    let cache_dir = std::env::temp_dir().join("fehrust");
    fs::create_dir_all(&cache_dir).map_err(|e| format!("Cannot create cache dir: {e}"))?;

    // Build a safe filename from the URL
    let filename = url_to_filename(url);
    let cached_path = cache_dir.join(&filename);

    // Return cached copy if it exists
    if cached_path.exists() {
        return Ok(cached_path);
    }

    let response = ureq::get(url)
        .call()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let mut body = Vec::new();
    response
        .into_reader()
        .read_to_end(&mut body)
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    let mut file =
        fs::File::create(&cached_path).map_err(|e| format!("Cannot create cache file: {e}"))?;
    file.write_all(&body)
        .map_err(|e| format!("Cannot write cache file: {e}"))?;

    Ok(cached_path)
}

/// Convert a URL to a safe local filename, preserving the extension.
fn url_to_filename(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();

    // Try to extract file extension from the URL
    let ext = url
        .rsplit('/')
        .next()
        .and_then(|segment| {
            let segment = segment.split('?').next().unwrap_or(segment);
            let dot_pos = segment.rfind('.')?;
            let ext = &segment[dot_pos + 1..];
            if ext.len() <= 5 && ext.chars().all(|c| c.is_ascii_alphanumeric()) {
                Some(ext.to_lowercase())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "jpg".to_string());

    format!("{hash:016x}.{ext}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_to_filename_jpg() {
        let name = url_to_filename("http://example.com/photo.jpg");
        assert!(name.ends_with(".jpg"));
    }

    #[test]
    fn url_to_filename_png_lowercase() {
        let name = url_to_filename("http://example.com/photo.PNG");
        assert!(name.ends_with(".png"));
    }

    #[test]
    fn url_to_filename_no_ext() {
        let name = url_to_filename("http://example.com/data");
        assert!(name.ends_with(".jpg"));
    }

    #[test]
    fn url_to_filename_deterministic() {
        let a = url_to_filename("http://example.com/photo.jpg");
        let b = url_to_filename("http://example.com/photo.jpg");
        assert_eq!(a, b);
    }

    #[test]
    fn url_to_filename_different() {
        let a = url_to_filename("http://example.com/a.jpg");
        let b = url_to_filename("http://example.com/b.jpg");
        assert_ne!(a, b);
    }
}
