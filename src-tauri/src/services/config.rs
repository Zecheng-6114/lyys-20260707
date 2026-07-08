use std::path::PathBuf;
use crate::error::AppError;
use crate::models::LauncherConfig;
use crate::utils;

pub fn get_config_path() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    exe.parent().unwrap_or(&PathBuf::from(".")).join("launcher_config.json")
}

pub fn get_game_directory(config: &LauncherConfig) -> PathBuf {
    config.game_directory.as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            home.join(".minecraft")
        })
}

pub fn load_config() -> Result<LauncherConfig, AppError> {
    let path = get_config_path();
    if !path.exists() { return Ok(LauncherConfig::default()); }
    utils::read_json_file(&path)
}

pub fn save_config(config: &LauncherConfig) -> Result<(), AppError> {
    let path = get_config_path();
    utils::write_json_file(&path, config).map_err(AppError::Io)
}