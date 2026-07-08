use std::sync::atomic::Ordering;
use tauri::{State, AppHandle, Manager, Emitter};
use crate::error::AppError;
use crate::models::{LauncherConfig, LaunchParams, LocalVersionInfo, Account};
use crate::services::{config, launcher, java, version};
use crate::state::AppState;
use crate::utils;

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<LauncherConfig, AppError> {
    Ok(state.config.lock().await.clone())
}

#[tauri::command]
pub async fn set_java_path(state: State<'_, AppState>, path: Option<String>) -> Result<(), AppError> {
    let mut cfg = state.config.lock().await;
    cfg.java_path = path;
    config::save_config(&cfg.clone())
}

#[tauri::command]
pub async fn set_game_directory(state: State<'_, AppState>, directory: String) -> Result<(), AppError> {
    let mut cfg = state.config.lock().await;
    cfg.game_directory = Some(directory);
    config::save_config(&cfg.clone())
}

#[tauri::command]
pub async fn detect_java() -> Result<Vec<String>, AppError> {
    Ok(java::detect_java().await?.into_iter().map(|p| p.to_string_lossy().to_string()).collect())
}

#[tauri::command]
pub async fn set_version_isolation(state: State<'_, AppState>, enabled: bool) -> Result<(), AppError> {
    let mut cfg = state.config.lock().await;
    cfg.version_isolation = Some(enabled);
    config::save_config(&cfg.clone())
}

#[tauri::command]
pub async fn list_local_versions(state: State<'_, AppState>) -> Result<Vec<LocalVersionInfo>, AppError> {
    let cfg = state.config.lock().await;
    version::list_local_versions(&config::get_game_directory(&cfg))
}

#[tauri::command]
pub async fn list_accounts(state: State<'_, AppState>) -> Result<Vec<Account>, AppError> {
    Ok(state.config.lock().await.accounts.clone())
}

#[tauri::command]
pub async fn add_offline_account(state: State<'_, AppState>, player_name: String) -> Result<(), AppError> {
    let mut cfg = state.config.lock().await;
    let is_first = cfg.accounts.is_empty();
    cfg.accounts.push(Account {
        player_name,
        uuid: utils::random_uuid(),
        account_type: "offline".to_string(),
        access_token: None,
        is_selected: is_first,
    });
    config::save_config(&cfg.clone())
}

#[tauri::command]
pub async fn select_account(state: State<'_, AppState>, index: usize) -> Result<(), AppError> {
    let mut cfg = state.config.lock().await;
    if index >= cfg.accounts.len() { return Err(AppError::AccountError("Invalid index".to_string())); }
    for (i, a) in cfg.accounts.iter_mut().enumerate() { a.is_selected = i == index; }
    config::save_config(&cfg.clone())
}

#[tauri::command]
pub async fn remove_account(state: State<'_, AppState>, index: usize) -> Result<(), AppError> {
    let mut cfg = state.config.lock().await;
    if index >= cfg.accounts.len() { return Err(AppError::AccountError("Invalid index".to_string())); }
    let was_sel = cfg.accounts[index].is_selected;
    cfg.accounts.remove(index);
    if was_sel && !cfg.accounts.is_empty() { cfg.accounts[0].is_selected = true; }
    config::save_config(&cfg.clone())
}

#[tauri::command]
pub async fn launch_game(app: AppHandle, state: State<'_, AppState>, params: LaunchParams) -> Result<u32, AppError> {
    {
        let pid = state.running_pid.load(Ordering::SeqCst);
        if pid != 0 { return Err(AppError::LaunchFailed("Game already running".to_string())); }
    }
    let cfg = state.config.lock().await;
    let game_dir = config::get_game_directory(&cfg);
    let isolation = cfg.version_isolation.unwrap_or(true);
    let account = params.account_index.and_then(|i| cfg.accounts.get(i).cloned())
        .or_else(|| cfg.accounts.iter().find(|a| a.is_selected).cloned())
        .ok_or_else(|| AppError::AccountError("No account".to_string()))?;
    drop(cfg);
    let child = launcher::launch_game(app.clone(), &game_dir, &params, &account, isolation).await?;
    let pid = child.id().unwrap_or(0);
    state.running_pid.store(pid, Ordering::SeqCst);
    let mut proc = state.running_process.lock().await;
    *proc = Some(child);
    drop(proc);
    // watch process exit (take child out so we don't hold mutex during wait)
    let app_clone = app.clone();
    tokio::spawn(async move {
        let child = {
            let st: tauri::State<'_, AppState> = app_clone.state();
            let mut p = st.running_process.lock().await;
            let c = p.take();
            drop(p);
            let _ = st.running_pid.store(0, Ordering::SeqCst);
            c
        };
        if let Some(mut c) = child {
            let _ = c.wait().await;
            let _ = app_clone.emit("game-exit", ());
        }
    });
    Ok(pid)
}

#[tauri::command]
pub async fn kill_game(state: State<'_, AppState>) -> Result<(), AppError> {
    let pid = state.running_pid.load(Ordering::SeqCst);
    if pid == 0 { return Ok(()); }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F", "/T"])
            .output()
            .ok();
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new("kill")
            .args(["-9", &pid.to_string()])
            .output()
            .ok();
    }
    Ok(())
}

#[tauri::command]
pub async fn get_launch_command(state: State<'_, AppState>, params: LaunchParams) -> Result<Vec<String>, AppError> {
    let cfg = state.config.lock().await;
    let game_dir = config::get_game_directory(&cfg);
    let isolation = cfg.version_isolation.unwrap_or(true);
    let account = params.account_index.and_then(|i| cfg.accounts.get(i).cloned())
        .or_else(|| cfg.accounts.iter().find(|a| a.is_selected).cloned())
        .ok_or_else(|| AppError::AccountError("No account".to_string()))?;
    launcher::build_launch_command(&game_dir, &params, &account, isolation).await
}