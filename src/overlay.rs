use crate::app::AppState;
use crate::config::ViewMode;

pub fn build_info_string(state: &AppState) -> String {
    let mut lines = Vec::new();

    // Mode indicator
    match state.mode {
        ViewMode::Zoom => lines.push("Mode: ZOOM (drag up/down)".to_string()),
        ViewMode::Rotate => lines.push("Mode: ROTATE (drag left/right)".to_string()),
        ViewMode::Pan => lines.push("Mode: PAN".to_string()),
        ViewMode::Normal => {}
    }

    // Filename
    if let Some(file) = state.filelist.current() {
        lines.push(format!("File: {}", file.path.display()));
    }

    // Image dimensions
    if let Some(ref img) = state.current_image {
        lines.push(format!("Dimensions: {}x{}", img.width, img.height));
    }

    // Zoom
    lines.push(format!("Zoom: {:.0}%", state.zoom * 100.0));

    // File index
    lines.push(format!(
        "Image: {} of {}",
        state.filelist.current_index() + 1,
        state.filelist.len()
    ));

    // File size
    if let Some(file) = state.filelist.current()
        && let Ok(meta) = std::fs::metadata(&file.path)
    {
        lines.push(format!("Size: {}", format_file_size(meta.len())));
    }

    // EXIF summary
    if let Some(ref exif) = state.current_exif {
        let summary = exif.format_summary();
        if !summary.is_empty() {
            lines.push(String::new());
            lines.push(summary);
        }
    }

    // Caption
    if let Some(ref caption) = state.current_caption {
        lines.push(String::new());
        lines.push(format!("Caption: {}", caption));
    }

    lines.join("\n")
}

fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_file_size_bytes() {
        assert_eq!(format_file_size(512), "512 B");
    }

    #[test]
    fn format_file_size_kb() {
        assert_eq!(format_file_size(2048), "2.0 KB");
    }

    #[test]
    fn format_file_size_mb() {
        assert_eq!(format_file_size(3 * 1024 * 1024), "3.0 MB");
    }

    #[test]
    fn format_file_size_gb() {
        assert_eq!(format_file_size(2 * 1024 * 1024 * 1024), "2.0 GB");
    }

    #[test]
    fn format_file_size_zero() {
        assert_eq!(format_file_size(0), "0 B");
    }
}
