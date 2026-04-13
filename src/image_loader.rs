use std::path::Path;

use windows::{
    Win32::Foundation::GENERIC_ACCESS_RIGHTS, Win32::Graphics::Imaging::D2D::IWICImagingFactory2,
    Win32::Graphics::Imaging::*, Win32::System::Com::*, core::*,
};

use std::os::windows::ffi::OsStrExt;

pub struct ImageLoader {
    wic_factory: IWICImagingFactory2,
}

pub struct LoadedImage {
    pub width: u32,
    pub height: u32,
    pub wic_bitmap: IWICFormatConverter,
}

impl ImageLoader {
    pub fn new() -> Result<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).ok()?;

            let wic_factory: IWICImagingFactory2 =
                CoCreateInstance(&CLSID_WICImagingFactory2, None, CLSCTX_INPROC_SERVER)?;

            Ok(Self { wic_factory })
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

impl Drop for ImageLoader {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
