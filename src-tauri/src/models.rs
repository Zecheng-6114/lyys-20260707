use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LauncherConfig {
    pub game_directory: Option<String>,
    pub java_path: Option<String>,
    pub max_memory_mb: Option<u64>,
    pub accounts: Vec<Account>,
    pub version_isolation: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub player_name: String,
    pub uuid: String,
    pub account_type: String,
    pub access_token: Option<String>,
    pub is_selected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchParams {
    pub version_name: String,
    pub account_index: Option<usize>,
    pub java_path: Option<String>,
    pub max_memory_mb: Option<u64>,
    pub min_memory_mb: Option<u32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub fullscreen: Option<bool>,
    pub game_directory: Option<String>,
    pub server_address: Option<String>,
    pub extra_jvm_args: Option<Vec<String>>,
    pub extra_game_args: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: Option<String>,
    #[serde(rename = "mainClass")]
    pub main_class: Option<String>,
    pub assets: Option<String>,
    #[serde(rename = "assetIndex")]
    pub asset_index: Option<AssetIndex>,
    pub libraries: Option<Vec<LibraryEntry>>,
    pub arguments: Option<Arguments>,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
    pub jar: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    pub id: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub total_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    pub component: Option<String>,
    pub major_version: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Option<Vec<ArgumentEntry>>,
    pub jvm: Option<Vec<ArgumentEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentEntry {
    Plain(String),
    Conditional(ConditionalArgument),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalArgument {
    pub rules: Option<Vec<Rule>>,
    pub value: ArgumentValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
    pub features: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryEntry {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub natives: Option<HashMap<String, String>>,
    pub rules: Option<Vec<Rule>>,
    pub extract: Option<ExtractConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<DownloadArtifact>,
    pub classifiers: Option<HashMap<String, DownloadArtifact>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadArtifact {
    pub url: Option<String>,
    pub sha1: Option<String>,
    pub size: Option<u64>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractConfig {
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct ResolvedLibrary {
    pub local_path: std::path::PathBuf,
    pub is_native: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalVersionInfo {
    pub id: String,
    pub version_type: Option<String>,
    pub has_jar: bool,
    pub inherits_from: Option<String>,
}