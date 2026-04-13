use crate::app::AppState;

pub fn build_info_string(state: &AppState) -> String {
    let mut lines = Vec::new();

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
    if let Some(file) = state.filelist.current() {
        if let Ok(meta) = std::fs::metadata(&file.path) {
            lines.push(format!("Size: {}", format_file_size(meta.len())));
        }
    }

    // EXIF summary
    if let Some(ref exif) = state.current_exif {
        let summary = exif.format_summary();
        if !summary.is_empty() {
            lines.push(String::new());
            lines.push(summary);
        }
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
