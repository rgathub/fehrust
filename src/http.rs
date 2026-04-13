use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

/// Maximum download size (512 MB)
const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;

/// Maximum total cache size (1 GB)
const MAX_CACHE_BYTES: u64 = 1024 * 1024 * 1024;

/// Content-Type prefixes accepted as images
const IMAGE_CONTENT_TYPES: &[&str] = &[
    "image/",
    "application/octet-stream", // some servers serve images as binary
];

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

    // Evict old cache entries before downloading
    evict_cache(&cache_dir);

    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(120))
        .build();
    let response = agent
        .get(url)
        .call()
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    // Validate Content-Type
    let content_type = response.content_type().to_lowercase();
    if !IMAGE_CONTENT_TYPES
        .iter()
        .any(|ct| content_type.starts_with(ct))
    {
        return Err(format!(
            "Response Content-Type '{content_type}' is not an image type"
        ));
    }

    let mut body = Vec::new();
    response
        .into_reader()
        .take(MAX_DOWNLOAD_BYTES)
        .read_to_end(&mut body)
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    if body.len() as u64 >= MAX_DOWNLOAD_BYTES {
        return Err(format!(
            "Download exceeds maximum size of {} MB",
            MAX_DOWNLOAD_BYTES / (1024 * 1024)
        ));
    }

    let mut file =
        fs::File::create(&cached_path).map_err(|e| format!("Cannot create cache file: {e}"))?;
    file.write_all(&body)
        .map_err(|e| format!("Cannot write cache file: {e}"))?;

    Ok(cached_path)
}

/// Evict oldest cache entries when total size exceeds the limit.
fn evict_cache(cache_dir: &std::path::Path) {
    let entries: Vec<_> = match fs::read_dir(cache_dir) {
        Ok(rd) => rd.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };

    let mut files: Vec<(PathBuf, u64, std::time::SystemTime)> = entries
        .iter()
        .filter_map(|e| {
            let meta = e.metadata().ok()?;
            if !meta.is_file() {
                return None;
            }
            let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
            Some((e.path(), meta.len(), mtime))
        })
        .collect();

    let total_size: u64 = files.iter().map(|(_, size, _)| size).sum();
    if total_size <= MAX_CACHE_BYTES {
        return;
    }

    // Sort oldest first
    files.sort_by_key(|(_, _, mtime)| *mtime);

    let mut freed = 0u64;
    let to_free = total_size - MAX_CACHE_BYTES;
    for (path, size, _) in &files {
        if freed >= to_free {
            break;
        }
        if fs::remove_file(path).is_ok() {
            freed += size;
        }
    }
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
