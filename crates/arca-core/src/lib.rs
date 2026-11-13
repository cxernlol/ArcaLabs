pub mod config;
pub mod crash;
pub mod keychain;
pub mod pipeline;
pub mod scenes;

use std::path::PathBuf;

pub fn config_dir() -> Result<PathBuf, String> {
    #[cfg(windows)]
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return Ok(PathBuf::from(appdata).join("com.arcalabs.desktop"));
    }
    let dirs = directories::ProjectDirs::from("com", "ArcaLabs", "ArcaLabs")
        .ok_or_else(|| "User config folder is unavailable".to_owned())?;
    Ok(dirs.config_dir().to_owned())
}

pub(crate) fn replace_file(
    temp: &std::path::Path,
    destination: &std::path::Path,
) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::Storage::FileSystem::{
            MOVE_FILE_FLAGS, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH, MoveFileExW,
        };
        let temp: Vec<u16> = temp.as_os_str().encode_wide().chain(Some(0)).collect();
        let destination: Vec<u16> = destination
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        unsafe {
            MoveFileExW(
                windows::core::PCWSTR(temp.as_ptr()),
                windows::core::PCWSTR(destination.as_ptr()),
                MOVE_FILE_FLAGS(MOVEFILE_REPLACE_EXISTING.0 | MOVEFILE_WRITE_THROUGH.0),
            )
        }
        .map_err(|error| format!("Could not replace config file safely: {error}"))
    }
    #[cfg(not(windows))]
    {
        std::fs::rename(temp, destination)
            .map_err(|error| format!("Could not replace config file safely: {error}"))
    }
}
