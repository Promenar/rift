# Rift 简体中文界面

本仓库基于上游 [Rift v0.8.3](https://github.com/acsandmann/rift/tree/v0.8.3)（提交 `165e29a567f5858aee2ed09fe9e2233b50b55576`），提供简体中文的原生设置界面、菜单、总览、搜索和提示。

仓库由上游原作者仓库 Fork 而来。`origin` 指向自有仓库 `Promenar/rift`，`upstream` 指向 `acsandmann/rift` 只读来源；本地提交只进入自有仓库，不向上游推送或创建 PR。

## 分支

| 分支 | 内容 |
| --- | --- |
| `main` | 上游最新 `main` 加简体中文提交，作为长期维护线 |
| `zh-cn/v0.8.3` | v0.8.3 汉化迁移的验证分支，验证通过后合入 `main` |
| `zh-cn/v0.7.0` | v0.7.0 基线的完整汉化与多屏页签修复历史，保留用于对比和回退 |

## 使用范围

界面标题、设置说明、布局选项、规则与快捷键编辑器、搜索、菜单栏、总览和提示使用简体中文。配置键名、配置值、命令接口与默认快捷键沿用上游格式；用户填写的工作区名称、应用名称、路径、窗口标题和键位表名称保持原样。

快捷键页的动作名称来自 `#[command(label = …)]` 命令目录，已随目录本地化；方向与尺寸选项显示为「左 / 右 / 上 / 下」「水平 / 垂直」。上游终端帮助、配置示例注释、底层技术错误详情和日志保留英文。汉化不会自动改名已有工作区。

内建控件来自依赖 `cgs 0.5.2`。项目在 `vendor/cgs/` 固定其源码，仅汉化用户可见文案（按钮、导航、路径选择、快捷键录制与无障碍文字）；版本、依赖行为和 GPL-3.0 许可保持不变。来源归档的 SHA-256 记录在 `vendor/cgs/source-provenance.json`。

## 与 v0.7.2 汉化的差异

- 上游 75 个提交带入的界面按 v0.8.3 结构重新汉化：重写的快捷键页（动作目录、键位表管理、命令参数编辑器）、应用规则的原始窗口标签编辑器、Mission Control 新增选项和菜单栏显示方式选择器。
- 快捷键动作目录从页面内的本地列表迁到 `rift-protocol` 与 `wm_controller` 的 `#[command(label)]` 属性；所有动作项已补齐中文标题，原先依赖自动英文命名的项也显式标注。
- 命令参数编辑器补充中文标签：方向、水平/垂直、显示器、文件、数值、增量等；未列出的字段回退为按字段名生成的英文标签。
- `cgs` 由 0.4.0 升至 0.5.2，新增的快捷键录制与快捷键设置组件文案已本地化，原有 39 条对照全部沿用。
- 上游把默认工作区模式改为 `global`（所有显示器共用一组工作区）。本仓库**沿用上游默认值**，未改回 `per_display`；需要旧行为时在配置中设置 `workspace_mode = "per_display"`。
- 上游把测试固定为单线程（`.cargo/config.toml` 设置 `RUST_TEST_THREADS=1`），本仓库保留该设置。

## 安装状态与回退

当前运行版本仍是 2026-11-02 安装的 v0.7.2 中文版（`~/Library/Application Support/Rift/Versions/0.7.2-zh-cn-20261102-000648`）。v0.8.3 尚未安装，只在 `target/aarch64-apple-darwin/release/` 生成了候选二进制。

升级安装时注意：

- 新二进制必须重新在“系统设置 > 隐私与安全性 > 辅助功能”中手动点 `+` 添加并授权；重新构建或签名不会继承旧授权。只加 `rift`，不需要 `rift-cli`。
- 授权后不要再重新 `codesign` 或替换该文件，否则 cdhash 变化会让授权失效。
- 旧条目只能在列表中手动选中后点 `-` 移除；不要执行 `tccutil reset Accessibility`，它会同时撤销当前可用授权。
- 上游 v0.8.3 发布物改为 `Rift.app` 包（bundle id `git.acsandmann.rift`）并保留旧的裸二进制 tarball；本仓库从源码构建，仍产出裸二进制，安装流程不变。

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
cargo test --locked --test settings_native -j 2 -- --ignored --keyboard
cargo test --locked --test settings_native -j 2 -- --ignored --about
cargo build --release --locked -j 2 --package rift-wm --bin rift --bin rift-cli --target aarch64-apple-darwin
```

2026-11-18 在 v0.8.3 基线上的实际结果：

| 验证 | 结果 |
| --- | --- |
| `cargo check --locked -j 2` | 通过 |
| `cargo test --locked --lib -j 2` | 911 项通过，0 失败 |
| `settings_native --ignored --keyboard` | 快捷键录制、冲突处理、目录选择通过 |
| `settings_native --ignored --about` | 更新状态与请求清理通过 |
| 设置页截图（`--render --page 0..10`） | 11 个页面均为中文渲染 |
| arm64 release 构建 | `rift` 与 `rift-cli` 生成成功 |

已知问题：`settings_native --ignored --verify` 在 `search replaces the sidebar` 断言失败，**纯上游 v0.8.3 在同一台机器上同样失败**，与本仓库汉化无关；该断言依赖搜索态下侧栏视图被替换的时序。快捷键与关于页另有独立入口，已单独通过。

构建产物位于 `target/aarch64-apple-darwin/release/`。本机工具链未安装 rustfmt，因此未运行格式化；编译、锁定依赖检查、单元测试、原生快捷键/关于验证、页面截图和 `git diff --check` 均已通过。

## 许可

Rift 代码沿用 [Apache License 2.0](LICENSE)。`cgs` 保留其独立的 [GPL-3.0 许可](vendor/cgs/LICENSE)，未将其改标为 Apache。
