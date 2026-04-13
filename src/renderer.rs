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
        draw_filename: bool,
        filename: &str,
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

                // Apply rotation around image center if needed
                if rotation.abs() > 0.001 {
                    let cx = x + img_w / 2.0;
                    let cy = y + img_h / 2.0;
                    let transform = windows_numerics::Matrix3x2::rotation_around(
                        rotation as f32,
                        windows_numerics::Vector2::new(cx as f32, cy as f32),
                    );
                    rt.SetTransform(&transform);
                }

                let dest_rect = D2D_RECT_F {
                    left: x as f32,
                    top: y as f32,
                    right: (x + img_w) as f32,
                    bottom: (y + img_h) as f32,
                };

                rt.DrawBitmap(
                    bitmap,
                    Some(&dest_rect),
                    1.0,
                    D2D1_BITMAP_INTERPOLATION_MODE_LINEAR,
                    None,
                );

                // Reset transform
                if rotation.abs() > 0.001 {
                    rt.SetTransform(&windows_numerics::Matrix3x2::identity());
                }
            }

            // Draw filename overlay
            if draw_filename && !filename.is_empty() {
                self.draw_text_overlay(rt, filename)?;
            }

            rt.EndDraw(None, None)?;
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
