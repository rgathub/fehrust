use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{Arc, Mutex},
    time::Instant,
};

const DEFAULT_LOG_PATH: &str = "perf.log";

#[derive(Clone)]
pub struct PerformanceMetrics {
    enabled: bool,
    file: Option<Arc<Mutex<File>>>,
}

impl PerformanceMetrics {
    pub fn new(enabled: bool) -> std::io::Result<Self> {
        Self::with_path(enabled, DEFAULT_LOG_PATH)
    }

    fn with_path<P: AsRef<std::path::Path>>(enabled: bool, path: P) -> std::io::Result<Self> {
        let file = if enabled {
            Some(Arc::new(Mutex::new(
                OpenOptions::new().create(true).append(true).open(path)?,
            )))
        } else {
            None
        };
        Ok(Self { enabled, file })
    }

    pub fn log(&self, operation: &str, started: Instant, detail: &str) {
        if self.enabled {
            let event = format_event(operation, started.elapsed(), detail);
            eprintln!("{event}");
            if let Some(file) = &self.file
                && let Ok(mut file) = file.lock()
            {
                if let Err(error) = writeln!(file, "{event}") {
                    eprintln!("[perf] failed to write {DEFAULT_LOG_PATH}: {error}");
                } else if let Err(error) = file.flush() {
                    eprintln!("[perf] failed to flush {DEFAULT_LOG_PATH}: {error}");
                }
            } else if self.file.is_some() {
                eprintln!("[perf] failed to lock {DEFAULT_LOG_PATH}");
            }
        }
    }
}

fn format_event(operation: &str, duration: std::time::Duration, detail: &str) -> String {
    format!(
        "[perf] operation={operation} duration_ms={} {detail}",
        duration.as_secs_f64() * 1000.0
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_preserve_enabled_state() {
        assert!(
            PerformanceMetrics {
                enabled: true,
                file: None
            }
            .enabled
        );
        assert!(!PerformanceMetrics::new(false).unwrap().enabled);
    }

    #[test]
    fn event_contains_operation_duration_and_detail() {
        let event = format_event(
            "image_decode",
            std::time::Duration::from_millis(125),
            "path=photo.jpg",
        );
        assert_eq!(
            event,
            "[perf] operation=image_decode duration_ms=125 path=photo.jpg"
        );
    }

    #[test]
    fn disabled_metrics_do_not_change_event_formatting_inputs() {
        let metrics = PerformanceMetrics::new(false).unwrap();
        assert!(!metrics.enabled);
        assert_eq!(
            format_event("frame_render", std::time::Duration::ZERO, "mode=image"),
            "[perf] operation=frame_render duration_ms=0 mode=image"
        );
    }

    #[test]
    fn enabled_metrics_append_events_to_file() {
        let temp = tempfile::NamedTempFile::new().unwrap();
        let metrics = PerformanceMetrics::with_path(true, temp.path()).unwrap();
        metrics.log("frame_render", Instant::now(), "mode=image");
        let contents = std::fs::read_to_string(temp.path()).unwrap();
        assert!(contents.contains("[perf] operation=frame_render"));
        assert!(contents.contains("mode=image"));
    }
}
