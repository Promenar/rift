# Rift 简体中文界面

本仓库基于上游 [Rift v0.7.2](https://github.com/acsandmann/rift/tree/v0.7.2)（提交 `572b9425ebe3d09196e5db2601ff8bc539ee407c`），提供简体中文的原生设置界面、菜单、总览、搜索和提示。

仓库由上游原作者仓库 Fork 而来。`origin` 指向自有仓库 `Promenar/rift`，`upstream` 指向 `acsandmann/rift` 只读来源；本地提交只进入自有仓库，不向上游推送或创建 PR。

## 分支

| 分支 | 内容 |
| --- | --- |
| `main` | 上游最新 `main` 加简体中文提交，作为长期维护线 |
| `zh-cn/v0.7.0` | v0.7.0 基线的完整汉化与多屏页签修复历史，保留用于对比和回退 |

## 使用范围

界面标题、设置说明、布局选项、规则与快捷键编辑器、搜索、菜单栏、总览和提示使用简体中文。配置键名、配置值、命令接口与默认快捷键沿用上游格式；用户填写的工作区名称、应用名称、路径和窗口标题保持原样。

上游终端帮助、配置示例注释和底层技术错误详情保留英文。汉化不会自动改名已有工作区。

内建控件来自依赖 `cgs 0.4.0`。项目在 `vendor/cgs/` 固定其源码，仅汉化用户可见文案（按钮、导航、路径选择、快捷键录制与无障碍文字）；版本、依赖行为和 GPL-3.0 许可保持不变。来源归档的 SHA-256 记录在 `vendor/cgs/source-provenance.json`。

## 与 v0.7.0 汉化的差异

- 设置界面、菜单栏、总览和搜索按 v0.7.2 的新结构重新汉化，并补译上游新增的布局说明、快捷键表头、Mission Control 提示与搜索索引。
- 枚举选项标签按“枚举名 + 变体”从旧汉化移植回 `ConfigEnum`。
- 上游 v0.7.2 已把 Cocoa/Quartz 坐标转换基准改为主显示器高度，覆盖了 v0.7.0 的多屏页签 218 点偏移问题；本仓库不再保留旧 `src/sys/screen.rs` 补丁，也未在本次进行真实多屏热插拔验收。
- v0.7.2 的设置界面比 v0.7.0 更宽：英文界面在 700pt 下最小约需 704pt，中文约需 713pt。原生窄窗口断言下限已调整为 720pt，700pt 窄窗口不再作为 v0.7.2 的保证。

## 安装状态与回退

2026-11-02 已在本机安装并替换为 v0.7.2 中文版：

- 版本目录：`~/Library/Application Support/Rift/Versions/0.7.2-zh-cn-20261102-000648`
- `/opt/homebrew/bin/rift` 与 `rift-cli` 指向该目录，`rift --version` 输出 `0.7.2+b673316a5e17`
- 升级更换了可执行文件，macOS 要求重新授予辅助功能权限。旧版本条目可在“系统设置 > 隐私与安全性 > 辅助功能”中手动选中并点 `-` 移除；保留指向上述 0.7.2 目录的最新条目。不要执行 `tccutil reset Accessibility`，它会同时撤销当前可用授权
- 用户配置、快捷键、应用规则与 LaunchAgent 未改

回退到安装前的 0.7.0 中文版：

```sh
python3 "$HOME/Library/Application Support/Rift/Backups/20261102-005502-v072-authorized/rollback.py" --check
python3 "$HOME/Library/Application Support/Rift/Backups/20261102-005502-v072-authorized/rollback.py"
```

更早的备份保留在 `~/Library/Application Support/Rift/Backups/`：
`20261027-091741-tab-coordinate`（0.7.0 页签修复版）、`20261019-192206`（首次中文版）以及
`20261102-000648-v072-install`、`20261102-001409-v072-inplace`（两次被辅助功能授权拦截的安装尝试）。

## 构建与验证

依赖 macOS 原生框架，使用本机 macOS arm64、Rust/Cargo 与 Xcode。验证命令：

```sh
cargo check --locked -j 2
cargo test --locked --lib -j 2
cargo test --locked --test settings_native -j 2 -- --ignored --verify
cargo build --release --locked -j 2 --package rift-wm --bin rift --bin rift-cli --target aarch64-apple-darwin
```

2026-11-01 在 v0.7.2 基线上的实际结果：

| 验证 | 结果 |
| --- | --- |
| `cargo check --locked -j 2` | 通过 |
| `cargo test --locked --lib -j 2` | 881 项通过，0 失败 |
| `cargo test --locked --test settings_native -j 2 -- --ignored --verify` | 原生控件、配置提交与同步、更新状态、窄窗口切换和关闭检查通过 |
| arm64 release 构建 | `rift` 与 `rift-cli` 生成成功 |

构建产物位于 `target/aarch64-apple-darwin/release/`。本机工具链未安装 rustfmt，因此未运行格式化；编译、锁定依赖检查、单元测试、原生验证和 `git diff --check` 均已通过。

## 许可

Rift 代码沿用 [Apache License 2.0](LICENSE)。`cgs` 保留其独立的 [GPL-3.0 许可](vendor/cgs/LICENSE)，未将其改标为 Apache。
