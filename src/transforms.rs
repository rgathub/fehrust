/// Zoom/pan/rotate transform state helpers

pub const ZOOM_MIN: f64 = 0.002;
pub const ZOOM_MAX: f64 = 2000.0;

/// Calculate the zoom level needed to fit an image within a viewport
pub fn fit_zoom(img_w: f64, img_h: f64, vp_w: f64, vp_h: f64) -> f64 {
    let scale_x = vp_w / img_w;
    let scale_y = vp_h / img_h;
    scale_x.min(scale_y).min(1.0) // don't upscale
}

/// Calculate zoom to fill the viewport (may crop)
pub fn fill_zoom(img_w: f64, img_h: f64, vp_w: f64, vp_h: f64) -> f64 {
    let scale_x = vp_w / img_w;
    let scale_y = vp_h / img_h;
    scale_x.max(scale_y)
}
