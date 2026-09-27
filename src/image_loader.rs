use std::path::Path;

use windows::{
    Win32::Foundation::{E_FAIL, GENERIC_ACCESS_RIGHTS},
    Win32::Graphics::Imaging::D2D::IWICImagingFactory2,
    Win32::Graphics::Imaging::*,
    Win32::System::Com::*,
    core::*,
};

use std::os::windows::ffi::OsStrExt;

pub struct ImageLoader {
    wic_factory: IWICImagingFactory2,
    _com_guard: ComGuard,
}

pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub wic_bitmap: IWICFormatConverter,
    _pixels: Option<Vec<u8>>,
}

pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

struct ComGuard;

impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

impl ImageLoader {
    pub fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;
            let com_guard = ComGuard;

            let wic_factory: IWICImagingFactory2 =
                CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER)?;

            Ok(Self {
                wic_factory,
                _com_guard: com_guard,
            })
        }
    }

    pub fn load(&self, path: &Path) -> Result<LoadedImage> {
        unsafe {
            let path_wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let path_str = PCWSTR(path_wide.as_ptr());

            let decoder = self.wic_factory.CreateDecoderFromFilename(
                path_str,
                None,
                GENERIC_ACCESS_RIGHTS(0x80000000), // GENERIC_READ
                WICDecodeMetadataCacheOnDemand,
            )?;

            let frame = decoder.GetFrame(0)?;

            let mut width = 0u32;
            let mut height = 0u32;
            frame.GetSize(&mut width, &mut height)?;

            let converter = self.wic_factory.CreateFormatConverter()?;
            converter.Initialize(
                &frame,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeMedianCut,
            )?;

            Ok(LoadedImage {
                width,
                height,
                wic_bitmap: converter,
                _pixels: None,
            })
        }
    }

    /// Decode pixels without returning COM interfaces across thread
    /// boundaries. The returned buffer is safe to send to a worker result.
    pub fn decode_pixels(&self, path: &Path) -> Result<DecodedImage> {
        unsafe {
            let path_wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let decoder = self.wic_factory.CreateDecoderFromFilename(
                PCWSTR(path_wide.as_ptr()),
                None,
                GENERIC_ACCESS_RIGHTS(0x80000000),
                WICDecodeMetadataCacheOnDemand,
            )?;
            let frame = decoder.GetFrame(0)?;
            let mut width = 0u32;
            let mut height = 0u32;
            frame.GetSize(&mut width, &mut height)?;
            let converter = self.wic_factory.CreateFormatConverter()?;
            converter.Initialize(
                &frame,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeMedianCut,
            )?;
            let stride = width
                .checked_mul(4)
                .ok_or_else(|| Error::new(E_FAIL, "Image row is too large"))?;
            let size = stride
                .checked_mul(height)
                .ok_or_else(|| Error::new(E_FAIL, "Image buffer is too large"))?;
            let mut pixels = vec![0u8; size as usize];
            converter.CopyPixels(std::ptr::null(), stride, &mut pixels)?;
            Ok(DecodedImage {
                width,
                height,
                pixels,
            })
        }
    }

    pub fn loaded_from_pixels(&self, decoded: DecodedImage) -> Result<LoadedImage> {
        unsafe {
            let stride = decoded
                .width
                .checked_mul(4)
                .ok_or_else(|| Error::new(E_FAIL, "Image row is too large"))?;
            let bitmap = self.wic_factory.CreateBitmapFromMemory(
                decoded.width,
                decoded.height,
                &GUID_WICPixelFormat32bppPBGRA,
                stride,
                &decoded.pixels,
            )?;
            let converter = self.wic_factory.CreateFormatConverter()?;
            converter.Initialize(
                &bitmap,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeMedianCut,
            )?;
            Ok(LoadedImage {
                width: decoded.width,
                height: decoded.height,
                wic_bitmap: converter,
                _pixels: Some(decoded.pixels),
            })
        }
    }

    /// Get image dimensions without fully decoding
    pub fn get_dimensions(&self, path: &Path) -> Result<(u32, u32)> {
        unsafe {
            let path_wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let decoder = self.wic_factory.CreateDecoderFromFilename(
                PCWSTR(path_wide.as_ptr()),
                None,
                GENERIC_ACCESS_RIGHTS(0x80000000), // GENERIC_READ
                WICDecodeMetadataCacheOnDemand,
            )?;

            let frame = decoder.GetFrame(0)?;
            let mut w = 0u32;
            let mut h = 0u32;
            frame.GetSize(&mut w, &mut h)?;
            Ok((w, h))
        }
    }

    /// Load an image scaled to fit within a thumbnail-sized square.
    pub fn load_thumbnail(&self, path: &Path, max_size: u32) -> Result<LoadedImage> {
        let image = self.load(path)?;
        let scale = (max_size as f32 / image.width as f32)
            .min(max_size as f32 / image.height as f32)
            .min(1.0);
        let width = ((image.width as f32 * scale).round() as u32).max(1);
        let height = ((image.height as f32 * scale).round() as u32).max(1);
        if width == image.width && height == image.height {
            return Ok(image);
        }

        unsafe {
            let scaler = self.wic_factory.CreateBitmapScaler()?;
            scaler.Initialize(
                &image.wic_bitmap,
                width,
                height,
                WICBitmapInterpolationModeFant,
            )?;
            let converter = self.wic_factory.CreateFormatConverter()?;
            converter.Initialize(
                &scaler,
                &GUID_WICPixelFormat32bppPBGRA,
                WICBitmapDitherTypeNone,
                None,
                0.0,
                WICBitmapPaletteTypeMedianCut,
            )?;
            Ok(LoadedImage {
                width,
                height,
                wic_bitmap: converter,
                _pixels: None,
            })
        }
    }

    /// Save image to a file using WIC encoder
    pub fn save(&self, image: &LoadedImage, path: &Path) -> Result<()> {
        unsafe {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
                .to_lowercase();

            let container_format = match ext.as_str() {
                "jpg" | "jpeg" => &GUID_ContainerFormatJpeg,
                "bmp" => &GUID_ContainerFormatBmp,
                "gif" => &GUID_ContainerFormatGif,
                "tiff" | "tif" => &GUID_ContainerFormatTiff,
                _ => &GUID_ContainerFormatPng,
            };

            let stream: IWICStream = self.wic_factory.CreateStream()?;
            let path_wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            stream.InitializeFromFilename(
                PCWSTR(path_wide.as_ptr()),
                GENERIC_ACCESS_RIGHTS(0x40000000).0,
            )?; // GENERIC_WRITE

            let encoder: IWICBitmapEncoder = self
                .wic_factory
                .CreateEncoder(container_format, std::ptr::null())?;
            encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;

            let mut frame: Option<IWICBitmapFrameEncode> = None;
            encoder.CreateNewFrame(&mut frame, std::ptr::null_mut())?;
            let frame = frame.ok_or_else(|| {
                windows::core::Error::new(
                    windows::Win32::Foundation::E_FAIL,
                    "Failed to create encoder frame",
                )
            })?;
            frame.Initialize(None)?;
            frame.SetSize(image.width, image.height)?;

            let mut pixel_format = GUID_WICPixelFormat32bppPBGRA;
            frame.SetPixelFormat(&mut pixel_format)?;

            frame.WriteSource(&image.wic_bitmap, std::ptr::null())?;
            frame.Commit()?;
            encoder.Commit()?;

            Ok(())
        }
    }

    pub fn wic_factory(&self) -> &IWICImagingFactory2 {
        &self.wic_factory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> &'static Path {
        Path::new("tests/fixtures/test_1x1.png")
    }

    #[test]
    fn dimensions_and_thumbnail_loading_preserve_small_images() {
        let loader = ImageLoader::new().unwrap();
        assert_eq!(loader.get_dimensions(fixture()).unwrap(), (1, 1));
        let image = loader.load_thumbnail(fixture(), 120).unwrap();
        assert_eq!((image.width, image.height), (1, 1));
    }

    #[test]
    fn decoded_pixels_can_be_rehydrated_for_rendering() {
        let loader = ImageLoader::new().unwrap();
        let decoded = loader.decode_pixels(fixture()).unwrap();
        assert_eq!((decoded.width, decoded.height), (1, 1));
        assert_eq!(decoded.pixels.len(), 4);
        let image = loader.loaded_from_pixels(decoded).unwrap();
        assert_eq!((image.width, image.height), (1, 1));
    }

    #[test]
    fn missing_images_return_errors_for_all_load_paths() {
        let loader = ImageLoader::new().unwrap();
        let missing = Path::new("tests/fixtures/does-not-exist.png");
        assert!(loader.load(missing).is_err());
        assert!(loader.decode_pixels(missing).is_err());
        assert!(loader.get_dimensions(missing).is_err());
        assert!(loader.load_thumbnail(missing, 64).is_err());
    }

    #[test]
    fn save_uses_supported_output_containers() {
        let loader = ImageLoader::new().unwrap();
        let image = loader.load(fixture()).unwrap();
        let temp = tempfile::tempdir().unwrap();

        for extension in ["png", "jpg", "bmp", "gif", "tiff", "unknown"] {
            let output = temp.path().join(format!("output.{extension}"));
            loader.save(&image, &output).unwrap();
            assert!(output.is_file());
            assert!(std::fs::metadata(output).unwrap().len() > 0);
        }
    }
}
