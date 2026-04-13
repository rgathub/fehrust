use std::collections::HashMap;

use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::core::*;

use crate::filelist::FileList;
use crate::image_loader::ImageLoader;

const THUMB_SIZE: f32 = 120.0;
const THUMB_PADDING: f32 = 10.0;
const LABEL_HEIGHT: f32 = 18.0;
const INDEX_THUMB_SIZE: f32 = 80.0;
const INDEX_PADDING: f32 = 6.0;

/// Cell dimensions for a thumbnail grid
struct CellMetrics {
    thumb_size: f32,
    padding: f32,
    cell_w: f32,
    cell_h: f32,
}

impl CellMetrics {
    fn normal() -> Self {
        let thumb_size = THUMB_SIZE;
        let padding = THUMB_PADDING;
        Self {
            thumb_size,
            padding,
            cell_w: thumb_size + padding * 2.0,
            cell_h: thumb_size + padding * 2.0 + LABEL_HEIGHT,
        }
    }

    fn index() -> Self {
        let thumb_size = INDEX_THUMB_SIZE;
        let padding = INDEX_PADDING;
        Self {
            thumb_size,
            padding,
            cell_w: thumb_size + padding * 2.0,
            cell_h: thumb_size + padding * 2.0 + LABEL_HEIGHT,
        }
    }
}

pub struct ThumbnailView {
    /// Cached bitmaps keyed by file index
    cache: HashMap<usize, ID2D1Bitmap>,
    /// Scroll offset in pixels (positive = scrolled down)
    pub scroll_y: f32,
    /// Whether this is an index/contact sheet view (smaller cells)
    pub index_mode: bool,
    /// Currently selected thumbnail index (for keyboard nav)
    pub selected: usize,
}

impl ThumbnailView {
    pub fn new(index_mode: bool) -> Self {
        Self {
            cache: HashMap::new(),
            scroll_y: 0.0,
            index_mode,
            selected: 0,
        }
    }

    fn metrics(&self) -> CellMetrics {
        if self.index_mode {
            CellMetrics::index()
        } else {
            CellMetrics::normal()
        }
    }

    /// Calculate the number of columns that fit in the given viewport width
    fn cols(&self, viewport_w: f32) -> usize {
        let m = self.metrics();
        (viewport_w / m.cell_w).floor().max(1.0) as usize
    }

    /// Public accessor for column count (used by input handling)
    pub fn cols_for(&self, viewport_w: f32) -> usize {
        self.cols(viewport_w)
    }

    /// Total content height for scrolling
    pub fn total_height(&self, file_count: usize, viewport_w: f32) -> f32 {
        let cols = self.cols(viewport_w);
        let rows = (file_count + cols - 1) / cols;
        let m = self.metrics();
        rows as f32 * m.cell_h
    }

