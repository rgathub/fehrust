use std::path::Path;

use windows::core::*;
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::UI::WindowsAndMessaging::*;

use std::os::windows::ffi::OsStrExt;

#[derive(Debug, Clone, Copy)]
pub enum WallpaperMode {
    Center,
    Fill,
    Fit,
    Stretch,
    Tile,
    Span,
}

/// Set the desktop wallpaper using SystemParametersInfoW (SPI_SETDESKWALLPAPER).
pub fn set_wallpaper(path: &Path, _mode: WallpaperMode) -> Result<()> {
    let abs_path = std::fs::canonicalize(path)
        .map_err(|e| Error::new(E_FAIL, format!("Cannot resolve path: {e}")))?;

    let path_wide: Vec<u16> = abs_path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            Some(path_wide.as_ptr() as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(
                SPIF_UPDATEINIFILE.0 | SPIF_SENDCHANGE.0,
            ),
        )?;
    }

    eprintln!("Wallpaper set to: {}", abs_path.display());
    Ok(())
}
