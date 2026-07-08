use serde::Serialize;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Version not found: {0}")]
    VersionNotFound(String),
    #[error("Java not found: {0}")]
    JavaNotFound(String),
    #[error("Library missing: {0}")]
    LibraryMissing(String),
    #[error("JAR not found: {0}")]
    JarNotFound(String),
    #[error("Account error: {0}")]
    AccountError(String),
    #[error("Launch failed: {0}")]
    LaunchFailed(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}