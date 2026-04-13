use std::path::Path;

use windows::{
    core::*,
    Win32::Foundation::GENERIC_READ,
    Win32::Graphics::Imaging::*,
    Win32::Graphics::Imaging::D2D::IWICImagingFactory2,
    Win32::System::Com::*,
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
                GENERIC_READ,
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

    pub fn wic_factory(&self) -> &IWICImagingFactory2 {
        &self.wic_factory
    }
}
