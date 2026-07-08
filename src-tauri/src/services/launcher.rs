use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Command, Child};
use tauri::AppHandle;
use tauri::Emitter;
use crate::error::AppError;
use crate::models::{VersionInfo, LaunchParams, Account, ArgumentEntry, ArgumentValue};
use crate::services::{version, library, java, rules};
use crate::utils::Platform;

pub async fn build_launch_command(root: &Path, params: &LaunchParams, account: &Account, isolation: bool) -> Result<Vec<String>, AppError> {
    let info = version::load_version_info(root, &params.version_name)?;
    let java = params.java_path.clone().unwrap_or_else(|| "java".to_string());
    let jver = java::get_java_version(&java).await?;
    if let Some(ref req) = info.java_version { if let Some(maj) = req.major_version { if jver < maj as i32 { return Err(AppError::JavaNotFound(format!("Java {} required, found {}", maj, jver))); } } }
    
    let vdir = version::get_version_dir(root, &params.version_name);
    
    // Fabric/Quilt: jar 字段指向 loader jar(存于 libraries/)，真正需要的游戏 jar 是父版本(vanilla)
    // inheritsFrom 优先级最高，其 jar 才是 Minecraft 本体 jar
    let jar = if let Some(ref parent_name) = info.inherits_from {
        let pj = version::get_version_jar(root, parent_name);
        if pj.exists() { pj } else { version::get_version_jar(root, &params.version_name) }
    } else if let Some(ref jar_name) = info.jar {
        let j = version::get_version_jar(root, jar_name);
        if j.exists() { j } else { version::get_version_jar(root, &params.version_name) }
    } else {
        version::get_version_jar(root, &params.version_name)
    };
    
    if !jar.exists() { return Err(AppError::JarNotFound(jar.to_string_lossy().to_string())); }
    
    let libs_dir = root.join("libraries");
    let assets = root.join("assets");
    let natives = library::get_natives_dir(&vdir);
    // 版本隔离：game_directory 指向版本目录(saves/config 等隔离)，但 cwd 始终是游戏根目录
    let run_dir = if isolation { vdir } else { root.to_path_buf() };
    
    let features = HashMap::from([("is_demo_user".to_string(), false), ("has_custom_resolution".to_string(), true)]);
    let libs = if let Some(ref l) = info.libraries { let (r, m) = library::resolve(l, &libs_dir, &features)?; if !m.is_empty() { return Err(AppError::LibraryMissing(m.join(", "))); } r } else { Vec::new() };
    library::extract_natives(&libs, &natives)?;
    let cp = library::build_classpath(&libs, &jar);
    let main = info.main_class.clone().unwrap_or_else(|| "net.minecraft.client.main.Main".to_string());
    
    let mut jvm = vec![format!("-Xmx{}m", params.max_memory_mb.unwrap_or(2048)), format!("-Xms{}m", params.min_memory_mb.unwrap_or(128)), format!("-Djava.library.path={}", natives.to_string_lossy()), "-cp".to_string(), cp.clone(), "-Dminecraft.launcher.brand=custom".to_string(), "-Dminecraft.launcher.version=1.0".to_string()];
    let mut game = Vec::new();
    
    if let Some(ref args) = info.arguments {
        if let Some(ref j) = args.jvm { for a in j { add_arg(a, &mut jvm, &info, &run_dir, root, &assets, account, params, &cp, &natives, &features); } }
        if let Some(ref g) = args.game { for a in g { add_arg(a, &mut game, &info, &run_dir, root, &assets, account, params, &cp, &natives, &features); } }
    } else if let Some(ref ma) = info.minecraft_arguments { for a in ma.split_whitespace() { game.push(repl(a, &info, &run_dir, root, &assets, account, params, &cp, &natives)); } }
    
    let mut cmd = vec![java];
    cmd.extend(jvm); cmd.push(main); cmd.extend(game);
    Ok(cmd)
}

fn add_arg(arg: &ArgumentEntry, out: &mut Vec<String>, info: &VersionInfo, run: &Path, root: &Path, assets: &Path, acc: &Account, p: &LaunchParams, cp: &str, nat: &Path, feat: &HashMap<String, bool>) {
    match arg {
        ArgumentEntry::Plain(s) => out.push(repl(s, info, run, root, assets, acc, p, cp, nat)),
        ArgumentEntry::Conditional(c) => {
            if c.rules.as_ref().map(|r| rules::evaluate_rules(r, feat)).unwrap_or(true) {
                match &c.value {
                    ArgumentValue::Single(s) => out.push(repl(s, info, run, root, assets, acc, p, cp, nat)),
                    ArgumentValue::Multiple(v) => for s in v { out.push(repl(s, info, run, root, assets, acc, p, cp, nat)); }
                }
            }
        }
    }
}

fn repl(s: &str, info: &VersionInfo, run: &Path, root: &Path, assets: &Path, acc: &Account, p: &LaunchParams, cp: &str, nat: &Path) -> String {
    s.replace("${auth_player_name}", &acc.player_name)
     .replace("${version_name}", &info.id)
     .replace("${game_directory}", &run.to_string_lossy())
     .replace("${assets_root}", &assets.to_string_lossy())
     .replace("${assets_index_name}", info.asset_index.as_ref().and_then(|a| a.id.as_ref()).or(info.assets.as_ref()).unwrap_or(&"legacy".to_string()))
     .replace("${auth_uuid}", &acc.uuid)
     .replace("${auth_access_token}", acc.access_token.as_deref().unwrap_or("0"))
     .replace("${user_type}", "mojang")
     .replace("${version_type}", info.version_type.as_deref().unwrap_or("release"))
     .replace("${user_properties}", "{}")
     .replace("${game_assets}", &assets.to_string_lossy())
     .replace("${auth_session}", acc.access_token.as_deref().unwrap_or("0"))
     .replace("${classpath}", cp)
     .replace("${natives_directory}", &nat.to_string_lossy())
     .replace("${launcher_name}", "custom")
     .replace("${launcher_version}", "1.0")
     .replace("${classpath_separator}", Platform::current().classpath_separator())
     .replace("${library_directory}", &root.join("libraries").to_string_lossy())
     .replace("${resolution_width}", &p.window_width.unwrap_or(854).to_string())
     .replace("${resolution_height}", &p.window_height.unwrap_or(480).to_string())
}

pub async fn launch_game(app: AppHandle, root: &Path, params: &LaunchParams, account: &Account, isolation: bool) -> Result<Child, AppError> {
    let cmd = build_launch_command(root, params, account, isolation).await?;
    if cmd.is_empty() { return Err(AppError::LaunchFailed("Empty command".to_string())); }
    let mut child = Command::new(&cmd[0]).args(&cmd[1..]).current_dir(root).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().map_err(|e| AppError::LaunchFailed(e.to_string()))?;
    let pid = child.id().unwrap_or(0);
    let _ = app.emit("game-log", format!("[启动] PID: {}", pid));
    let _ = app.emit("game-log", format!("[目录] {}", root.to_string_lossy()));
    
    if let Some(out) = child.stdout.take() { let app = app.clone(); let mut r = BufReader::new(out).lines(); tokio::spawn(async move { while let Ok(Some(l)) = r.next_line().await { let _ = app.emit("game-log", l); } }); }
    if let Some(err) = child.stderr.take() { let mut r = BufReader::new(err).lines(); tokio::spawn(async move { while let Ok(Some(l)) = r.next_line().await { let _ = app.emit("game-log", format!("[E] {}", l)); } }); }
    
    Ok(child)
}