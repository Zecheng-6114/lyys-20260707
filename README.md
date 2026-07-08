# Lyys Launcher

基于 Tauri 2.x + Vue 3 的 Minecraft 启动器。

## 技术栈

| 层 | 技术 |
|---|---|
| 桌面框架 | Tauri 2.x |
| 前端 | Vue 3 + TypeScript + Vite |
| 后端 | Rust (tokio 异步) |

## 功能

- 离线账号登录
- Java 自动检测与手动指定
- 本地版本扫描（含 inheritsFrom 继承链合并）
- classpath 构建、natives 解压
- 游戏参数构建与进程启动
- 游戏进程监控（启动/停止/退出监听）
- 版本隔离
- 内存与分辨率设置

## 开发

```bash
# 安装依赖
npm install

# 开发模式
npm run tauri dev

# 构建
npm run tauri build
```

## 项目结构

```
src/              # Vue 前端
src-tauri/        # Rust 后端
  src/
    commands.rs   # Tauri IPC 命令
    models.rs     # 数据结构
    services/     # 核心逻辑
      launcher.rs # 启动命令构建
      version.rs  # 版本解析
      library.rs  # 依赖库处理
      rules.rs    # 参数规则评估
      java.rs     # Java 检测
      config.rs   # 配置管理
```
