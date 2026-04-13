use std::path::Path;

use windows::Win32::Foundation::E_FAIL;
use windows::Win32::System::Registry::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::*;

use std::os::windows::ffi::OsStrExt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WallpaperMode {
    Center,
    Fill,
    Fit,
    Stretch,
    Tile,
    Span,
}

/// Set the desktop wallpaper using SystemParametersInfoW (SPI_SETDESKWALLPAPER).
pub fn set_wallpaper(path: &Path, mode: WallpaperMode) -> Result<()> {
    let abs_path = std::fs::canonicalize(path)
        .map_err(|e| Error::new(E_FAIL, format!("Cannot resolve path: {e}")))?;

    // Set wallpaper style via registry
    let (style, tile) = match mode {
        WallpaperMode::Center => ("0", "0"),
        WallpaperMode::Stretch => ("2", "0"),
        WallpaperMode::Fit => ("6", "0"),
        WallpaperMode::Fill => ("10", "0"),
        WallpaperMode::Span => ("22", "0"),
        WallpaperMode::Tile => ("0", "1"),
    };
    set_registry_wallpaper_style(style, tile)?;

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
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(SPIF_UPDATEINIFILE.0 | SPIF_SENDCHANGE.0),
        )?;
    }

    eprintln!("Wallpaper set to: {}", abs_path.display());
    Ok(())
}

fn set_registry_wallpaper_style(style: &str, tile: &str) -> Result<()> {
    unsafe {
        let subkey: Vec<u16> = "Control Panel\\Desktop\0".encode_utf16().collect();
        let mut hkey = HKEY::default();
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(subkey.as_ptr()),
            Some(0),
            KEY_SET_VALUE,
            &mut hkey,
        )
        .ok()
        .map_err(|_| Error::new(E_FAIL, "Failed to open Desktop registry key"))?;

        let style_name: Vec<u16> = "WallpaperStyle\0".encode_utf16().collect();
        let style_val: Vec<u8> = style.bytes().chain(std::iter::once(0)).collect();
        RegSetValueExW(
            hkey,
            PCWSTR(style_name.as_ptr()),
            Some(0),
            REG_SZ,
            Some(&style_val),
        )
        .ok()
        .map_err(|_| {
            let _ = RegCloseKey(hkey);
            Error::new(E_FAIL, "Failed to set WallpaperStyle registry value")
        })?;

        let tile_name: Vec<u16> = "TileWallpaper\0".encode_utf16().collect();
        let tile_val: Vec<u8> = tile.bytes().chain(std::iter::once(0)).collect();
        RegSetValueExW(
            hkey,
            PCWSTR(tile_name.as_ptr()),
            Some(0),
            REG_SZ,
            Some(&tile_val),
        )
        .ok()
        .map_err(|_| {
            let _ = RegCloseKey(hkey);
            Error::new(E_FAIL, "Failed to set TileWallpaper registry value")
        })?;

        let _ = RegCloseKey(hkey);
        Ok(())
    }
}
