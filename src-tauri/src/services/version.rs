use std::path::{Path, PathBuf};
use crate::error::AppError;
use crate::models::{VersionInfo, LocalVersionInfo};
use crate::utils;

pub fn get_versions_dir(game_dir: &Path) -> PathBuf { game_dir.join("versions") }
pub fn get_version_dir(game_dir: &Path, name: &str) -> PathBuf { get_versions_dir(game_dir).join(name) }
pub fn get_version_json(game_dir: &Path, name: &str) -> PathBuf { get_version_dir(game_dir, name).join(format!("{}.json", name)) }
pub fn get_version_jar(game_dir: &Path, name: &str) -> PathBuf { get_version_dir(game_dir, name).join(format!("{}.jar", name)) }

pub fn list_local_versions(game_dir: &Path) -> Result<Vec<LocalVersionInfo>, AppError> {
    let dir = get_versions_dir(game_dir);
    if !dir.exists() { return Ok(Vec::new()); }
    let mut versions = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() { continue; }
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let json = get_version_json(game_dir, &name);
        if !json.exists() { continue; }
        let info: Result<VersionInfo, _> = utils::read_json_file(&json);
        let (vtype, inherits) = info.map(|i| (i.version_type, i.inherits_from)).unwrap_or((None, None));
        let has_jar = get_version_jar(game_dir, &name).exists();
        versions.push(LocalVersionInfo { id: name, version_type: vtype, has_jar, inherits_from: inherits });
    }
    Ok(versions)
}

pub fn load_version_info(game_dir: &Path, name: &str) -> Result<VersionInfo, AppError> {
    let json = get_version_json(game_dir, name);
    if !json.exists() { return Err(AppError::VersionNotFound(name.to_string())); }
    let mut info: VersionInfo = utils::read_json_file(&json)?;
    if let Some(ref parent) = info.inherits_from {
        let pjson = get_version_json(game_dir, parent);
        if pjson.exists() { merge(&mut info, &utils::read_json_file(&pjson)?); }
    }
    Ok(info)
}

fn merge(child: &mut VersionInfo, parent: &VersionInfo) {
    // Fabric/Quilt 等模组加载器的 mainClass 应保留
    if child.main_class.is_none() { child.main_class = parent.main_class.clone(); }
    
    // 合并参数：child 的参数在前，parent 的参数在后（这对 Fabric 很重要）
    if child.arguments.is_none() { 
        child.arguments = parent.arguments.clone(); 
    } else if let Some(pa) = &parent.arguments {
        if let Some(ca) = &mut child.arguments {
            // JVM 参数：parent 在后（child 的 Fabric 参数优先）
            if let Some(pj) = &pa.jvm {
                if let Some(cj) = &mut ca.jvm {
                    cj.extend(pj.iter().cloned());
                } else {
                    ca.jvm = Some(pj.clone());
                }
            }
            // Game 参数：parent 在后
            if let Some(pg) = &pa.game {
                if let Some(cg) = &mut ca.game {
                    cg.extend(pg.iter().cloned());
                } else {
                    ca.game = Some(pg.clone());
                }
            }
        }
    }
    
    // 旧版 minecraft_arguments
    if child.minecraft_arguments.is_none() { child.minecraft_arguments = parent.minecraft_arguments.clone(); }
    
    // 库：child 的库在前，parent 的库在后
    if child.libraries.is_none() { 
        child.libraries = parent.libraries.clone(); 
    } else if let Some(pl) = &parent.libraries { 
        if let Some(cl) = &mut child.libraries { 
            cl.extend(pl.iter().cloned()); 
        } 
    }
    
    // 其他元数据
    if child.asset_index.is_none() { child.asset_index = parent.asset_index.clone(); }
    if child.assets.is_none() { child.assets = parent.assets.clone(); }
    if child.java_version.is_none() { child.java_version = parent.java_version.clone(); }
    
    // jar 字段：如果 child 没有指定 jar，使用 parent 的（对于 Fabric 很重要）
    if child.jar.is_none() { child.jar = parent.jar.clone(); }
}