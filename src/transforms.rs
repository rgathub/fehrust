/// Zoom/pan/rotate transform state helpers
pub const ZOOM_MIN: f64 = 0.002;
pub const ZOOM_MAX: f64 = 2000.0;

/// Calculate the zoom level needed to fit an image within a viewport
pub fn fit_zoom(img_w: f64, img_h: f64, vp_w: f64, vp_h: f64) -> f64 {
    let scale_x = vp_w / img_w;
    let scale_y = vp_h / img_h;
    scale_x.min(scale_y).min(1.0) // don't upscale
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_zoom_smaller_than_viewport() {
        // Image smaller than viewport: should not upscale, returns 1.0
        assert_eq!(fit_zoom(200.0, 100.0, 800.0, 600.0), 1.0);
    }

    #[test]
    fn fit_zoom_larger_than_viewport() {
        // Image larger in both dimensions
        let z = fit_zoom(1600.0, 1200.0, 800.0, 600.0);
        assert!((z - 0.5).abs() < 1e-9);
    }

    #[test]
    fn fit_zoom_exact_fit() {
        // Image exactly matches viewport
        assert_eq!(fit_zoom(800.0, 600.0, 800.0, 600.0), 1.0);
    }

    #[test]
    fn fit_zoom_tall_image() {
        // Tall image constrained by height
        let z = fit_zoom(400.0, 1200.0, 800.0, 600.0);
        assert!((z - 0.5).abs() < 1e-9);
    }

    #[test]
    fn fit_zoom_wide_image() {
        // Wide image constrained by width
        let z = fit_zoom(1600.0, 400.0, 800.0, 600.0);
        assert!((z - 0.5).abs() < 1e-9);
    }

    #[test]
    fn constants_check() {
        assert!((ZOOM_MIN - 0.002).abs() < 1e-9);
        assert!((ZOOM_MAX - 2000.0).abs() < 1e-9);
    }
}
