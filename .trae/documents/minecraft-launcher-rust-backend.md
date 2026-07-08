# Minecraft Launcher Rust 后端实现计划

## Context

借鉴 CMCL（Console Minecraft Launcher，Java）的核心逻辑，用 Rust 重写 Minecraft 启动器后端，集成到现有的 Tauri + Vue 3 桌面应用中。不直接集成 CMCL，而是参考其算法独立实现。

## 模块结构

```
src-tauri/src/
├── main.rs              # 入口（已有）
├── lib.rs               # Tauri 注册（修改）
├── error.rs             # 统一错误类型
├── state.rs             # Tauri 管理状态
├── models/              # 数据结构
│   ├── mod.rs
│   ├── version.rs       # VersionManifest, VersionInfo, VersionMeta
│   ├── library.rs       # Library, ArtifactEntry, Classifiers
│   ├── arguments.rs     # ArgumentEntry, ArgumentRule, ArgumentValue
│   ├── account.rs       # Account, AccountType
│   ├── config.rs        # LauncherConfig, LaunchParams
│   └── download.rs      # DownloadTask, DownloadProgress
├── services/            # 核心业务逻辑
│   ├── mod.rs
│   ├── version.rs       # 解析 version.json，处理 inheritsFrom
│   ├── launcher.rs      # 构建启动命令，启动进程
│   ├── library.rs       # 解析依赖库，构建 classpath
│   ├── assets.rs        # 解析 asset index
│   ├── rules.rs         # 参数规则评估（OS、features）
│   ├── java.rs          # Java 检测
│   ├── account.rs       # 离线账号、微软登录
│   ├── download.rs      # 多线程下载引擎
│   ├── install.rs       # 版本安装流水线
│   └── config.rs        # 配置读写
├── api/                 # 下载源
│   ├── mod.rs
│   ├── traits.rs        # DownloadApiProvider trait
│   ├── mojang.rs        # 官方源
│   └── bmclapi.rs       # BMCLAPI 镜像源
├── commands/            # Tauri 命令（薄封装）
│   ├── mod.rs
│   ├── version_cmd.rs   # 版本管理
│   ├── launch_cmd.rs    # 启动游戏
│   ├── account_cmd.rs   # 账号管理
│   ├── config_cmd.rs    # 配置管理
│   └── download_cmd.rs  # 下载相关
└── utils/               # 工具
    ├── mod.rs
    ├── fs.rs            # 文件读写
    ├── hash.rs          # SHA-1 校验
    ├── platform.rs      # 平台检测
    └── uuid.rs          # 离线 UUID 生成
```

## 分阶段实现

### 第一阶段：核心启动（先实现这个）

目标：能用离线账号启动已安装的 Minecraft

1. **error.rs** — AppError 枚举（thiserror）
2. **models/** — 所有数据结构
3. **utils/** — 平台检测、文件工具、UUID、SHA-1
4. **services/config.rs** — 配置读写（JSON）
5. **services/version.rs** — 解析 version.json，处理 inheritsFrom 继承
6. **services/rules.rs** — 参数规则评估（参考 CMCL `isMeetConditions`）
7. **services/library.rs** — 解析依赖库路径，构建 classpath
8. **services/launcher.rs** — 构建启动命令，替换变量，启动进程（参考 CMCL `MinecraftLauncher`）
9. **services/java.rs** — Java 路径检测
10. **commands/** — launch_game, list_local_versions, get_config 等
11. **state.rs + lib.rs** — 注册命令和状态

### 第二阶段：下载与安装

目标：能下载并安装新版本

1. **api/** — 下载源（官方 + BMCLAPI）
2. **services/download.rs** — 多线程下载 + 进度回调（Tauri Events）
3. **services/assets.rs** — Asset 索引解析
4. **services/install.rs** — 完整安装流水线

### 第三阶段：账号系统

目标：微软账号登录、多账号管理

1. **services/account.rs** — 微软 OAuth 流程
2. **commands/account_cmd.rs** — 账号管理命令

### 第四阶段：Mod 加载器

目标：安装 Forge、Fabric、OptiFine

## 关键 Cargo 依赖

```toml
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"], default-features = false }
thiserror = "2"
uuid = { version = "1", features = ["v3", "v4", "serde"] }
sha1 = "0.10"
futures = "0.3"
regex = "1"
dirs = "6"
zip = "2"
base64 = "0.22"
rand = "0.8"
```

## Tauri Commands API

| 命令 | 说明 |
|------|------|
| `list_local_versions` | 列出已安装版本 |
| `fetch_version_manifest` | 获取版本列表 |
| `install_version` | 安装版本（带进度事件） |
| `launch_game` | 启动游戏（返回 PID） |
| `stop_game` | 停止游戏进程 |
| `list_accounts` | 列出账号 |
| `add_offline_account` | 添加离线账号 |
| `login_microsoft` | 微软登录 |
| `get_config` / `update_config` | 配置读写 |
| `detect_java` | 检测 Java |

## 验证方式

1. 第一阶段完成后：手动放一个已下载的 Minecraft 版本到 `.minecraft/versions/`，通过前端调用 `launch_game` 启动
2. 检查启动命令是否正确（`get_launch_command` 调试用）
3. 第二阶段完成后：通过前端下载安装一个新版本
