use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*,
};

use crate::image_loader::LoadedImage;

pub struct Renderer {
    factory: ID2D1Factory,
    render_target: Option<ID2D1HwndRenderTarget>,
    current_bitmap: Option<ID2D1Bitmap>,
}

fn multiply_matrix3x2(
    a: windows_numerics::Matrix3x2,
    b: windows_numerics::Matrix3x2,
) -> windows_numerics::Matrix3x2 {
    windows_numerics::Matrix3x2 {
        M11: a.M11 * b.M11 + a.M12 * b.M21,
        M12: a.M11 * b.M12 + a.M12 * b.M22,
        M21: a.M21 * b.M11 + a.M22 * b.M21,
        M22: a.M21 * b.M12 + a.M22 * b.M22,
        M31: a.M31 * b.M11 + a.M32 * b.M21 + b.M31,
        M32: a.M31 * b.M12 + a.M32 * b.M22 + b.M32,
    }
}

impl Renderer {
    pub fn new() -> Result<Self> {
        unsafe {
            let factory: ID2D1Factory =
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;

            Ok(Self {
                factory,
                render_target: None,
                current_bitmap: None,
            })
        }
    }

    pub fn create_render_target(&mut self, hwnd: HWND, width: u32, height: u32) -> Result<()> {
        unsafe {
            let render_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_DEFAULT,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 0.0,
                dpiY: 0.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let hwnd_props = D2D1_HWND_RENDER_TARGET_PROPERTIES {
                hwnd,
                pixelSize: D2D_SIZE_U { width, height },
                presentOptions: D2D1_PRESENT_OPTIONS_NONE,
            };

            let rt = self
                .factory
                .CreateHwndRenderTarget(&render_props, &hwnd_props)?;

            self.render_target = Some(rt);
            Ok(())
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) -> Result<()> {
        if let Some(rt) = &self.render_target {
            unsafe {
                rt.Resize(&D2D_SIZE_U { width, height })?;
            }
        }
        Ok(())
    }

    pub fn load_bitmap(
        &mut self,
        image: &LoadedImage,
        _wic_factory: &windows::Win32::Graphics::Imaging::D2D::IWICImagingFactory2,
    ) -> Result<()> {
        if let Some(rt) = &self.render_target {
            unsafe {
                let bitmap = rt.CreateBitmapFromWicBitmap(&image.wic_bitmap, None)?;
                self.current_bitmap = Some(bitmap);
            }
        }
        Ok(())
    }

    pub fn render(
        &self,
        zoom: f64,
        pan_x: f64,
        pan_y: f64,
        rotation: f64,
        flip_h: bool,
        flip_v: bool,
        draw_filename: bool,
        filename: &str,
        draw_info: bool,
        info_text: &str,
    ) -> Result<()> {
        let rt = match &self.render_target {
            Some(rt) => rt,
            None => return Ok(()),
        };

        unsafe {
            rt.BeginDraw();

            // Clear to dark gray background
            rt.Clear(Some(&D2D1_COLOR_F {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            }));

            if let Some(bitmap) = &self.current_bitmap {
                let bmp_size = bitmap.GetSize();
                let rt_size = rt.GetSize();

                let img_w = bmp_size.width as f64 * zoom;
                let img_h = bmp_size.height as f64 * zoom;

                // Center image in window + pan offset
                let x = (rt_size.width as f64 - img_w) / 2.0 + pan_x;
                let y = (rt_size.height as f64 - img_h) / 2.0 + pan_y;

                let dest_rect = D2D_RECT_F {
                    left: x as f32,
                    top: y as f32,
                    right: (x + img_w) as f32,
                    bottom: (y + img_h) as f32,
                };

                // Draw checkerboard behind image for transparency
                self.draw_checkerboard(rt, &dest_rect)?;

                // Build combined transform (flip + rotation)
                let needs_transform = flip_h || flip_v || rotation.abs() > 0.001;
                if needs_transform {
                    let cx = x + img_w / 2.0;
                    let cy = y + img_h / 2.0;
                    let mut transform = windows_numerics::Matrix3x2::identity();

                    if flip_h || flip_v {
                        let sx: f32 = if flip_h { -1.0 } else { 1.0 };
                        let sy: f32 = if flip_v { -1.0 } else { 1.0 };
                        let flip = windows_numerics::Matrix3x2 {
                            M11: sx,
                            M12: 0.0,
                            M21: 0.0,
                            M22: sy,
                            M31: if flip_h { 2.0 * cx as f32 } else { 0.0 },
                            M32: if flip_v { 2.0 * cy as f32 } else { 0.0 },
                        };
                        transform = multiply_matrix3x2(transform, flip);
                    }

                    if rotation.abs() > 0.001 {
                        let rot = windows_numerics::Matrix3x2::rotation_around(
                            rotation as f32,
                            windows_numerics::Vector2::new(cx as f32, cy as f32),
                        );
                        transform = multiply_matrix3x2(transform, rot);
                    }

                    rt.SetTransform(&transform);
                }

                rt.DrawBitmap(
                    bitmap,
                    Some(&dest_rect),
                    1.0,
                    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                    None,
                );

                if needs_transform {
                    rt.SetTransform(&windows_numerics::Matrix3x2::identity());
                }
            }

            // Draw filename overlay (bottom bar)
            if draw_filename && !filename.is_empty() {
                self.draw_text_overlay(rt, filename)?;
            }

            // Draw info overlay (top-left box)
            if draw_info && !info_text.is_empty() {
                self.draw_info_overlay(rt, info_text)?;
            }

            rt.EndDraw(None, None)?;
        }

        Ok(())
    }

    fn draw_checkerboard(
        &self,
        rt: &ID2D1HwndRenderTarget,
        rect: &D2D_RECT_F,
    ) -> Result<()> {
        unsafe {
            let light = rt.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.8,
                    g: 0.8,
                    b: 0.8,
                    a: 1.0,
                },
                None,
            )?;
            let dark = rt.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.6,
                    g: 0.6,
                    b: 0.6,
                    a: 1.0,
                },
                None,
            )?;

            let cell_size = 16.0f32;
            let mut col = 0u32;
            let mut cx = rect.left;
            while cx < rect.right {
                let mut row = 0u32;
                let mut cy = rect.top;
                while cy < rect.bottom {
                    let brush = if (row + col) % 2 == 0 { &light } else { &dark };
                    let cell = D2D_RECT_F {
                        left: cx,
                        top: cy,
                        right: (cx + cell_size).min(rect.right),
                        bottom: (cy + cell_size).min(rect.bottom),
                    };
                    rt.FillRectangle(&cell, brush);
                    cy += cell_size;
                    row += 1;
                }
                cx += cell_size;
                col += 1;
            }
        }
        Ok(())
    }

    fn draw_text_overlay(
        &self,
        rt: &ID2D1HwndRenderTarget,
        text: &str,
    ) -> Result<()> {
        unsafe {
            let rt_size = rt.GetSize();

            let bg_brush = rt.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.6,
                },
                None,
            )?;

            let text_brush = rt.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
                None,
            )?;

            let dwrite_factory: windows::Win32::Graphics::DirectWrite::IDWriteFactory =
                windows::Win32::Graphics::DirectWrite::DWriteCreateFactory(
                    windows::Win32::Graphics::DirectWrite::DWRITE_FACTORY_TYPE_SHARED,
                )?;

            let text_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let font_name: Vec<u16> = "Consolas\0".encode_utf16().collect();

            let text_format = dwrite_factory.CreateTextFormat(
                PCWSTR(font_name.as_ptr()),
                None,
                windows::Win32::Graphics::DirectWrite::DWRITE_FONT_WEIGHT_NORMAL,
                windows::Win32::Graphics::DirectWrite::DWRITE_FONT_STYLE_NORMAL,
                windows::Win32::Graphics::DirectWrite::DWRITE_FONT_STRETCH_NORMAL,
                14.0,
                PCWSTR(w!("en-us").as_ptr()),
            )?;

            let bar_height = 24.0f32;
            let bar_y = rt_size.height - bar_height;

            let bg_rect = D2D_RECT_F {
                left: 0.0,
                top: bar_y,
                right: rt_size.width,
                bottom: rt_size.height,
            };
            rt.FillRectangle(&bg_rect, &bg_brush);

            let text_rect = D2D_RECT_F {
                left: 6.0,
                top: bar_y + 3.0,
                right: rt_size.width - 6.0,
                bottom: rt_size.height,
            };
            rt.DrawText(
                &text_wide[..text_wide.len() - 1],
                &text_format,
                &text_rect,
                &text_brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
            );

            Ok(())
        }
    }

    fn draw_info_overlay(
        &self,
        rt: &ID2D1HwndRenderTarget,
        text: &str,
    ) -> Result<()> {
        unsafe {
            let bg_brush = rt.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.7,
                },
                None,
            )?;

            let text_brush = rt.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                },
                None,
            )?;

            let dwrite_factory: windows::Win32::Graphics::DirectWrite::IDWriteFactory =
                windows::Win32::Graphics::DirectWrite::DWriteCreateFactory(
                    windows::Win32::Graphics::DirectWrite::DWRITE_FACTORY_TYPE_SHARED,
                )?;

            let font_name: Vec<u16> = "Consolas\0".encode_utf16().collect();
            let text_format = dwrite_factory.CreateTextFormat(
                PCWSTR(font_name.as_ptr()),
                None,
                windows::Win32::Graphics::DirectWrite::DWRITE_FONT_WEIGHT_NORMAL,
                windows::Win32::Graphics::DirectWrite::DWRITE_FONT_STYLE_NORMAL,
                windows::Win32::Graphics::DirectWrite::DWRITE_FONT_STRETCH_NORMAL,
                13.0,
                PCWSTR(w!("en-us").as_ptr()),
            )?;

            let text_wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let rt_size = rt.GetSize();
            let max_width = (rt_size.width * 0.5).max(300.0);

            let text_layout = dwrite_factory.CreateTextLayout(
                &text_wide[..text_wide.len() - 1],
                &text_format,
                max_width,
                rt_size.height,
            )?;

            let mut metrics =
                windows::Win32::Graphics::DirectWrite::DWRITE_TEXT_METRICS::default();
            text_layout.GetMetrics(&mut metrics)?;

            let padding = 8.0f32;
            let box_width = metrics.width + padding * 2.0;
            let box_height = metrics.height + padding * 2.0;

            let bg_rect = D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: box_width,
                bottom: box_height,
            };
            rt.FillRectangle(&bg_rect, &bg_brush);

            let text_rect = D2D_RECT_F {
                left: padding,
                top: padding,
                right: box_width - padding,
                bottom: box_height,
            };
            rt.DrawText(
                &text_wide[..text_wide.len() - 1],
                &text_format,
                &text_rect,
                &text_brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                windows::Win32::Graphics::DirectWrite::DWRITE_MEASURING_MODE_NATURAL,
            );

            Ok(())
        }
    }

    pub fn has_bitmap(&self) -> bool {
        self.current_bitmap.is_some()
    }

    pub fn bitmap_size(&self) -> Option<(f32, f32)> {
        self.current_bitmap.as_ref().map(|b| {
            let size = unsafe { b.GetSize() };
            (size.width, size.height)
        })
    }
}
