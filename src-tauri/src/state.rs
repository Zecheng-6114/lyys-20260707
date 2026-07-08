use crate::models::LauncherConfig;
use tokio::sync::Mutex;
use tokio::process::Child;
use std::sync::atomic::AtomicU32;

pub struct AppState {
    pub config: Mutex<LauncherConfig>,
    pub running_process: Mutex<Option<Child>>,
    pub running_pid: AtomicU32,
}

impl AppState {
    pub fn new(config: LauncherConfig) -> Self {
        Self { config: Mutex::new(config), running_process: Mutex::new(None), running_pid: AtomicU32::new(0) }
    }
}