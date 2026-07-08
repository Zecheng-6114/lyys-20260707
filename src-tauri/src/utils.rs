use std::path::Path;
use std::fs;

pub struct Platform {
    name: String,
    arch: String,
    classpath_separator: String,
}

impl Platform {
    pub fn current() -> Self {
        let name = if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "macos" } else { "linux" };
        let arch = if cfg!(target_arch = "x86_64") { "x86" } else { "x86" };
        let sep = if cfg!(windows) { ";" } else { ":" };
        Self { name: name.to_string(), arch: arch.to_string(), classpath_separator: sep.to_string() }
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn arch(&self) -> &str { &self.arch }
    pub fn classpath_separator(&self) -> &str { &self.classpath_separator }
}

pub fn random_uuid() -> String {
    uuid::Uuid::new_v4().to_string().replace('-', "")
}

pub fn ensure_dir(path: &Path) -> std::io::Result<()> {
    if !path.exists() { fs::create_dir_all(path)?; }
    Ok(())
}

pub fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, crate::error::AppError> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(crate::error::AppError::Json)
}

pub fn write_json_file<T: serde::Serialize>(path: &Path, data: &T) -> std::io::Result<()> {
    ensure_dir(path.parent().unwrap())?;
    let content = serde_json::to_string_pretty(data)?;
    fs::write(path, content)
}