    /// Render the thumbnail grid
    pub fn render(
        &mut self,
        rt: &ID2D1HwndRenderTarget,
        filelist: &FileList,
        image_loader: &ImageLoader,
    ) -> Result<()> {
        unsafe {
            rt.BeginDraw();
            rt.Clear(Some(&D2D1_COLOR_F {
                r: 0.12,
                g: 0.12,
                b: 0.12,
                a: 1.0,
            }));

            let rt_size = rt.GetSize();
            let m = self.metrics();
            let cols = self.cols(rt_size.width);

            let bg_brush = rt.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.2,
                    g: 0.2,
                    b: 0.2,
                    a: 1.0,
                },
                None,
            )?;
            let sel_brush = rt.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.3,
                    g: 0.5,
                    b: 0.9,
                    a: 1.0,
                },
                None,
            )?;
            let text_brush = rt.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                },
                None,
            )?;

            // Create DirectWrite resources for labels
            let dwrite_factory: windows::Win32::Graphics::DirectWrite::IDWriteFactory =
                windows::Win32::Graphics::DirectWrite::DWriteCreateFactory(
                    windows::Win32::Graphics::DirectWrite::DWRITE_FACTORY_TYPE_SHARED,
                )?;
            let font_name: Vec<u16> = "Segoe UI\0".encode_utf16().collect();
            let text_format = dwrite_factory.CreateTextFormat(
                PCWSTR(font_name.as_ptr()),
                None,
                windows::Win32::Graphics::DirectWrite::DWRITE_FONT_WEIGHT_NORMAL,
                windows::Win32::Graphics::DirectWrite::DWRITE_FONT_STYLE_NORMAL,
                windows::Win32::Graphics::DirectWrite::DWRITE_FONT_STRETCH_NORMAL,
                10.0,
                PCWSTR(w!("en-us").as_ptr()),
            )?;
            text_format.SetTextAlignment(
                windows::Win32::Graphics::DirectWrite::DWRITE_TEXT_ALIGNMENT_CENTER,
            )?;

            let file_count = filelist.len();
            for idx in 0..file_count {
                let col = idx % cols;
                let row = idx / cols;

                let cx = col as f32 * m.cell_w;
                let cy = row as f32 * m.cell_h - self.scroll_y;

                // Skip if entirely offscreen
                if cy + m.cell_h < 0.0 || cy > rt_size.height {
                    continue;
                }

                let thumb_x = cx + m.padding;
                let thumb_y = cy + m.padding;

                // Draw cell background
                let cell_rect = D2D_RECT_F {
                    left: cx + 2.0,
                    top: cy + 2.0,
                    right: cx + m.cell_w - 2.0,
                    bottom: cy + m.cell_h - 2.0,
                };

                if idx == self.selected {
                    rt.FillRectangle(&cell_rect, &sel_brush);
                } else {
                    rt.FillRectangle(&cell_rect, &bg_brush);
                }

                // Load thumbnail bitmap on demand
                if !self.cache.contains_key(&idx) {
                    if let Some(file) = filelist.file_at(idx) {
                        if let Ok(img) = image_loader.load(&file.path) {
                            if let Ok(bmp) = rt.CreateBitmapFromWicBitmap(&img.wic_bitmap, None) {
                                self.cache.insert(idx, bmp);
                            }
                        }
                    }
                }

                // Draw the thumbnail
                if let Some(bmp) = self.cache.get(&idx) {
                    let bmp_size = bmp.GetSize();
                    let scale = (m.thumb_size / bmp_size.width).min(m.thumb_size / bmp_size.height);
                    let draw_w = bmp_size.width * scale;
                    let draw_h = bmp_size.height * scale;
                    let ox = thumb_x + (m.thumb_size - draw_w) / 2.0;
                    let oy = thumb_y + (m.thumb_size - draw_h) / 2.0;

                    let dest = D2D_RECT_F {
                        left: ox,
                        top: oy,
                        right: ox + draw_w,
                        bottom: oy + draw_h,
                    };
                    rt.DrawBitmap(
                        bmp,
                        Some(&dest),
                        1.0,
                        D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                        None,
                    );
                }

                // Draw filename label
                if let Some(file) = filelist.file_at(idx) {
                    let label = truncate_name(&file.name, 16);
                    let label_wide: Vec<u16> =
                        label.encode_utf16().chain(std::iter::once(0)).collect();
                    let label_rect = D2D_RECT_F {
                        left: cx,
                        top: cy + m.thumb_size + m.padding * 2.0 - 2.0,
                        right: cx + m.cell_w,
                        bottom: cy + m.cell_h,
                    };
                    rt.DrawText(
                        &label_wide[..label_wide.len() - 1],
                        &text_format,
                        &label_rect,
                        &text_brush,
                        D2D1_DRAW_TEXT_OPTIONS_NONE,
                        windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
                    );
                }
            }

            rt.EndDraw(None, None)?;
        }
        Ok(())
    }

    /// Determine which thumbnail was clicked. Returns file index.
    pub fn handle_click(&self, x: f32, y: f32, viewport_w: f32) -> Option<usize> {
        let m = self.metrics();
        let cols = self.cols(viewport_w);
        let abs_y = y + self.scroll_y;

        let col = (x / m.cell_w) as usize;
        let row = (abs_y / m.cell_h) as usize;

        if col >= cols {
            return None;
        }

        let idx = row * cols + col;
        Some(idx)
    }

    /// Scroll by a delta amount (positive = scroll down)
    pub fn scroll(&mut self, delta: f32, file_count: usize, viewport_w: f32, viewport_h: f32) {
        self.scroll_y += delta;
        let max_scroll = (self.total_height(file_count, viewport_w) - viewport_h).max(0.0);
        self.scroll_y = self.scroll_y.clamp(0.0, max_scroll);
    }
}

fn truncate_name(name: &str, max_len: usize) -> String {
    if name.len() <= max_len {
        name.to_string()
    } else {
        format!("{}…", &name[..max_len - 1])
    }
}
