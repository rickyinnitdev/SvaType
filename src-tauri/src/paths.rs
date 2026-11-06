use crate::error::{AppError, AppResult};
use std::path::{Path, PathBuf};
pub fn resolve(exe: &Path, local: &Path, shared: &Path) -> PathBuf {
    if exe.join("portable.txt").is_file() {
        exe.join("data")
    } else if shared.join("SvaType/shared.txt").is_file() {
        shared.join("SvaType")
    } else {
        local.join("SvaType")
    }
}
pub fn data_dir() -> AppResult<PathBuf> {
    #[cfg(debug_assertions)]
    if let Some(path) = std::env::var_os("SVATYPE_TEST_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    let exe = std::env::current_exe()?;
    let local = std::env::var_os("LOCALAPPDATA")
        .ok_or_else(|| AppError::invalid("missing_local_app_data"))?;
    let shared = std::env::var_os("PROGRAMDATA").unwrap_or_else(|| "C:\\ProgramData".into());
    Ok(resolve(
        exe.parent()
            .ok_or_else(|| AppError::invalid("missing_exe_dir"))?,
        Path::new(&local),
        Path::new(&shared),
    ))
}
