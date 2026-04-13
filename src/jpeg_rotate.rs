use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::{E_FAIL, GENERIC_ACCESS_RIGHTS};
use windows::Win32::Graphics::Imaging::*;
use windows::core::*;

use crate::image_loader::ImageLoader;

/// Rotation direction for lossless JPEG save
#[derive(Debug, Clone, Copy)]
pub enum RotateDirection {
    Clockwise90,
    CounterClockwise90,
}

impl RotateDirection {
    pub fn to_wic_transform(self) -> WICBitmapTransformOptions {
        match self {
            RotateDirection::Clockwise90 => WICBitmapTransformRotate90,
            RotateDirection::CounterClockwise90 => WICBitmapTransformRotate270,
        }
    }
}

impl ImageLoader {
    /// Save a rotated copy of an image using WIC.
    /// For PNG/BMP this is lossless; for JPEG it re-encodes (acceptable trade-off).
    pub fn save_rotated(&self, path: &Path, transform: WICBitmapTransformOptions) -> Result<()> {
        unsafe {
            let path_wide: Vec<u16> = path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // Decode the source image
            let decoder = self.wic_factory().CreateDecoderFromFilename(
                PCWSTR(path_wide.as_ptr()),
                None,
                GENERIC_ACCESS_RIGHTS(0x80000000), // GENERIC_READ
                WICDecodeMetadataCacheOnDemand,
            )?;
            let frame = decoder.GetFrame(0)?;

            // Apply flip/rotate transform
            let rotator: IWICBitmapFlipRotator = self.wic_factory().CreateBitmapFlipRotator()?;
            rotator.Initialize(&frame, transform)?;

            // Get container format from original decoder
            let container_format = decoder.GetContainerFormat()?;

            // Write to a temp file, then replace original
            let tmp_path = path.with_extension("fehrust_tmp");
            let tmp_wide: Vec<u16> = tmp_path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let encoder: IWICBitmapEncoder = self
                .wic_factory()
                .CreateEncoder(&container_format, std::ptr::null())?;

            let stream: IWICStream = self.wic_factory().CreateStream()?;
            stream.InitializeFromFilename(PCWSTR(tmp_wide.as_ptr()), 0x40000000u32)?; // GENERIC_WRITE
            encoder.Initialize(&stream, WICBitmapEncoderNoCache)?;

            let frame_encode: IWICBitmapFrameEncode = {
                let mut fe: Option<IWICBitmapFrameEncode> = None;
                encoder.CreateNewFrame(&mut fe, std::ptr::null_mut())?;
                fe.unwrap()
            };
            frame_encode.Initialize(None)?;

            let mut w = 0u32;
            let mut h = 0u32;
            rotator.GetSize(&mut w, &mut h)?;
            frame_encode.SetSize(w, h)?;

            let mut pixel_format = rotator.GetPixelFormat()?;
            frame_encode.SetPixelFormat(&mut pixel_format)?;

            frame_encode.WriteSource(&rotator, std::ptr::null())?;
            frame_encode.Commit()?;
            encoder.Commit()?;

            // Replace original with rotated version
            drop(stream);
            drop(encoder);
            std::fs::rename(&tmp_path, path)
                .map_err(|e| Error::new(E_FAIL, format!("Failed to replace file: {e}")))?;

            Ok(())
        }
    }
}
