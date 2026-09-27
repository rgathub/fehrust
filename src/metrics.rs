use std::time::Instant;

#[derive(Clone, Copy)]
pub struct PerformanceMetrics {
    enabled: bool,
}

impl PerformanceMetrics {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn enabled(self) -> bool {
        self.enabled
    }

    pub fn log(self, operation: &str, started: Instant, detail: &str) {
        if self.enabled {
            eprintln!("{}", format_event(operation, started.elapsed(), detail));
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
        assert!(PerformanceMetrics::new(true).enabled());
        assert!(!PerformanceMetrics::new(false).enabled());
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
        let metrics = PerformanceMetrics::new(false);
        assert!(!metrics.enabled());
        assert_eq!(
            format_event("frame_render", std::time::Duration::ZERO, "mode=image"),
            "[perf] operation=frame_render duration_ms=0 mode=image"
        );
    }
}
