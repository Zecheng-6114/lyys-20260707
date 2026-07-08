use std::path::PathBuf;
use std::process::Command;
use crate::error::AppError;

pub async fn detect_java() -> Result<Vec<PathBuf>, AppError> {
    let mut paths = Vec::new();
    
    // Check JAVA_HOME
    if let Ok(home) = std::env::var("JAVA_HOME") {
        let java = PathBuf::from(home).join("bin").join("java.exe");
        if java.exists() { paths.push(java); }
    }
    
    // Check PATH - 只接受位于 bin 目录下的 java.exe，排除 Oracle javapath stub
    if let Ok(path) = std::env::var("PATH") {
        for p in path.split(';') {
            let java = PathBuf::from(p).join("java.exe");
            if java.exists() && !paths.contains(&java) {
                // 只接受 parent 目录名是 "bin" 的（排除 javapath 等 stub）
                if java.parent().and_then(|d| d.file_name()).map(|n| n.eq_ignore_ascii_case("bin")).unwrap_or(false) {
                    paths.push(java);
                }
            }
        }
    }
    
    // Check common locations
    let common = ["C:\\Program Files\\Java", "C:\\Program Files (x86)\\Java"];
    for base in common {
        if let Ok(entries) = std::fs::read_dir(base) {
            for entry in entries.flatten() {
                let java = entry.path().join("bin").join("java.exe");
                if java.exists() && !paths.contains(&java) { paths.push(java); }
            }
        }
    }
    
    Ok(paths)
}

pub async fn get_java_version(path: &str) -> Result<i32, AppError> {
    let output = Command::new(path).arg("-version").output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let version = stderr.lines().next().unwrap_or("");
    
    // Parse version like "java version "17.0.1" or "openjdk version "17.0.1"
    let v = version.split('"').nth(1).unwrap_or("0");
    let major = v.split('.').next().unwrap_or("0").parse::<i32>().unwrap_or(0);
    
    Ok(if major == 1 { v.split('.').nth(1).unwrap_or("0").parse::<i32>().unwrap_or(0) } else { major })
}