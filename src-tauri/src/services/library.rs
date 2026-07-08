use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs;
use crate::error::AppError;
use crate::models::{LibraryEntry, ResolvedLibrary};
use crate::services::rules;
use crate::utils::Platform;

pub fn resolve(libs: &[LibraryEntry], dir: &Path, feat: &HashMap<String, bool>) -> Result<(Vec<ResolvedLibrary>, Vec<String>), AppError> {
    let mut res = Vec::new(); let mut miss = Vec::new(); let plat = Platform::current();
    for lib in libs {
        if let Some(ref r) = lib.rules { if !rules::evaluate_rules(r, feat) { continue; } }
        let parts: Vec<&str> = lib.name.split(':').collect(); if parts.len() < 3 { continue; }
        let (g, a, v) = (parts[0], parts[1], parts[2]);
        let native = lib.natives.as_ref().and_then(|n| n.get(plat.name()).cloned());
        let gp = g.replace('.', "/");
        let file = if let Some(cls) = &native { format!("{}-{}-{}.jar", a, v, cls) } else { format!("{}-{}.jar", a, v) };
        let path = dir.join(&gp).join(a).join(v).join(&file);
        if path.exists() { res.push(ResolvedLibrary { local_path: path, is_native: native.is_some() }); }
        else if native.is_none() { miss.push(lib.name.clone()); }
    }
    Ok((res, miss))
}

pub fn build_classpath(libs: &[ResolvedLibrary], jar: &Path) -> String {
    let plat = Platform::current();
    let mut p: Vec<String> = libs.iter().filter(|l| !l.is_native).map(|l| l.local_path.to_string_lossy().to_string()).collect();
    p.push(jar.to_string_lossy().to_string());
    p.join(plat.classpath_separator())
}

pub fn get_natives_dir(vdir: &Path) -> PathBuf { vdir.join("natives") }

pub fn extract_natives(libs: &[ResolvedLibrary], ndir: &Path) -> Result<(), AppError> {
    if !ndir.exists() { fs::create_dir_all(ndir)?; }
    for lib in libs.iter().filter(|l| l.is_native) {
        if !lib.local_path.exists() { continue; }
        let f = fs::File::open(&lib.local_path)?;
        let mut ar = zip::ZipArchive::new(f).map_err(|e| AppError::LaunchFailed(format!("Zip error: {}", e)))?;
        for i in 0..ar.len() {
            let mut zf = ar.by_index(i).map_err(|e| AppError::LaunchFailed(format!("Zip entry error: {}", e)))?;
            let name = zf.name();
            if name.starts_with("META-INF/") || name.ends_with('/') { continue; }
            let out = ndir.join(name);
            if out.exists() { continue; }
            if let Some(p) = out.parent() { if !p.exists() { fs::create_dir_all(p)?; } }
            let mut of = fs::File::create(&out)?;
            std::io::copy(&mut zf, &mut of)?;
        }
    }
    Ok(())
}